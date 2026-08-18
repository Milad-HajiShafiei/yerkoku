use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::{fs, io::Write};
use std::{io, sync::mpsc};
use clap::Parser;

mod app;
mod blueprint;
mod draft;
mod form;
mod package_registry;
mod prompt;
mod ui;
mod widgets;

use app::{App, Screen};
use blueprint::{FieldType, FieldValue};


// ─────────────────────────────────────────────
// Embed default blueprints into the binary at compile time
// ─────────────────────────────────────────────
const DEFAULT_BACKEND: &str = include_str!("../blueprints/backend.json");
const DEFAULT_FRONTEND: &str = include_str!("../blueprints/frontend.json");
const DEFAULT_MOBILE: &str = include_str!("../blueprints/mobile.json");
const DEFAULT_DESKTOP: &str = include_str!("../blueprints/desktop.json");


// ─────────────────────────────────────────────
// Debug logging helper (only active in debug builds)
// ─────────────────────────────────────────────
#[cfg(debug_assertions)]
fn debug_log(msg: &str) {
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("debug.log")
    {
        let _ = writeln!(file, "[{}] {}", chrono::Local::now().format("%H:%M:%S%.3f"), msg);
    }
}

#[cfg(not(debug_assertions))]
fn debug_log(_msg: &str) {
    // No-op in release builds — no file created
}

// ─────────────────────────────────────────────
// CLI Argument Parser
// ─────────────────────────────────────────────
#[derive(Parser, Debug)]
#[command(
    author, 
    version, 
    about = "Generate comprehensive AI development prompts from JSON blueprints.",
    long_about = None
)]
struct Cli {
    /// Force reset and re-install default blueprints into the app data directory
    #[arg(short, long)]
    init: bool,

    /// Custom path to blueprints directory (overrides default app data location)
    #[arg(short, long)]
    blueprints: Option<String>,
}

// ─────────────────────────────────────────────
// Initialization Logic
// ─────────────────────────────────────────────
fn init_default_blueprints(force: bool) {
    let mut blueprints_dir = app::get_app_data_dir();
    blueprints_dir.push("blueprints");

    // Check if we need to initialize
    let needs_init = if !blueprints_dir.exists() {
        true
    } else if force {
        true // User passed --init flag
    } else {
        // Check if directory is empty or has no json files
        match fs::read_dir(&blueprints_dir) {
            Ok(entries) => {
                let has_json = entries.filter_map(Result::ok).any(|e| {
                    e.path().extension().and_then(|s| s.to_str()) == Some("json")
                });
                !has_json
            }
            Err(_) => true,
        }
    };

    if needs_init {
        if let Err(e) = fs::create_dir_all(&blueprints_dir) {
            eprintln!("⚠ Warning: Failed to create blueprints directory: {}", e);
            return;
        }

        let files = [
            ("backend.json", DEFAULT_BACKEND),
            ("frontend.json", DEFAULT_FRONTEND),
            ("mobile.json", DEFAULT_MOBILE),
            ("desktop.json", DEFAULT_DESKTOP),
        ];

        let mut success_count = 0;
        for (filename, content) in files {
            let path = blueprints_dir.join(filename);
            // Only write if forcing or file doesn't exist
            if force || !path.exists() {
                match fs::write(&path, content) {
                    Ok(_) => success_count += 1,
                    Err(e) => eprintln!("⚠ Warning: Failed to write {}: {}", filename, e),
                }
            }
        }

        if success_count > 0 {
            eprintln!(
                "✓ Initialized {} default blueprint(s) in: {}",
                success_count,
                blueprints_dir.display()
            );
        }
    }
}

// ─────────────────────────────────────────────
// Main entry point
// ─────────────────────────────────────────────
fn main() -> Result<()> {
    // 1. Parse CLI arguments
    let cli = Cli::parse();

    // 2. Initialize default blueprints (runs BEFORE App::new)
    init_default_blueprints(cli.init);

    // 3. Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // 4. Create App — pass custom blueprints path directly (no unsafe needed!)
    let mut app = App::new(cli.blueprints);

    // 5. Run the app
    let result = run_app(&mut terminal, &mut app);

    // 6. Cleanup terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = result {
        eprintln!("Application error: {}", err);
    }

    Ok(())
}

