use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders, Paragraph},
};

pub struct CrateInput {
    pub label: String,
    // pub crate_name: String,
    pub value: String,
    pub placeholder: String,
    pub cursor_pos: usize,
    pub focused: bool,
    pub editing: bool,
    pub button_focused: bool,
    pub loading: bool,
}

impl CrateInput {
    pub fn new(label: &str, _crate_name: &str, value: &str, placeholder: &str) -> Self {
        Self {
            label: label.to_string(),
            // crate_name: crate_name.to_string(),
            value: value.to_string(),
            placeholder: placeholder.to_string(),
            cursor_pos: value.len(),
            focused: false,
            editing: false,
            button_focused: false,
            loading: false,
        }
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        if area.height < 3 || area.width < 20 {
            return;
        }

        // Outer block for the whole widget
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

        if inner.height == 0 || inner.width < 16 {
            return;
        }

        // Split into: input (flex) + button (fixed 12 cols)
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(10), Constraint::Length(14)])
            .split(inner);

        // ── Text input on the left ──
        let input_border_color = if self.focused && self.editing {
            Color::Rgb(100, 220, 130) // Green when editing
        } else if self.focused && !self.button_focused {
            Color::Cyan
        } else {
            Color::Rgb(80, 80, 120)
        };

        let input_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(input_border_color));

        let input_inner = input_block.inner(chunks[0]);
        f.render_widget(input_block, chunks[0]);

        if input_inner.height > 0 && input_inner.width > 0 {
            let text = if self.value.is_empty() && !self.editing {
                Paragraph::new(self.placeholder.as_str()).style(
                    Style::default()
                        .fg(Color::Rgb(100, 100, 130))
                        .add_modifier(Modifier::ITALIC),
                )
            } else if self.editing {
                let cursor = self.cursor_pos.min(self.value.len());
                let before = &self.value[..cursor];
                let after = &self.value[cursor..];
                let display = format!("{}█{}", before, after);
                Paragraph::new(display).style(Style::default().fg(Color::White))
            } else {
                Paragraph::new(self.value.as_str()).style(Style::default().fg(Color::White))
            };
            f.render_widget(text, input_inner);
        }

        // ── Button on the right ──
        let button_border_color = if self.focused && self.button_focused {
            Color::Yellow
        } else {
            Color::Rgb(80, 80, 120)
        };

        let button_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(button_border_color));

        let button_inner = button_block.inner(chunks[1]);
        f.render_widget(button_block, chunks[1]);

        if button_inner.height > 0 && button_inner.width > 0 {
            let button_text = if self.loading {
                "⏳ Loading"
            } else {
                "Get Latest"
            };

            let button_style = if self.focused && self.button_focused {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Rgb(150, 150, 170))
            };

            let button_para = Paragraph::new(button_text)
                .style(button_style)
                .alignment(ratatui::layout::Alignment::Center);
            f.render_widget(button_para, button_inner);
        }
    }
}
