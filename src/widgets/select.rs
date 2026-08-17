use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders, Paragraph},
};

pub struct Select {
    pub label: String,
    pub options: Vec<(String, String)>,
    pub selected: usize,
    pub focused: bool,
}

impl Select {
    pub fn new(label: &str, options: Vec<(String, String)>) -> Self {
        Self {
            label: label.to_string(),
            options,
            selected: 0,
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
            .title(format!(" {} ", self.label))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(border_color));

        let inner = block.inner(area);
        f.render_widget(block, area);

        if inner.height == 0 || inner.width == 0 {
            return;
        }

        let display = if self.options.is_empty() {
            "No options".to_string()
        } else {
            let idx = self.selected.min(self.options.len().saturating_sub(1));
            let (_, label) = &self.options[idx];
            format!("◈ {}", label)
        };

        let style = if self.focused {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Yellow)
        };

        f.render_widget(Paragraph::new(display).style(style), inner);
    }
}
