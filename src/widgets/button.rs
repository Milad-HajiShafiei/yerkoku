use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders, Paragraph},
};

pub struct Button {
    pub label: String,
    pub button_text: String,
    pub focused: bool,
}

impl Button {
    pub fn new(label: &str, button_text: &str) -> Self {
        Self {
            label: label.to_string(),
            button_text: button_text.to_string(),
            focused: false,
        }
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        if area.height < 3 || area.width < 10 {
            return;
        }

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

        if inner.height == 0 || inner.width < 8 {
            return;
        }

        // Inner button block with emphasis styling
        let button_bg = if self.focused {
            Color::Rgb(40, 80, 100)
        } else {
            Color::Rgb(30, 30, 45)
        };

        let button_border = if self.focused {
            Color::Rgb(100, 220, 130) // Green when focused
        } else {
            Color::Rgb(100, 100, 130)
        };

        let button_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Thick)
            .border_style(Style::default().fg(button_border))
            .style(Style::default().bg(button_bg));

        let button_inner = button_block.inner(inner);
        f.render_widget(button_block, inner);

        if button_inner.height == 0 || button_inner.width == 0 {
            return;
        }

        let text_style = if self.focused {
            Style::default()
                .fg(Color::Rgb(100, 255, 150))
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Rgb(200, 200, 220))
        };

        let button_text = Paragraph::new(self.button_text.as_str())
            .style(text_style)
            .alignment(Alignment::Center);

        f.render_widget(button_text, button_inner);
    }
}