// ─────────────────────────────────────────────
// Main event loop
// ─────────────────────────────────────────────

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> Result<()> {
    debug_log("=== App started ===");

    loop {
        // Update visible field count
        if app.current_screen == Screen::Form {
            if let Some(blueprint) = &app.current_blueprint {
                if blueprint.sections.get(app.form.current_section).is_some() {
                    let terminal_height = terminal.size()?.height;
                    let available_height = terminal_height.saturating_sub(8);
                    let avg_field_height = 3;
                    let estimated_visible = (available_height / avg_field_height) as usize;
                    app.form.set_visible_field_count(estimated_visible.max(1));
                }
            }
        }

        // ── Check if background search completed ──
        check_search_completion(app);

        // Render UI
        terminal.draw(|f| ui::render(f, app))?;

        // Read event with timeout so we can check search completion
        let has_event = event::poll(std::time::Duration::from_millis(50))?;
        if !has_event {
            continue; // No event, loop back to check search and re-render
        }

        let event = event::read()?;

        // If error modal is showing, only handle dismiss
        if app.has_error() {
            if let Event::Key(key) = &event {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Enter | KeyCode::Esc | KeyCode::Char(' ') | KeyCode::Char('q') => {
                            app.dismiss_error();
                        }
                        _ => {}
                    }
                }
            }
            if let Event::Mouse(_) = &event {
                app.dismiss_error();
            }
            continue;
        }

        // Dispatch event
        match app.current_screen {
            Screen::Menu => handle_menu_event(app, &event)?,
            Screen::Drafts => handle_drafts_event(app, &event)?,
            Screen::Form => handle_form_event(app, &event)?,
            Screen::Review => handle_review_event(app, &event)?,
            Screen::Success => handle_success_event(app, &event)?,
        }

        // Handle prompt generation
        if app.generate_prompt {
            handle_prompt_generation(app);
        }

        // Check quit
        if app.should_quit {
            if app.current_blueprint.is_some() {
                let _ = app.save_current_draft();
            }
            break;
        }
    }

    Ok(())
}

// ─────────────────────────────────────────────
// Prompt generation handler
// ─────────────────────────────────────────────

fn handle_prompt_generation(app: &mut App) {
    if let Some(blueprint) = &app.current_blueprint {
        let prompt_text = prompt::generate_prompt(blueprint, &app.form);

        // Create prompts directory in OS app data
        let mut prompts_dir = crate::app::get_app_data_dir();
        prompts_dir.push("prompts");

        if !prompts_dir.exists() {
            let _ = std::fs::create_dir_all(&prompts_dir);
        }

        if let Err(e) = std::fs::create_dir_all(&prompts_dir) {
            app.show_error(
                "Directory Error",
                &format!("Failed to create prompts directory: {}", e),
            );
            app.generate_prompt = false;
            app.copy_to_clipboard = false;
            return;
        }

        let output_path = prompts_dir.join(format!(
            "{}_prompt.md",
            blueprint.name.to_lowercase().replace(' ', "_")
        ));

        match std::fs::write(&output_path, &prompt_text) {
            Ok(_) => {
                app.last_output_path = Some(output_path.display().to_string());

                if app.copy_to_clipboard {
                    match copy_to_clipboard(&prompt_text) {
                        Ok(_) => {
                            app.last_copy_result = Some("✓ Copied to clipboard".to_string());
                        }
                        Err(e) => {
                            app.last_copy_result = Some(format!("✗ Clipboard error: {}", e));
                        }
                    }
                } else {
                    app.last_copy_result = None;
                }

                app.last_error = None;
                let _ = app.save_current_draft();
            }
            Err(_e) => {
                app.show_error("Save Failed", &format!("Failed to save prompt to file"));
                app.last_output_path = None;
                app.last_copy_result = None;
            }
        }

        app.current_screen = Screen::Success;
        app.generate_prompt = false;
        app.copy_to_clipboard = false;
    }
}

// ─────────────────────────────────────────────
// Clipboard utility
// ─────────────────────────────────────────────

fn copy_to_clipboard(text: &str) -> Result<(), String> {
    let mut clipboard =
        arboard::Clipboard::new().map_err(|e| format!("Failed to initialize clipboard: {}", e))?;
    clipboard
        .set_text(text.to_string())
        .map_err(|e| format!("Failed to copy: {}", e))?;
    Ok(())
}

// ─────────────────────────────────────────────
// Menu screen handler
// ─────────────────────────────────────────────

