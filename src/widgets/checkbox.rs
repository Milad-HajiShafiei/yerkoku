use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
};

pub struct Checkbox {
    pub label: String,
    pub checked: bool,
    pub focused: bool,
}

impl Checkbox {
    pub fn new(label: &str, checked: bool) -> Self {
        Self {
            label: label.to_string(),
            checked,
            focused: false,
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
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(border_color));

        let inner = block.inner(area);
        f.render_widget(block, area);

        if inner.height == 0 || inner.width == 0 {
            return;
        }

        let (checkmark, check_color) = if self.checked {
            ("☑", Color::Green)
        } else {
            ("☐", Color::Rgb(100, 100, 130))
        };

        let label_style = if self.focused {
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };

        let text = Line::from(vec![
            Span::styled(format!(" {} ", checkmark), Style::default().fg(check_color)),
            Span::styled(&self.label, label_style),
        ]);

        f.render_widget(Paragraph::new(text), inner);
    }
}
