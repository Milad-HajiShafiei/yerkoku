use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
};

pub struct TextInput {
    pub label: String,
    pub value: String,
    pub placeholder: String,
    pub cursor_pos: usize,
    pub focused: bool,
    pub editing: bool,
    pub multiline: bool,
    pub scroll_offset: u16,
}

impl TextInput {
    pub fn new(label: &str, value: &str, placeholder: &str) -> Self {
        Self {
            label: label.to_string(),
            value: value.to_string(),
            placeholder: placeholder.to_string(),
            cursor_pos: value.len(),
            focused: false,
            editing: false,
            multiline: false,
            scroll_offset: 0,
        }
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        if area.height < 2 || area.width < 4 {
            return;
        }

        let border_color = if self.focused {
            Color::Cyan
        } else {
            Color::Rgb(80, 80, 120)
        };

        let block = Block::default()
            .title(format!(" {} ", self.label))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(border_color));

        let inner = block.inner(area);
        f.render_widget(block, area);

        if inner.height == 0 || inner.width == 0 {
            return;
        }

        let text = if self.value.is_empty() && !self.editing {
            let mut para = Paragraph::new(self.placeholder.as_str()).style(
                Style::default()
                    .fg(Color::Rgb(100, 100, 130))
                    .add_modifier(Modifier::ITALIC),
            );
            if self.multiline {
                para = para.wrap(Wrap { trim: false });
            }
            para
        } else if self.editing {
            let cursor = self.cursor_pos.min(self.value.len());
            let before = &self.value[..cursor];
            let after = &self.value[cursor..];
            let display = format!("{}█{}", before, after);
            let mut para = Paragraph::new(display).style(Style::default().fg(Color::White));
            if self.multiline {
                para = para.wrap(Wrap { trim: false });
            }
            para
        } else {
            let mut para =
                Paragraph::new(self.value.as_str()).style(Style::default().fg(Color::White));
            if self.multiline {
                para = para.wrap(Wrap { trim: false });
            }
            para
        };

        // Apply scroll offset for multiline text
        if self.multiline && self.scroll_offset > 0 {
            let text = text.scroll((self.scroll_offset, 0));
            f.render_widget(text, inner);
        } else {
            f.render_widget(text, inner);
        }
    }
}
