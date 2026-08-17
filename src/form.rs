use crate::blueprint::{Blueprint, FieldType, FieldValue, SelectOption};
use std::collections::{BTreeMap, HashMap};

pub struct Form {
    pub values: BTreeMap<String, FieldValue>,
    pub section_fields: Vec<Vec<String>>,
    pub field_types: HashMap<String, FieldType>,
    pub selected_field: usize,
    pub editing: bool,
    pub cursor_pos: usize,
    pub scroll_offset: usize,
    pub current_section: usize,
    pub total_sections: usize,
    pub sub_focus: usize,
    pub list_selected: usize,
    pub visible_field_count: usize,
    pub text_scroll_offset: u16,
}

impl Form {
    pub fn new(blueprint: &Blueprint) -> Self {
        let mut values = BTreeMap::new();
        let mut section_fields: Vec<Vec<String>> = Vec::new();
        let mut field_types: HashMap<String, FieldType> = HashMap::new();

        for section in &blueprint.sections {
            let mut keys = Vec::new();
            for field in &section.fields {
                if !matches!(field.field_type, FieldType::SectionBreak) && !field.hidden {
                    let key = field.key.clone();
                    values.insert(key.clone(), field.default.clone());
                    field_types.insert(key.clone(), field.field_type.clone());
                    keys.push(key);
                }
            }
            section_fields.push(keys);
        }

        Self {
            values,
            section_fields,
            field_types,
            selected_field: 0,
            editing: false,
            cursor_pos: 0,
            scroll_offset: 0,
            current_section: 0,
            total_sections: blueprint.sections.len(),
            sub_focus: 0,
            list_selected: 0,
            visible_field_count: 5,
            text_scroll_offset: 0,
        }
    }

    // ─────────────────────────────────────────────
    // Field access helpers
    // ─────────────────────────────────────────────

    fn get_current_key(&self) -> Option<&String> {
        self.section_fields
            .get(self.current_section)
            .and_then(|fields| fields.get(self.selected_field))
    }

    pub fn current_section_field_count(&self) -> usize {
        self.section_fields
            .get(self.current_section)
            .map(|f| f.len())
            .unwrap_or(0)
    }

    #[allow(dead_code)]
    pub fn is_multiline(&self) -> bool {
        if let Some(key) = self.get_current_key() {
            if let Some(field_type) = self.field_types.get(key) {
                return matches!(field_type, FieldType::Textarea);
            }
        }
        false
    }

    pub fn set_visible_field_count(&mut self, count: usize) {
        self.visible_field_count = count.max(1);
    }

    // ─────────────────────────────────────────────
    // Navigation
    // ─────────────────────────────────────────────

    pub fn next_field(&mut self) {
        let count = self.current_section_field_count();
        if count == 0 {
            return;
        }

        // Clamp current position
        if self.selected_field >= count {
            self.selected_field = count - 1;
        }

        // Move to next field with wrap
        self.selected_field = (self.selected_field + 1) % count;

        // Reset editing state
        self.editing = false;
        self.cursor_pos = 0;
        self.sub_focus = 0;
        self.text_scroll_offset = 0;
        self.list_selected = 0;

        self.ensure_visible();
    }

    pub fn prev_field(&mut self) {
        let count = self.current_section_field_count();
        if count == 0 {
            return;
        }

        // Clamp current position
        if self.selected_field >= count {
            self.selected_field = count - 1;
        }

        // Move to previous field with wrap
        if self.selected_field == 0 {
            self.selected_field = count - 1;
        } else {
            self.selected_field -= 1;
        }

        // Reset editing state
        self.editing = false;
        self.cursor_pos = 0;
        self.sub_focus = 0;
        self.text_scroll_offset = 0;
        self.list_selected = 0;

        self.ensure_visible();
    }

    pub fn next_section(&mut self) {
        if self.current_section < self.total_sections - 1 {
            self.current_section += 1;
        } else {
            self.current_section = 0;
        }
        self.selected_field = 0;
        self.scroll_offset = 0;
        self.editing = false;
        self.cursor_pos = 0;
        self.sub_focus = 0;
        self.text_scroll_offset = 0;
        self.list_selected = 0;
    }

