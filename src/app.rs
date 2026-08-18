use crate::blueprint::{Blueprint, list_blueprints};
use crate::draft::{Draft, DraftManager};
use crate::form::Form;
use anyhow::Result;
use std::fs;
use std::path::PathBuf;
use std::sync::mpsc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Menu,
    Drafts,
    Form,
    Review,
    Success,
}

pub struct App {
    pub current_screen: Screen,
    pub blueprints: Vec<Blueprint>,
    pub selected_blueprint: usize,
    pub current_blueprint: Option<Blueprint>,
    pub form: Form,
    pub should_quit: bool,
    pub generate_prompt: bool,
    pub copy_to_clipboard: bool,
    pub status_message: Option<String>,
    pub blueprints_dir: String,
    pub is_loading: bool,
    pub loading_message: String,
    pub preview_scroll: u16,
    pub last_output_path: Option<String>,
    pub last_copy_result: Option<String>,
    pub last_error: Option<String>,
    // ── NEW: Draft state ──
    pub draft_manager: DraftManager,
    pub drafts: Vec<(String, Draft)>,
    pub selected_draft: usize,
    pub current_draft_name: Option<String>,

    pub navbar_scroll_offset: usize,

    // ── NEW: Error modal state ──
    pub error_title: Option<String>,
    pub error_message: Option<String>,
    pub error_details: Option<String>,

    // ── NEW: Async search state ──
    pub search_receiver:
        Option<mpsc::Receiver<Result<crate::package_registry::PackageInfo, String>>>,
    pub search_target_key: Option<String>,
    pub search_add_to_list: bool,
}

impl App {
    pub fn new(custom_blueprints_dir: Option<String>) -> Self {
        let blueprints_dir = Self::find_blueprints_dir(custom_blueprints_dir);
        let blueprints = list_blueprints(&blueprints_dir);
        let draft_manager = DraftManager::new();
        let drafts = draft_manager.list_drafts();

        let status = if blueprints.is_empty() {
            Some(format!(
                "No blueprints found in '{}' (cwd: '{}')",
                blueprints_dir,
                std::env::current_dir()
                    .map(|p| p.display().to_string())
                    .unwrap_or_else(|_| "unknown".into())
            ))
        } else {
            Some(format!(
                "Loaded {} blueprint(s), {} draft(s)",
                blueprints.len(),
                drafts.len()
            ))
        };

        Self {
            current_screen: Screen::Drafts,
            blueprints,
            selected_blueprint: 0,
            current_blueprint: None,
            form: Form::new(&Blueprint {
                name: String::new(),
                description: String::new(),
                icon: String::new(),
                sections: vec![],
            }),
            should_quit: false,
            generate_prompt: false,
            copy_to_clipboard: false,
            status_message: status,
            blueprints_dir,
            is_loading: false,
            loading_message: String::new(),
            preview_scroll: 0,
            last_output_path: None,
            last_copy_result: None,
            last_error: None,
            draft_manager,
            drafts,
            selected_draft: 0,
            current_draft_name: None,
            navbar_scroll_offset: 0,
            error_title: None,
            error_message: None,
            error_details: None,
            search_receiver: None,
            search_target_key: None,
            search_add_to_list: false,
        }
    }

    fn find_blueprints_dir(custom_path: Option<String>) -> String {
        // 1. Use custom path if provided via CLI
        if let Some(custom) = custom_path {
            if PathBuf::from(&custom).exists() {
                return custom;
            }
        }

        // 2. Check environment variable override
        if let Ok(dir) = std::env::var("BLUEPRINTS_DIR") {
            if PathBuf::from(&dir).exists() {
                return dir;
            }
        }

        // 3. Current working directory (for development)
        let cwd_blueprints = PathBuf::from("blueprints");
        if cwd_blueprints.exists() {
            return "blueprints".to_string();
        }

        // 4. OS-specific app data directory
        let mut app_blueprints = get_app_data_dir();
        app_blueprints.push("blueprints");
        if app_blueprints.exists() {
            return app_blueprints.to_string_lossy().to_string();
        }

        // Fallback
        "blueprints".to_string()
    }

    pub fn menu_next(&mut self) {
        if !self.blueprints.is_empty() {
            self.selected_blueprint = (self.selected_blueprint + 1) % self.blueprints.len();
        }
    }

    pub fn menu_prev(&mut self) {
        if !self.blueprints.is_empty() {
            self.selected_blueprint = if self.selected_blueprint == 0 {
                self.blueprints.len() - 1
            } else {
                self.selected_blueprint - 1
            };
        }
    }

    pub fn select_blueprint(&mut self) -> Result<()> {
        if let Some(blueprint) = self.blueprints.get(self.selected_blueprint) {
            self.current_blueprint = Some(blueprint.clone());
            self.form = Form::new(blueprint);
            self.current_screen = Screen::Form;
            self.current_draft_name = None;
            self.navbar_scroll_offset = 0; // <-- Reset navbar scroll
            self.status_message = Some(format!("Loaded: {}", blueprint.name));
        }
        Ok(())
    }

    pub fn refresh_blueprints(&mut self) {
        self.blueprints = list_blueprints(&self.blueprints_dir);
        self.selected_blueprint = 0;
        self.status_message = Some(format!(
            "Found {} blueprint(s) in '{}'",
            self.blueprints.len(),
            self.blueprints_dir
        ));
    }

