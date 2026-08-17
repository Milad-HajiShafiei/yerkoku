use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders, Paragraph},
};

pub struct MultiSelect {
    pub label: String,
    pub options: Vec<(String, String)>,
    pub selected: Vec<bool>,
    pub cursor: usize,
    pub focused: bool,
}

impl MultiSelect {
    pub fn new(label: &str, options: Vec<(String, String)>) -> Self {
        let selected = vec![false; options.len()];
        Self {
            label: label.to_string(),
            options,
            selected,
            cursor: 0,
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

        let selected_count = self.selected.iter().filter(|&&s| s).count();
        let display = format!("{} of {} selected", selected_count, self.options.len());

        let style = if self.focused {
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };

        f.render_widget(Paragraph::new(display).style(style), inner);
    }
}