    pub fn prev_section(&mut self) {
        if self.current_section > 0 {
            self.current_section -= 1;
        } else {
            self.current_section = self.total_sections.saturating_sub(1);
        }
        self.selected_field = 0;
        self.scroll_offset = 0;
        self.editing = false;
        self.cursor_pos = 0;
        self.sub_focus = 0;
        self.text_scroll_offset = 0;
        self.list_selected = 0;
    }

    #[allow(dead_code)]
    pub fn go_to_section(&mut self, idx: usize) {
        if idx < self.total_sections {
            self.current_section = idx;
            self.selected_field = 0;
            self.scroll_offset = 0;
            self.editing = false;
            self.cursor_pos = 0;
            self.sub_focus = 0;
            self.text_scroll_offset = 0;
            self.list_selected = 0;
        }
    }

    // ─────────────────────────────────────────────
    // Scroll management
    // ─────────────────────────────────────────────

    fn ensure_visible(&mut self) {
        let count = self.current_section_field_count();
        if count == 0 {
            return;
        }

        // Clamp selected field
        if self.selected_field >= count {
            self.selected_field = count - 1;
        }

        let visible = self.visible_field_count.max(1);

        // If selected field is above visible area, scroll up
        if self.selected_field < self.scroll_offset {
            self.scroll_offset = self.selected_field;
        }
        // If selected field is below visible area, scroll down
        else if self.selected_field >= self.scroll_offset + visible {
            self.scroll_offset = self.selected_field.saturating_sub(visible - 1);
        }

        // Clamp scroll offset
        let max_scroll = count.saturating_sub(visible);
        if self.scroll_offset > max_scroll {
            self.scroll_offset = max_scroll;
        }
    }

    pub fn scroll_up(&mut self, amount: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(amount);
    }

    pub fn scroll_down(&mut self, amount: usize) {
        self.scroll_offset += amount;
        let count = self.current_section_field_count();
        let max_scroll = count.saturating_sub(self.visible_field_count);
        if self.scroll_offset > max_scroll {
            self.scroll_offset = max_scroll;
        }
    }

    pub fn scroll_text_up(&mut self, amount: u16) {
        self.text_scroll_offset = self.text_scroll_offset.saturating_sub(amount);
    }

    pub fn scroll_text_down(&mut self, amount: u16) {
        self.text_scroll_offset = self.text_scroll_offset.saturating_add(amount);
    }

    #[allow(dead_code)]
    pub fn reset_text_scroll(&mut self) {
        self.text_scroll_offset = 0;
    }

    // ─────────────────────────────────────────────
    // Editing
    // ─────────────────────────────────────────────

    pub fn start_editing(&mut self) {
        self.editing = true;
        self.text_scroll_offset = 0;
        let mut new_pos = 0;
        if let Some(key) = self.get_current_key() {
            if let Some(FieldValue::Text(s)) = self.values.get(key) {
                new_pos = s.len();
            }
        }
        self.cursor_pos = new_pos;
    }

    pub fn stop_editing(&mut self) {
        self.editing = false;
        self.cursor_pos = 0;
        self.text_scroll_offset = 0;
    }

    pub fn is_editing(&self) -> bool {
        self.editing
    }

    // ─────────────────────────────────────────────
    // Text manipulation
    // ─────────────────────────────────────────────

    pub fn insert_char(&mut self, c: char) {
        let pos = self.cursor_pos;
        let mut updated_pos = None;

        if let Some(key) = self.get_current_key().cloned() {
            if let Some(FieldValue::Text(s)) = self.values.get_mut(&key) {
                if pos <= s.len() {
                    s.insert(pos, c);
                    updated_pos = Some(pos + c.len_utf8());
                }
            } else {
                // If field is empty or not text, create new string
                let mut s = String::new();
                s.insert(pos, c);
                self.values.insert(key, FieldValue::Text(s));
                updated_pos = Some(pos + c.len_utf8());
            }
        }

        if let Some(new_pos) = updated_pos {
            self.cursor_pos = new_pos;
        }
    }

