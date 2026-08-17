use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph},
};

pub struct CrateSearch {
    pub label: String,
    pub input_value: String,
    pub added_crates: Vec<String>,
    pub placeholder: String,
    pub cursor_pos: usize,
    pub focused: bool,
    pub input_editing: bool,
    pub button_focused: bool,
    pub list_focused: bool,
    pub list_selected: usize,
    pub loading: bool,
}

impl CrateSearch {
    pub fn new(label: &str, added_crates: Vec<String>, placeholder: &str) -> Self {
        Self {
            label: label.to_string(),
            input_value: String::new(),
            added_crates,
            placeholder: placeholder.to_string(),
            cursor_pos: 0,
            focused: false,
            input_editing: false,
            button_focused: false,
            list_focused: false,
            list_selected: 0,
            loading: false,
        }
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
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
        let has_items = !self.added_crates.is_empty();
        let list_height = if has_items {
            Constraint::Min(2)
        } else {
            Constraint::Length(1)
        };

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), list_height])
            .split(inner);

        // ── Top: Input + Search Button side by side ──
        let top_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(10), Constraint::Length(16)])
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

        // ── Search & Add button ──
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
            let (button_text, button_style) = if self.loading {
                (
                    "⏳ Searching",
                    Style::default().fg(Color::Rgb(255, 200, 80)),
                )
            } else if self.button_focused {
                (
                    "Search & Add",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                (
                    "Search & Add",
                    Style::default().fg(Color::Rgb(150, 150, 170)),
                )
            };

            let button_para = Paragraph::new(button_text)
                .style(button_style)
                .alignment(Alignment::Center);
            f.render_widget(button_para, button_inner);
        }

        // ── Bottom: List of added crates ──
        if !has_items {
            let empty_text = Paragraph::new("  No packages added yet")
                .style(Style::default().fg(Color::Rgb(100, 100, 130)));
            f.render_widget(empty_text, chunks[1]);
            return;
        }

        let list_border_color = if self.focused && self.list_focused {
            Color::Cyan
        } else {
            Color::Rgb(80, 80, 120)
        };

        let list_title = if self.list_focused {
            " 📦 Added (↑/↓ select, d delete) "
        } else {
            &format!(" 📦 Added ({}) ", self.added_crates.len())
        };

        let list_block = Block::default()
            .title(list_title.to_string())
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(list_border_color));

        let items: Vec<ListItem> = self
            .added_crates
            .iter()
            .enumerate()
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
        if self.list_focused && !self.added_crates.is_empty() {
            state.select(Some(self.list_selected));
        }

        let list = List::new(items).block(list_block).highlight_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        );

        f.render_stateful_widget(list, chunks[1], &mut state);
    }
}
