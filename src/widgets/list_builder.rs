use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph},
};

pub struct ListBuilder {
    pub label: String,
    pub input_value: String,
    pub items: Vec<String>,
    pub placeholder: String,
    pub cursor_pos: usize,
    pub focused: bool,
    pub input_editing: bool,
    pub button_focused: bool,
    pub list_focused: bool,
    pub list_selected: usize,
}

impl ListBuilder {
    pub fn new(label: &str, items: Vec<String>, placeholder: &str) -> Self {
        Self {
            label: label.to_string(),
            input_value: String::new(),
            items,
            placeholder: placeholder.to_string(),
            cursor_pos: 0,
            focused: false,
            input_editing: false,
            button_focused: false,
            list_focused: false,
            list_selected: 0,
        }
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        if area.height < 6 || area.width < 20 {
            return;
        }

        // Outer block
        let border_color = if self.focused {
            Color::Cyan
        } else {
            Color::Rgb(80, 80, 120)
        };

        let outer_block = Block::default()
            .title(format!(" {} ({}) ", self.label, self.items.len()))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(border_color));

        let inner = outer_block.inner(area);
        f.render_widget(outer_block, area);

        if inner.height < 4 || inner.width < 16 {
            return;
        }

        // Split: input row (3 rows) + list (rest)
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(2)])
            .split(inner);

        // ── Top: Input + Add button ──
        let top_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(10), Constraint::Length(10)])
            .split(chunks[0]);

        // Input
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

        // Add button
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
            let button_style = if self.focused && self.button_focused {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Rgb(150, 150, 170))
            };

            let button_para = Paragraph::new("➕ Add")
                .style(button_style)
                .alignment(ratatui::layout::Alignment::Center);
            f.render_widget(button_para, button_inner);
        }

        // ── Bottom: List of items ──
        let list_border_color = if self.focused && self.list_focused {
            Color::Cyan
        } else {
            Color::Rgb(80, 80, 120)
        };

        let list_block = Block::default()
            .title(" Items ")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(list_border_color));

        if self.items.is_empty() {
            let empty = Paragraph::new("  No items yet. Type above and press Add.")
                .style(Style::default().fg(Color::Rgb(100, 100, 130)))
                .block(list_block);
            f.render_widget(empty, chunks[1]);
            return;
        }

        let items: Vec<ListItem> = self
            .items
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let is_selected = i == self.list_selected && self.list_focused;
                let style = if is_selected {
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };
                ListItem::new(Line::from(vec![
                    Span::styled(
                        format!("  {}. ", i + 1),
                        Style::default().fg(Color::Rgb(150, 150, 170)),
                    ),
                    Span::styled(item.as_str(), style),
                ]))
            })
            .collect();

        let mut state = ListState::default();
        if self.list_focused {
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