    pub fn backspace(&mut self) {
        let pos = self.cursor_pos;
        let mut updated_pos = None;

        if let Some(key) = self.get_current_key().cloned() {
            if let Some(FieldValue::Text(s)) = self.values.get_mut(&key) {
                if pos > 0 {
                    let prev = Self::prev_char_boundary(s.as_str(), pos);
                    s.drain(prev..pos);
                    updated_pos = Some(prev);
                }
            }
        }

        if let Some(new_pos) = updated_pos {
            self.cursor_pos = new_pos;
        }
    }

    pub fn delete(&mut self) {
        let pos = self.cursor_pos;

        if let Some(key) = self.get_current_key().cloned() {
            if let Some(FieldValue::Text(s)) = self.values.get_mut(&key) {
                if pos < s.len() {
                    let next = Self::next_char_boundary(s.as_str(), pos);
                    s.drain(pos..next);
                }
            }
        }
    }

    pub fn cursor_left(&mut self) {
        let pos = self.cursor_pos;
        let mut new_pos = pos;

        if let Some(key) = self.get_current_key() {
            if let Some(FieldValue::Text(s)) = self.values.get(key) {
                new_pos = Self::prev_char_boundary(s.as_str(), pos);
            }
        }

        self.cursor_pos = new_pos;
    }

    pub fn cursor_right(&mut self) {
        let pos = self.cursor_pos;
        let mut new_pos = pos;

        if let Some(key) = self.get_current_key() {
            if let Some(FieldValue::Text(s)) = self.values.get(key) {
                new_pos = Self::next_char_boundary(s.as_str(), pos);
            }
        }

        self.cursor_pos = new_pos;
    }

    pub fn cursor_start(&mut self) {
        self.cursor_pos = 0;
    }

    pub fn cursor_end(&mut self) {
        let mut new_pos = 0;
        if let Some(key) = self.get_current_key() {
            if let Some(FieldValue::Text(s)) = self.values.get(key) {
                new_pos = s.len();
            }
        }
        self.cursor_pos = new_pos;
    }

    // ─────────────────────────────────────────────
    // Checkbox
    // ─────────────────────────────────────────────

    pub fn toggle_checkbox(&mut self) {
        if let Some(key) = self.get_current_key().cloned() {
            if let Some(val) = self.values.get_mut(&key) {
                match val {
                    FieldValue::Bool(b) => *b = !*b,
                    FieldValue::Empty => *val = FieldValue::Bool(true),
                    _ => {}
                }
            }
        }
    }

    // ─────────────────────────────────────────────
    // Select
    // ─────────────────────────────────────────────

    pub fn cycle_select(&mut self, options: &[SelectOption]) {
        if options.is_empty() {
            return;
        }

        if let Some(key) = self.get_current_key().cloned() {
            let current_value = self.values.get(&key).cloned().unwrap_or_default();
            let current_str = current_value.as_str();

            let next_index = options
                .iter()
                .position(|opt| opt.value == current_str)
                .map(|i| (i + 1) % options.len())
                .unwrap_or(0);

            let next_value = options[next_index].value.clone();
            self.values.insert(key, FieldValue::Text(next_value));
        }
    }

    // ─────────────────────────────────────────────
    // Value access
    // ─────────────────────────────────────────────

    pub fn set_current_value(&mut self, value: FieldValue) {
        if let Some(key) = self.get_current_key().cloned() {
            self.values.insert(key, value);
        }
    }

    pub fn get_current_value(&self) -> FieldValue {
        if let Some(key) = self.get_current_key() {
            self.values.get(key).cloned().unwrap_or_default()
        } else {
            FieldValue::Empty
        }
    }

    // ─────────────────────────────────────────────
    // List builder methods
    // ─────────────────────────────────────────────

