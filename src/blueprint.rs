use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Blueprint {
    pub name: String,
    pub description: String,
    pub icon: String,
    #[serde(default)]
    pub sections: Vec<Section>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section {
    pub title: String,
    pub icon: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub fields: Vec<Field>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Field {
    pub key: String,
    pub label: String,
    #[serde(rename = "type")]
    pub field_type: FieldType,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub placeholder: String,
    #[serde(default)]
    pub default: FieldValue,
    #[serde(default)]
    pub options: Vec<SelectOption>,
    #[serde(default)]
    pub min: Option<f64>,
    #[serde(default)]
    pub max: Option<f64>,
    #[serde(default)]
    pub rows: Option<u16>,
    #[serde(default)]
    pub search_crate: Option<String>,
    #[serde(default)]
    pub crate_name: Option<String>,
    #[serde(default)]
    pub target_list_key: Option<String>,
    #[serde(default)]
    pub hidden: bool,
    pub button_text: Option<String>,
    #[serde(default)]
    pub action: Option<String>,
    #[serde(default = "default_registry")]
    pub registry: String,
}

fn default_registry() -> String {
    "crates.io".to_string()
}

// ✅ FIXED: Removed #[serde(tag = "type")] — it was causing deserialization to fail
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FieldType {
    #[serde(rename = "text")]
    Text,
    #[serde(rename = "textarea")]
    Textarea,
    #[serde(rename = "number")]
    Number,
    #[serde(rename = "checkbox")]
    Checkbox,
    #[serde(rename = "select")]
    Select,
    #[serde(rename = "multiselect")]
    Multiselect,
    #[serde(rename = "search_crate")]
    SearchCrate,
    #[serde(rename = "crate_input")]
    CrateInput,
    #[serde(rename = "list_builder")]
    ListBuilder,
    #[serde(rename = "section_break")]
    SectionBreak,
    #[serde(rename = "crate_search")]
    CrateSearch,
    #[serde(rename = "action_button")]
    ActionButton,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SelectOption {
    pub value: String,
    pub label: String,
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(untagged)]
pub enum FieldValue {
    #[default]
    Empty,
    Text(String),
    Number(f64),
    Bool(bool),
    Array(Vec<String>),
}

impl FieldValue {
    pub fn as_str(&self) -> &str {
        match self {
            FieldValue::Text(s) => s.as_str(),
            _ => "",
        }
    }

    pub fn as_bool(&self) -> bool {
        match self {
            FieldValue::Bool(b) => *b,
            _ => false,
        }
    }

    pub fn as_vec(&self) -> Vec<String> {
        match self {
            FieldValue::Array(v) => v.clone(),
            _ => vec![],
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            FieldValue::Empty => true,
            FieldValue::Text(s) => s.trim().is_empty(),
            FieldValue::Bool(b) => !b,
            FieldValue::Number(n) => *n == 0.0,
            FieldValue::Array(v) => v.is_empty(),
        }
    }
}

impl Blueprint {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path.as_ref())
            .with_context(|| format!("Failed to read blueprint: {:?}", path.as_ref()))?;
        let blueprint: Blueprint = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse blueprint JSON: {:?}", path.as_ref()))?;
        Ok(blueprint)
    }
}

// ✅ FIXED: Added error reporting instead of silent failures
pub fn list_blueprints<P: AsRef<Path>>(dir: P) -> Vec<Blueprint> {
    let mut blueprints = Vec::new();
    let dir_path = dir.as_ref();

    // Check if directory exists
    if !dir_path.exists() {
        eprintln!("⚠ Blueprints directory not found: {:?}", dir_path);
        eprintln!(
            "  Current working directory: {:?}",
            std::env::current_dir().unwrap_or_default()
        );
        eprintln!("  Create a 'blueprints/' folder with .json files.");
        return blueprints;
    }

    match fs::read_dir(dir_path) {
        Ok(entries) => {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    match Blueprint::load(&path) {
                        Ok(blueprint) => {
                            blueprints.push(blueprint);
                        }
                        Err(e) => {
                            eprintln!(
                                "⚠ Failed to load blueprint {:?}: {}",
                                path.file_name().unwrap_or_default(),
                                e
                            );
                        }
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("⚠ Cannot read directory {:?}: {}", dir_path, e);
        }
    }

    blueprints.sort_by(|a, b| a.name.cmp(&b.name));

    blueprints
}
