use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph},
};

pub struct ListBuilder {
    pub label: String,
    pub items: Vec<String>,
    pub input_value: String,
    pub placeholder: String,
    pub cursor_pos: usize,
    pub focused: bool,
    pub input_editing: bool,
    pub button_focused: bool,
    pub list_focused: bool,
    pub list_selected: usize,
    pub list_scroll_offset: usize,
}

impl ListBuilder {
    pub fn new(label: &str, items: Vec<String>, placeholder: &str) -> Self {
        Self {
            label: label.to_string(),
            items,
            input_value: String::new(),
            placeholder: placeholder.to_string(),
            cursor_pos: 0,
            focused: false,
            input_editing: false,
            button_focused: false,
            list_focused: false,
            list_selected: 0,
            list_scroll_offset: 0,
        }
    }

    /// Ensure the selected item is visible by adjusting scroll offset
    fn ensure_selected_visible(&mut self, visible_count: usize) {
        if visible_count == 0 || self.items.is_empty() {
            return;
        }
        if self.list_selected < self.list_scroll_offset {
            self.list_scroll_offset = self.list_selected;
        } else if self.list_selected >= self.list_scroll_offset + visible_count {
            self.list_scroll_offset = self.list_selected.saturating_sub(visible_count - 1);
        }
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        if area.height < 3 || area.width < 20 {
            return;
        }

        // ── Outer block ──
        let border_color = if self.focused {
            Color::Cyan
        } else {
            Color::Rgb(80, 80, 120)
        };

        let outer_block = Block::default()
            .title(format!(" {} ", self.label))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(border_color));

        let inner = outer_block.inner(area);
        f.render_widget(outer_block, area);

        if inner.height < 3 || inner.width < 16 {
            return;
        }

        // ── Split: input row (3 rows) + list (rest) ──
        let has_items = !self.items.is_empty();
        let list_constraint = if has_items {
            Constraint::Min(2)
        } else {
            Constraint::Length(1)
        };

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), list_constraint])
            .split(inner);

        // ── Top: Input + Add Button side by side ──
        let top_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(10), Constraint::Length(12)])
            .split(chunks[0]);

        // ── Input field ──
        let input_border_color = if self.focused && self.input_editing {
            Color::Rgb(100, 220, 130)
        } else if self.focused && !self.button_focused && !self.list_focused {
            Color::Cyan
        } else {
            Color::Rgb(80, 80, 120)
        };

        let input_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(input_border_color));

        let input_inner = input_block.inner(top_chunks[0]);
        f.render_widget(input_block, top_chunks[0]);

        if input_inner.height > 0 && input_inner.width > 0 {
            let text = if self.input_value.is_empty() && !self.input_editing {
                Paragraph::new(self.placeholder.as_str()).style(
                    Style::default()
                        .fg(Color::Rgb(100, 100, 130))
                        .add_modifier(Modifier::ITALIC),
                )
            } else if self.input_editing {
                let cursor = self.cursor_pos.min(self.input_value.len());
                let before = &self.input_value[..cursor];
                let after = &self.input_value[cursor..];
                let display = format!("{}█{}", before, after);
                Paragraph::new(display).style(Style::default().fg(Color::White))
            } else {
                Paragraph::new(self.input_value.as_str()).style(Style::default().fg(Color::White))
            };
            f.render_widget(text, input_inner);
        }

        // ── Add button ──
        let button_border_color = if self.focused && self.button_focused {
            Color::Yellow
        } else {
            Color::Rgb(80, 80, 120)
        };

        let button_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(button_border_color));

        let button_inner = button_block.inner(top_chunks[1]);
        f.render_widget(button_block, top_chunks[1]);

        if button_inner.height > 0 && button_inner.width > 0 {
            let (button_text, button_style) = if self.button_focused {
                (
                    "+ Add",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                ("+ Add", Style::default().fg(Color::Rgb(150, 150, 170)))
            };

            let button_para = Paragraph::new(button_text)
                .style(button_style)
                .alignment(Alignment::Center);
            f.render_widget(button_para, button_inner);
        }

        // ── Bottom: List of items ──
        if !has_items {
            let empty_text = Paragraph::new("  No items added yet")
                .style(Style::default().fg(Color::Rgb(100, 100, 130)));
            f.render_widget(empty_text, chunks[1]);
            return;
        }

        // Calculate how many items fit in the list area
        let list_area_height = chunks[1].height.saturating_sub(2) as usize;
        let visible_count = list_area_height.max(1);

        // Ensure selected item is visible
        self.ensure_selected_visible(visible_count);

        let list_border_color = if self.focused && self.list_focused {
            Color::Cyan
        } else {
            Color::Rgb(80, 80, 120)
        };

        let list_title = if self.list_focused {
            format!(
                " 📋 Items ({}) — ↑/↓ select, d delete, Tab back ",
                self.items.len()
            )
        } else {
            format!(" 📋 Items ({}) ", self.items.len())
        };

        let list_block = Block::default()
            .title(list_title)
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(list_border_color));

        // Build visible items starting from scroll offset
        let items: Vec<ListItem> = self
            .items
            .iter()
            .enumerate()
            .skip(self.list_scroll_offset)
            .take(visible_count)
            .map(|(i, item)| {
                let is_selected = i == self.list_selected && self.list_focused;
                let (prefix, style) = if is_selected {
                    (
                        "▶ ",
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    )
                } else {
                    ("  ", Style::default().fg(Color::White))
                };
                ListItem::new(Line::from(vec![
                    Span::styled(prefix, style),
                    Span::styled(
                        format!("{}. ", i + 1),
                        Style::default().fg(Color::Rgb(150, 150, 170)),
                    ),
                    Span::styled(item.as_str(), style),
                ]))
            })
            .collect();

        let mut state = ListState::default();
        if self.list_focused && !self.items.is_empty() {
            // Select index relative to the visible window
            let relative_index = self.list_selected.saturating_sub(self.list_scroll_offset);
            state.select(Some(relative_index.min(visible_count.saturating_sub(1))));
        }

        let list = List::new(items).block(list_block).highlight_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        );

        f.render_stateful_widget(list, chunks[1], &mut state);
    }
}