fn handle_menu_event(app: &mut App, event: &Event) -> Result<()> {
    match event {
        Event::Key(key) => {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        app.should_quit = true;
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        app.menu_prev();
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        app.menu_next();
                    }
                    KeyCode::Char('r') => {
                        app.refresh_blueprints();
                    }
                    KeyCode::Enter => {
                        if let Err(e) = app.select_blueprint() {
                            app.show_error(
                                "Blueprint Error",
                                &format!("Failed to load blueprint: {}", e),
                            );
                        }
                    }
                    _ => {}
                }
            }
        }
        Event::Mouse(mouse) => {
            ui::handle_menu_mouse(app, mouse);
        }
        _ => {}
    }
    Ok(())
}

// ─────────────────────────────────────────────
// Drafts screen handler
// ─────────────────────────────────────────────

fn handle_drafts_event(app: &mut App, event: &Event) -> Result<()> {
    match event {
        Event::Key(key) => {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        app.should_quit = true;
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        app.drafts_prev();
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        app.drafts_next();
                    }
                    KeyCode::Enter => {
                        if let Err(e) = app.load_selected_draft() {
                            app.show_error("Draft Error", &format!("Failed to load draft: {}", e));
                        }
                    }
                    KeyCode::Char('d') => {
                        if let Err(e) = app.delete_selected_draft() {
                            app.show_error(
                                "Draft Error",
                                &format!("Failed to delete draft: {}", e),
                            );
                        }
                    }
                    KeyCode::Char('n') => {
                        app.current_screen = Screen::Menu;
                    }
                    KeyCode::Char('r') => {
                        app.refresh_drafts();
                    }
                    _ => {}
                }
            }
        }
        Event::Mouse(mouse) => {
            ui::handle_drafts_mouse(app, mouse);
        }
        _ => {}
    }
    Ok(())
}

// ─────────────────────────────────────────────
// Form screen handler
// ─────────────────────────────────────────────

fn handle_form_event(app: &mut App, event: &Event) -> Result<()> {
    match event {
        Event::Key(key) => {
            if key.kind == KeyEventKind::Press {
                handle_form_key(app, key.code, key.modifiers)?;
            }
        }
        Event::Mouse(mouse) => {
            ui::handle_form_mouse(app, mouse);
        }
        _ => {}
    }
    Ok(())
}

// ─────────────────────────────────────────────
// Form key handler (unified)
// ─────────────────────────────────────────────

