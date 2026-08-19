use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
};

pub struct MultiSelect {
    pub label: String,
    pub options: Vec<(String, String)>, // (value, label)
    pub selected: Vec<bool>,
    pub cursor: usize,
    pub focused: bool,
}

#[allow(dead_code)]
impl MultiSelect {
    pub fn new(label: &str, options: Vec<(String, String)>) -> Self {
        let len = options.len();
        Self {
            label: label.to_string(),
            options,
            selected: vec![false; len],
            cursor: 0,
            focused: false,
        }
    }

    pub fn toggle_current(&mut self) {
        if self.cursor < self.selected.len() {
            self.selected[self.cursor] = !self.selected[self.cursor];
        }
    }

    pub fn cursor_up(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        } else if !self.options.is_empty() {
            self.cursor = self.options.len() - 1;
        }
    }

    pub fn cursor_down(&mut self) {
        if !self.options.is_empty() {
            self.cursor = (self.cursor + 1) % self.options.len();
        }
    }

    pub fn selected_count(&self) -> usize {
        self.selected.iter().filter(|&&s| s).count()
    }

    pub fn selected_values(&self) -> Vec<String> {
        self.options
            .iter()
            .zip(self.selected.iter())
            .filter(|(_, sel)| **sel)
            .map(|((val, _), _)| val.clone())
            .collect()
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        if area.height < 2 || area.width < 10 {
            return;
        }

        // ── Outer border with label and count ──
        let border_color = if self.focused {
            Color::Cyan
        } else {
            Color::Rgb(80, 80, 120)
        };

        let selected_count = self.selected_count();
        let title = format!(
            " {} ({}/{}) ",
            self.label,
            selected_count,
            self.options.len()
        );

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(border_color));

        let inner = block.inner(area);
        f.render_widget(block, area);

        if inner.height == 0 || inner.width == 0 {
            return;
        }

        // ── Render each option with checkbox ──
        let mut lines: Vec<Line> = Vec::new();

        for (i, (_value, label)) in self.options.iter().enumerate() {
            let is_checked = self.selected.get(i).copied().unwrap_or(false);
            let is_cursor = i == self.cursor && self.focused;

            let checkbox = if is_checked { "☑" } else { "☐" };
            let checkbox_color = if is_checked {
                Color::Rgb(100, 220, 130)
            } else {
                Color::Rgb(100, 100, 130)
            };

            let cursor_indicator = if is_cursor { "▶ " } else { "  " };
            let cursor_color = if is_cursor {
                Color::Cyan
            } else {
                Color::Rgb(80, 80, 120)
            };

            let label_style = if is_cursor {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else if is_checked {
                Style::default().fg(Color::White)
            } else {
                Style::default().fg(Color::Rgb(180, 180, 200))
            };

            lines.push(Line::from(vec![
                Span::styled(cursor_indicator, Style::default().fg(cursor_color)),
                Span::styled(
                    format!("{} ", checkbox),
                    Style::default().fg(checkbox_color),
                ),
                Span::styled(label.as_str(), label_style),
            ]));
        }

        let paragraph = Paragraph::new(lines);
        f.render_widget(paragraph, inner);
    }
}