    pub fn refresh_drafts(&mut self) {
        self.drafts = self.draft_manager.list_drafts();
        self.selected_draft = 0;
    }

    // ── NEW: Draft methods ──

    pub fn drafts_next(&mut self) {
        if !self.drafts.is_empty() {
            self.selected_draft = (self.selected_draft + 1) % self.drafts.len();
        }
    }

    pub fn drafts_prev(&mut self) {
        if !self.drafts.is_empty() {
            self.selected_draft = if self.selected_draft == 0 {
                self.drafts.len() - 1
            } else {
                self.selected_draft - 1
            };
        }
    }

    pub fn load_selected_draft(&mut self) -> Result<()> {
        if let Some((filename, draft)) = self.drafts.get(self.selected_draft) {
            let blueprint = self
                .blueprints
                .iter()
                .find(|b| b.name == draft.blueprint_name);

            if let Some(bp) = blueprint {
                self.current_blueprint = Some(bp.clone());
                self.form = Form::new(bp);
                self.form.values = draft.values.clone();
                self.form.current_section = draft.current_section;
                self.form.selected_field = draft.selected_field;
                self.current_draft_name = Some(filename.clone());
                self.current_screen = Screen::Form;
                self.navbar_scroll_offset = 0; // <-- Reset navbar scroll
                self.status_message = Some(format!("Loaded draft: {}", draft.name));
            } else {
                self.status_message = Some(format!(
                    "⚠ Blueprint '{}' not found for draft",
                    draft.blueprint_name
                ));
            }
        }
        Ok(())
    }

    pub fn save_current_draft(&mut self) -> Result<()> {
        if let Some(blueprint) = &self.current_blueprint {
            // Use existing draft name if we have one, otherwise create a new name
            let draft_name = self.current_draft_name.clone().unwrap_or_else(|| {
                format!("{}_draft", blueprint.name.to_lowercase().replace(' ', "_"))
            });

            let mut draft = Draft::new(
                draft_name.clone(),
                blueprint.name.clone(),
                self.form.values.clone(),
                self.form.current_section,
                self.form.selected_field,
            );

            // If this is an existing draft, preserve the created_at timestamp
            if let Some(existing_name) = &self.current_draft_name {
                let path = self
                    .draft_manager
                    .drafts_dir()
                    .join(format!("{}.json", existing_name));
                if path.exists() {
                    if let Ok(existing) = self.draft_manager.load_draft(&path) {
                        draft.created_at = existing.created_at;
                    }
                }
            }

            self.draft_manager.save_draft(&draft_name, &draft)?;
            self.current_draft_name = Some(draft_name.clone());
            self.status_message = Some(format!("✓ Draft saved: {}", draft_name));
            self.refresh_drafts();
        }
        Ok(())
    }

    pub fn delete_selected_draft(&mut self) -> Result<()> {
        if let Some((filename, _)) = self.drafts.get(self.selected_draft) {
            self.draft_manager.delete_draft(filename)?;
            self.refresh_drafts();
            self.status_message = Some("Draft deleted".to_string());
        }
        Ok(())
    }

    #[allow(dead_code)]
    pub fn navbar_scroll_left(&mut self) {
        self.navbar_scroll_offset = self.navbar_scroll_offset.saturating_sub(1);
    }
    #[allow(dead_code)]
    pub fn navbar_scroll_right(&mut self) {
        if let Some(blueprint) = &self.current_blueprint {
            let max_offset = blueprint.sections.len().saturating_sub(1);
            self.navbar_scroll_offset = (self.navbar_scroll_offset + 1).min(max_offset);
        }
    }

    /// Ensure the current section is visible in the navbar
    pub fn ensure_navbar_visible(&mut self, visible_tab_count: usize) {
        if visible_tab_count == 0 {
            return;
        }

        let current = self.form.current_section;

        // If current section is to the left of visible area, scroll left
        if current < self.navbar_scroll_offset {
            self.navbar_scroll_offset = current;
        }
        // If current section is to the right of visible area, scroll right
        else if current >= self.navbar_scroll_offset + visible_tab_count {
            self.navbar_scroll_offset = current.saturating_sub(visible_tab_count - 1);
        }
    }

    /// Show an error modal
    pub fn show_error(&mut self, title: &str, message: &str) {
        self.error_title = Some(title.to_string());
        self.error_message = Some(message.to_string());
        self.error_details = None;
    }

    /// Dismiss the error modal
    pub fn dismiss_error(&mut self) {
        self.error_title = None;
        self.error_message = None;
        self.error_details = None;
    }

    /// Check if error modal is showing
    pub fn has_error(&self) -> bool {
        self.error_message.is_some()
    }
}

/// Gets the OS-specific application data directory
/// Linux: ~/.local/share/yerkoku
/// macOS: ~/Library/Application Support/yerkoku
/// Windows: C:\Users\Username\AppData\Roaming\yerkoku
pub fn get_app_data_dir() -> PathBuf {
    let mut path = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("yerkoku"); // Your app name

    if !path.exists() {
        let _ = fs::create_dir_all(&path);
    }
    path
}