fn handle_form_key(app: &mut App, code: KeyCode, modifiers: event::KeyModifiers) -> Result<()> {
    debug_log(&format!(
        "Form key: {:?}, editing: {}, sub_focus: {}",
        code,
        app.form.is_editing(),
        app.form.sub_focus
    ));

    // ── Global Ctrl shortcuts ──
    if modifiers.contains(event::KeyModifiers::CONTROL) {
        match code {
            KeyCode::Char('c') | KeyCode::Char('q') => {
                app.should_quit = true;
                return Ok(());
            }
            KeyCode::Char('s') => {
                app.generate_prompt = true;
                app.should_quit = true;
                return Ok(());
            }
            _ => {}
        }
    }

    // ── Get current field info ──
    let blueprint = match &app.current_blueprint {
        Some(bp) => bp.clone(),
        None => return Ok(()),
    };

    let section = match blueprint.sections.get(app.form.current_section) {
        Some(s) => s,
        None => return Ok(()),
    };

    let editable_fields: Vec<&blueprint::Field> = section
        .fields
        .iter()
        .filter(|f| !matches!(f.field_type, FieldType::SectionBreak) && !f.hidden)
        .collect();

    let field = match editable_fields.get(app.form.selected_field) {
        Some(f) => *f,
        None => return Ok(()),
    };

    let field_type = field.field_type.clone();
    let is_editing = app.form.is_editing();

    // ── EDITING MODE ──
    if is_editing {
        debug_log("In editing mode");
        match code {
            KeyCode::Esc => {
                app.form.stop_editing();
            }
            KeyCode::Enter => {
                if matches!(field_type, FieldType::Textarea) {
                    app.form.insert_char('\n');
                } else {
                    app.form.stop_editing();
                }
            }
            KeyCode::Tab => {
                app.form.stop_editing();
                app.form.next_field();
            }
            KeyCode::BackTab => {
                app.form.stop_editing();
                app.form.prev_field();
            }
            KeyCode::Backspace => {
                if matches!(field_type, FieldType::ListBuilder | FieldType::CrateSearch) {
                    app.form.list_input_backspace();
                } else {
                    app.form.backspace();
                }
            }
            KeyCode::Delete => {
                app.form.delete();
            }
            KeyCode::Left => {
                app.form.cursor_left();
            }
            KeyCode::Right => {
                app.form.cursor_right();
            }
            KeyCode::Home => {
                app.form.cursor_start();
            }
            KeyCode::End => {
                app.form.cursor_end();
            }
            KeyCode::PageUp => {
                if matches!(field_type, FieldType::Textarea) {
                    app.form.scroll_text_up(3);
                }
            }
            KeyCode::PageDown => {
                if matches!(field_type, FieldType::Textarea) {
                    app.form.scroll_text_down(3);
                }
            }
            KeyCode::Char(c) => {
                if matches!(field_type, FieldType::ListBuilder | FieldType::CrateSearch) {
                    app.form.insert_list_input_char(c);
                } else {
                    app.form.insert_char(c);
                }
            }
            _ => {}
        }
        return Ok(());
    }

    // ── NAVIGATION MODE ──
    debug_log("In navigation mode");
    match code {
        // Quit / Back
        KeyCode::Char('q') | KeyCode::Esc => {
            app.current_screen = Screen::Menu;
        }

        // Navigation - check sub_focus first
        KeyCode::Up | KeyCode::Char('k') => {
            if app.form.sub_focus == 2 {
                // ── List is focused: navigate list items ──
                match field_type {
                    FieldType::ListBuilder => {
                        // let items = app.form.get_current_value().as_vec();
                        if app.form.list_selected > 0 {
                            app.form.list_selected -= 1;
                        }
                    }
                    FieldType::CrateSearch => {
                        // let target_key = field
                        //     .target_list_key
                        //     .as_deref()
                        //     .unwrap_or("tech.additional_crates");
                        // let items = app.form.get_target_list(target_key);
                        if app.form.list_selected > 0 {
                            app.form.list_selected -= 1;
                        }
                    }
                    _ => {}
                }
            } else {
                // ── Normal navigation ──
                app.form.sub_focus = 0;
                app.form.prev_field();
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if app.form.sub_focus == 2 {
                // ── List is focused: navigate list items ──
                match field_type {
                    FieldType::ListBuilder => {
                        let items = app.form.get_current_value().as_vec();
                        if app.form.list_selected < items.len().saturating_sub(1) {
                            app.form.list_selected += 1;
                        }
                    }
                    FieldType::CrateSearch => {
                        let target_key = field
                            .target_list_key
                            .as_deref()
                            .unwrap_or("tech.additional_crates");
                        let items = app.form.get_target_list(target_key);
                        if app.form.list_selected < items.len().saturating_sub(1) {
                            app.form.list_selected += 1;
                        }
                    }
                    _ => {}
                }
            } else {
                // ── Normal navigation ──
                app.form.sub_focus = 0;
                app.form.next_field();
            }
        }

        // Tab navigation within composite widgets
        KeyCode::Tab => match field_type {
            FieldType::CrateInput => {
                if app.form.sub_focus == 0 {
                    app.form.sub_focus = 1;
                } else {
                    app.form.sub_focus = 0;
                    app.form.next_field();
                }
            }
            FieldType::CrateSearch | FieldType::ListBuilder => {
                if app.form.sub_focus == 0 {
                    app.form.sub_focus = 1;
                } else if app.form.sub_focus == 1 {
                    app.form.sub_focus = 2;
                } else {
                    app.form.sub_focus = 0;
                    app.form.next_field();
                }
            }
            _ => {
                app.form.sub_focus = 0;
                app.form.next_field();
            }
        },
        KeyCode::BackTab => match field_type {
            FieldType::CrateInput | FieldType::CrateSearch | FieldType::ListBuilder => {
                if app.form.sub_focus > 0 {
                    app.form.sub_focus -= 1;
                } else {
                    app.form.prev_field();
                }
            }
            _ => {
                app.form.sub_focus = 0;
                app.form.prev_field();
            }
        },

        // ── ENTER KEY ──
        KeyCode::Enter => {
            debug_log(&format!("Enter pressed on field type: {:?}", field_type));
            match field_type {
                FieldType::Text | FieldType::Textarea => {
                    app.form.start_editing();
                }
                FieldType::Checkbox => {
                    app.form.toggle_checkbox();
                }
                FieldType::Select => {
                    app.form.cycle_select(&field.options);
                }
                FieldType::SearchCrate => {
                    let package_name = field.search_crate.as_deref().unwrap_or("");
                    let registry = field.registry.clone();
                    if !package_name.is_empty() {
                        trigger_crate_search(app, package_name, false, &registry);
                    }
                }
                FieldType::CrateInput => {
                    if app.form.sub_focus == 1 {
                        let package_name = field
                            .crate_name
                            .as_deref()
                            .or(field.search_crate.as_deref())
                            .unwrap_or("");
                        let registry = field.registry.clone();
                        if !package_name.is_empty() {
                            trigger_crate_search(app, package_name, false, &registry);
                        }
                        app.form.sub_focus = 0;
                    } else {
                        app.form.start_editing();
                    }
                }
                                FieldType::ListBuilder => {
                    match app.form.sub_focus {
                        0 => {
                            // Start editing the INPUT BUFFER, not the main field
                            app.form.editing = true;
                            app.form.text_scroll_offset = 0;
                            let input_value = app.form.get_list_input_value();
                            app.form.cursor_pos = input_value.len();
                        }
                        1 => {
                            let value = app.form.get_list_input_value();
                            let trimmed = value.trim().to_string();
                            if !trimmed.is_empty() {
                                app.form.add_list_item(trimmed);
                                app.form.clear_list_input();
                            }
                            app.form.sub_focus = 0;
                        }
                        _ => {}
                    }
                }
                FieldType::CrateSearch => {
                    match app.form.sub_focus {
                        0 => {
                            // Start editing the INPUT BUFFER, not the main field
                            app.form.editing = true;
                            app.form.text_scroll_offset = 0;
                            let input_value = app.form.get_list_input_value();
                            app.form.cursor_pos = input_value.len();
                        }
                        1 => {
                            let value = app.form.get_list_input_value();
                            let trimmed = value.trim().to_string();
                            if !trimmed.is_empty() {
                                let target_key = field.target_list_key.as_deref()
                                    .unwrap_or("tech.additional_crates")
                                    .to_string();
                                let registry = field.registry.clone();
                                app.search_target_key = Some(target_key);
                                trigger_crate_search(app, &trimmed, true, &registry);
                            }
                            app.form.sub_focus = 0;
                        }
                        _ => {}
                    }
                }
                FieldType::ActionButton => {
                    // DO NOT trigger on Enter - just show hint
                    app.status_message = Some("Press SPACE to activate this button".to_string());
                }
                _ => {}
            }
        }

        // ── SPACE KEY ──
        KeyCode::Char(' ') => match field_type {
            FieldType::ActionButton => {
                let action = field.action.as_deref().unwrap_or("");
                match action {
                    "generate_copy" => {
                        app.generate_prompt = true;
                        app.copy_to_clipboard = true;
                    }
                    "generate_only" => {
                        app.generate_prompt = true;
                        app.copy_to_clipboard = false;
                    }
                    _ => {}
                }
            }
            FieldType::Checkbox => {
                app.form.toggle_checkbox();
            }
            _ => {}
        },

        // Edit shortcut
        KeyCode::Char('e') => {
            match field_type {
                FieldType::Text | FieldType::Textarea | FieldType::CrateInput => {
                    app.form.start_editing();
                }
                FieldType::ListBuilder | FieldType::CrateSearch => {
                    // Edit the INPUT BUFFER, not the main field
                    app.form.editing = true;
                    app.form.text_scroll_offset = 0;
                    let input_value = app.form.get_list_input_value();
                    app.form.cursor_pos = input_value.len();
                }
                _ => {}
            }
        }

        // Section navigation
        KeyCode::Char('n') => {
            app.form.next_section();
            app.ensure_navbar_visible(5);
        }
        KeyCode::Char('p') => {
            app.form.prev_section();
            app.ensure_navbar_visible(5);
        }
        KeyCode::Left => {
            app.form.prev_section();
            app.ensure_navbar_visible(5);
        }
        KeyCode::Right => {
            app.form.next_section();
            app.ensure_navbar_visible(5);
        }

        // Review screen
        KeyCode::Char('r') => {
            app.current_screen = Screen::Review;
        }

        // Generate shortcut
        KeyCode::Char('g') => {
            app.generate_prompt = true;
            app.should_quit = true;
        }

        // Save draft
        KeyCode::Char('s') => {
            if let Err(e) = app.save_current_draft() {
                app.show_error("Save Error", &format!("Failed to save draft: {}", e));
            }
        }

        // Delete in list
        KeyCode::Delete | KeyCode::Char('d') => {
            if app.form.sub_focus == 2 {
                match field_type {
                    FieldType::ListBuilder => {
                        let items = app.form.get_current_value().as_vec();
                        if !items.is_empty() && app.form.list_selected < items.len() {
                            app.form.remove_list_item(app.form.list_selected);
                            // Adjust list_selected after removal
                            let new_len = items.len().saturating_sub(1);
                            if app.form.list_selected >= new_len && new_len > 0 {
                                app.form.list_selected = new_len - 1;
                            } else if new_len == 0 {
                                app.form.list_selected = 0;
                            }
                        }
                    }
                    FieldType::CrateSearch => {
                        let target_key = field
                            .target_list_key
                            .as_deref()
                            .unwrap_or("tech.additional_crates");
                        let items = app.form.get_target_list(target_key);
                        if !items.is_empty() && app.form.list_selected < items.len() {
                            app.form
                                .remove_from_target_list(target_key, app.form.list_selected);
                            // Adjust list_selected after removal
                            let new_len = items.len().saturating_sub(1);
                            if app.form.list_selected >= new_len && new_len > 0 {
                                app.form.list_selected = new_len - 1;
                            } else if new_len == 0 {
                                app.form.list_selected = 0;
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        // Scrolling
        KeyCode::PageUp => {
            app.form.scroll_up(3);
        }
        KeyCode::PageDown => {
            app.form.scroll_down(3);
        }

        _ => {}
    }

    Ok(())
}

// ─────────────────────────────────────────────
// Crate search trigger
// ─────────────────────────────────────────────

fn trigger_crate_search(app: &mut App, package_name: &str, add_to_list: bool, registry: &str) {
    let package_name = package_name.to_string();
    let registry = registry.to_string();

    app.is_loading = true;
    app.loading_message = format!("Searching {} for '{}'...", registry, package_name);
    app.search_add_to_list = add_to_list;

    // Spawn search in a background thread
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let result =
            package_registry::search_package(&registry, &package_name).map_err(|e| e.to_string());
        let _ = tx.send(result);
    });

    app.search_receiver = Some(rx);
}

// ─────────────────────────────────────────────
// Review screen handler
// ─────────────────────────────────────────────

fn handle_review_event(app: &mut App, event: &Event) -> Result<()> {
    match event {
        Event::Key(key) => {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        app.current_screen = Screen::Form;
                    }
                    KeyCode::Enter | KeyCode::Char('g') | KeyCode::Char('y') => {
                        app.generate_prompt = true;
                        app.should_quit = true;
                    }
                    KeyCode::Char('e') => {
                        app.current_screen = Screen::Form;
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    }
    Ok(())
}

// ─────────────────────────────────────────────
// Success modal handler
// ─────────────────────────────────────────────

fn handle_success_event(app: &mut App, event: &Event) -> Result<()> {
    match event {
        Event::Key(key) => {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Enter | KeyCode::Esc | KeyCode::Char(' ') => {
                        app.current_screen = Screen::Form;
                    }
                    KeyCode::Char('q') => {
                        app.should_quit = true;
                    }
                    KeyCode::Char('m') => {
                        app.current_screen = Screen::Menu;
                    }
                    _ => {}
                }
            }
        }
        Event::Mouse(mouse) => {
            ui::handle_success_mouse(app, mouse);
        }
        _ => {}
    }
    Ok(())
}

fn check_search_completion(app: &mut App) {
    if let Some(receiver) = &app.search_receiver {
        // Check if search is complete (non-blocking)
        match receiver.try_recv() {
            Ok(result) => {
                app.is_loading = false;
                app.search_receiver = None;

                match result {
                    Ok(info) => {
                        let version_str = format!("{} v{}", info.name, info.version);

                        if app.search_add_to_list {
                            let target_key = app
                                .search_target_key
                                .clone()
                                .unwrap_or_else(|| "tech.additional_crates".to_string());
                            app.form.add_to_target_list(&target_key, version_str);
                            app.form.clear_list_input();
                            app.status_message = Some(format!("✓ Added: {}", info.version));
                        } else {
                            app.form.set_current_value(FieldValue::Text(version_str));
                            app.status_message = Some(format!("✓ Found: {}", info.version));
                        }
                    }
                    Err(e) => {
                        app.show_error("Search Failed", &e);
                    }
                }

                app.search_target_key = None;
                app.search_add_to_list = false;
            }
            Err(mpsc::TryRecvError::Empty) => {
                // Search still in progress, keep loading
            }
            Err(mpsc::TryRecvError::Disconnected) => {
                // Thread died unexpectedly
                app.is_loading = false;
                app.search_receiver = None;
                app.show_error("Search Error", "Search thread terminated unexpectedly");
            }
        }
    }
}