    pub fn add_list_item(&mut self, item: String) {
        if item.is_empty() {
            return;
        }
        if let Some(key) = self.get_current_key().cloned() {
            let existing = self.values.get(&key).cloned().unwrap_or_default();
            let mut items = existing.as_vec();
            if !items.contains(&item) {
                items.push(item);
                self.values.insert(key, FieldValue::Array(items));
            }
        }
    }

    pub fn remove_list_item(&mut self, index: usize) {
        if let Some(key) = self.get_current_key().cloned() {
            let existing = self.values.get(&key).cloned().unwrap_or_default();
            let mut items = existing.as_vec();
            if index < items.len() {
                items.remove(index);
                self.values.insert(key, FieldValue::Array(items));
            }
        }
    }

    pub fn get_list_input_value(&self) -> String {
        if let Some(key) = self.get_current_key() {
            let input_key = format!("{}_input", key);
            self.values
                .get(&input_key)
                .map(|v| v.as_str().to_string())
                .unwrap_or_default()
        } else {
            String::new()
        }
    }

    pub fn set_list_input_value(&mut self, value: String) {
        if let Some(key) = self.get_current_key().cloned() {
            let input_key = format!("{}_input", key);
            self.values.insert(input_key, FieldValue::Text(value));
        }
    }

    pub fn insert_list_input_char(&mut self, c: char) {
        let mut value = self.get_list_input_value();
        let pos = self.cursor_pos.min(value.len());
        value.insert(pos, c);
        self.cursor_pos = pos + c.len_utf8();
        self.set_list_input_value(value);
    }

    pub fn list_input_backspace(&mut self) {
        let mut value = self.get_list_input_value();
        if self.cursor_pos > 0 {
            let prev = Self::prev_char_boundary(&value, self.cursor_pos);
            value.drain(prev..self.cursor_pos);
            self.cursor_pos = prev;
            self.set_list_input_value(value);
        }
    }

    pub fn clear_list_input(&mut self) {
        self.set_list_input_value(String::new());
        self.cursor_pos = 0;
    }

    // ─────────────────────────────────────────────
    // Target list methods (for crate_search)
    // ─────────────────────────────────────────────

    // ─────────────────────────────────────────────
    // Target list methods (for crate_search)
    // ─────────────────────────────────────────────

    pub fn get_target_list(&self, target_key: &str) -> Vec<String> {
        self.values
            .get(target_key)
            .map(|v| v.as_vec())
            .unwrap_or_default()
    }

    pub fn add_to_target_list(&mut self, target_key: &str, item: String) {
        if item.is_empty() {
            return;
        }
        let existing = self.values.get(target_key).cloned().unwrap_or_default();
        let mut items = existing.as_vec();
        if !items.contains(&item) {
            items.push(item);
            self.values
                .insert(target_key.to_string(), FieldValue::Array(items));
        }
    }

    pub fn remove_from_target_list(&mut self, target_key: &str, index: usize) {
        let existing = self.values.get(target_key).cloned().unwrap_or_default();
        let mut items = existing.as_vec();
        if index < items.len() {
            items.remove(index);
            self.values
                .insert(target_key.to_string(), FieldValue::Array(items));
        }
    }

    #[allow(dead_code)]
    pub fn clear_target_list(&mut self, target_key: &str) {
        self.values
            .insert(target_key.to_string(), FieldValue::Array(Vec::new()));
    }

    // ─────────────────────────────────────────────
    // Character boundary helpers
    // ─────────────────────────────────────────────

    fn prev_char_boundary(s: &str, pos: usize) -> usize {
        let mut p = pos;
        while p > 0 {
            p -= 1;
            if s.is_char_boundary(p) {
                return p;
            }
        }
        0
    }

    fn next_char_boundary(s: &str, pos: usize) -> usize {
        let mut p = pos;
        while p < s.len() {
            p += 1;
            if s.is_char_boundary(p) {
                return p;
            }
        }
        s.len()
    }
}
