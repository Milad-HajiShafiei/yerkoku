use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::app::get_app_data_dir;
use crate::blueprint::FieldValue;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Draft {
    pub name: String,
    pub blueprint_name: String,
    pub created_at: String,
    pub updated_at: String,
    pub values: BTreeMap<String, FieldValue>,
    pub current_section: usize,
    pub selected_field: usize,
}

impl Draft {
    pub fn new(
        name: String,
        blueprint_name: String,
        values: BTreeMap<String, FieldValue>,
        current_section: usize,
        selected_field: usize,
    ) -> Self {
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        Self {
            name,
            blueprint_name,
            created_at: now.clone(),
            updated_at: now,
            values,
            current_section,
            selected_field,
        }
    }

    pub fn update_timestamp(&mut self) {
        self.updated_at = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    }
}

pub struct DraftManager {
    drafts_dir: PathBuf,
}

impl DraftManager {
    pub fn new() -> Self {
        let drafts_dir = Self::find_drafts_dir();
        Self { drafts_dir }
    }

    fn find_drafts_dir() -> PathBuf {
        // 1. Check environment variable override (useful for development)
        if let Ok(dir) = std::env::var("DRAFTS_DIR") {
            let path = PathBuf::from(&dir);
            if path.exists() || fs::create_dir_all(&path).is_ok() {
                return path;
            }
        }

        // 2. Use OS-specific app data directory
        let mut drafts_dir = get_app_data_dir();
        drafts_dir.push("drafts");

        if !drafts_dir.exists() {
            let _ = fs::create_dir_all(&drafts_dir);
        }

        drafts_dir
    }

    /// List all drafts sorted by most recent
    pub fn list_drafts(&self) -> Vec<(String, Draft)> {
        let mut drafts: Vec<(String, Draft)> = Vec::new();

        if let Ok(entries) = fs::read_dir(&self.drafts_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    match self.load_draft(&path) {
                        Ok(draft) => {
                            let filename = path
                                .file_stem()
                                .and_then(|s| s.to_str())
                                .unwrap_or("unknown")
                                .to_string();
                            drafts.push((filename, draft));
                        }
                        Err(e) => {
                            eprintln!("⚠ Failed to load draft {:?}: {}", path, e);
                        }
                    }
                }
            }
        }

        // Sort by updated_at descending (most recent first)
        drafts.sort_by(|a, b| b.1.updated_at.cmp(&a.1.updated_at));
        drafts
    }

    /// Load a draft from a file
    pub fn load_draft<P: AsRef<Path>>(&self, path: P) -> Result<Draft> {
        let content = fs::read_to_string(path.as_ref())
            .with_context(|| format!("Failed to read draft: {:?}", path.as_ref()))?;
        let draft: Draft = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse draft JSON: {:?}", path.as_ref()))?;
        Ok(draft)
    }

    /// Save a draft to a file (creates new or updates existing)
    pub fn save_draft(&self, filename: &str, draft: &Draft) -> Result<()> {
        let path = self.drafts_dir.join(format!("{}.json", filename));

        // If draft exists, preserve the created_at timestamp
        let mut final_draft = draft.clone();
        if path.exists() {
            if let Ok(existing) = self.load_draft(&path) {
                final_draft.created_at = existing.created_at;
            }
        }
        final_draft.update_timestamp();

        let content =
            serde_json::to_string_pretty(&final_draft).context("Failed to serialize draft")?;
        fs::write(&path, content)
            .with_context(|| format!("Failed to write draft to: {:?}", path))?;
        Ok(())
    }

    /// Delete a draft
    pub fn delete_draft(&self, filename: &str) -> Result<()> {
        let path = self.drafts_dir.join(format!("{}.json", filename));
        if path.exists() {
            fs::remove_file(&path)
                .with_context(|| format!("Failed to delete draft: {:?}", path))?;
        }
        Ok(())
    }

    /// Get the drafts directory path
    pub fn drafts_dir(&self) -> &Path {
        &self.drafts_dir
    }
}
