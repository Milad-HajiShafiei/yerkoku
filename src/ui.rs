use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph, Tabs, Wrap,
    },
};

use crate::app::{App, Screen};
use crate::blueprint::FieldType;
use crate::widgets::{
    Button, Checkbox, CrateInput, CrateSearch, ListBuilder, MultiSelect, Select, TextInput,
};

// ─────────────────────────────────────────────
// Color palette
// ─────────────────────────────────────────────

const COLOR_BG: Color = Color::Rgb(25, 25, 35);
const COLOR_SIDEBAR: Color = Color::Rgb(35, 35, 50);
const COLOR_ACCENT: Color = Color::Rgb(100, 200, 255);
const COLOR_SUCCESS: Color = Color::Rgb(100, 220, 130);
const COLOR_WARNING: Color = Color::Rgb(255, 200, 80);
const COLOR_DANGER: Color = Color::Rgb(255, 100, 100);
const COLOR_TEXT: Color = Color::Rgb(230, 230, 240);
const COLOR_DIM: Color = Color::Rgb(150, 150, 170);
const COLOR_BORDER: Color = Color::Rgb(80, 80, 120);

// ─────────────────────────────────────────────
// Main render dispatcher
// ─────────────────────────────────────────────

pub fn render(f: &mut Frame, app: &App) {
    let size = f.area();
    f.render_widget(Clear, size);
    let bg = Block::default().style(Style::default().bg(COLOR_BG));
    f.render_widget(bg, size);

    match app.current_screen {
        Screen::Menu | Screen::Drafts => render_combined_screen(f, app, size),
        Screen::Form => render_form_split(f, app, size),
        Screen::Review => render_review(f, app, size),
        Screen::Success => {
            render_form_split(f, app, size);
            render_success_modal(f, app, size);
        }
    }

    // Exit confirmation modal (on top of form)
    if app.show_exit_confirm {
        render_exit_confirm_modal(f, app, size);
    }

    // Error modal appears on top of everything
    if app.has_error() {
        render_error_modal(f, app, size);
    }
}

// ─────────────────────────────────────────────
// Combined Blueprints + Drafts screen
// ─────────────────────────────────────────────

fn render_combined_screen(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Content (two panels)
            Constraint::Length(3), // Footer
        ])
        .split(area);

    // Header
    let header = Block::default()
        .title(" 🥕 Yerkoku 🥕 — AI Prompt Generator")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(COLOR_ACCENT))
        .style(Style::default().bg(COLOR_SIDEBAR));
    f.render_widget(header, chunks[0]);

    // Content: two panels side by side
    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);

    render_blueprints_panel(f, app, content_chunks[0]);
    render_drafts_panel(f, app, content_chunks[1]);

    // Footer
    let footer_text = match app.panel_focus {
        crate::app::PanelFocus::Blueprints => {
            "↑/↓: Navigate | Enter: Select | Tab/→: Drafts | r: Refresh | q: Quit"
        }
        crate::app::PanelFocus::Drafts => {
            "↑/↓: Navigate | Enter: Open | d: Delete | Tab/←: Blueprints | n: New | q: Quit"
        }
    };

    let footer = Paragraph::new(Line::from(vec![
        Span::styled(" ⌨  ", Style::default().fg(COLOR_ACCENT)),
        Span::styled(footer_text, Style::default().fg(COLOR_TEXT)),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(COLOR_BORDER)),
    );
    f.render_widget(footer, chunks[2]);
}

fn render_blueprints_panel(f: &mut Frame, app: &App, area: Rect) {
    let is_focused = matches!(app.panel_focus, crate::app::PanelFocus::Blueprints);

    let border_color = if is_focused {
        COLOR_ACCENT
    } else {
        COLOR_BORDER
    };

    let title = if is_focused {
        format!(" 📂 Blueprints ({}) ", app.blueprints.len())
    } else {
        format!(" 📂 Blueprints ({}) ", app.blueprints.len())
    };

    if app.blueprints.is_empty() {
        let empty = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(
                "No blueprints found!",
                Style::default().fg(COLOR_WARNING),
            )),
            Line::from(Span::styled(
                "Run with --init to install defaults",
                Style::default().fg(COLOR_DIM),
            )),
        ])
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(border_color)),
        );
        f.render_widget(empty, area);
        return;
    }

    let items: Vec<ListItem> = app
        .blueprints
        .iter()
        .enumerate()
        .map(|(i, bp)| {
            let selected = i == app.selected_blueprint && is_focused;
            let style = if selected {
                Style::default()
                    .fg(COLOR_ACCENT)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(COLOR_TEXT)
            };
            let prefix = if selected { "▶ " } else { "  " };
            ListItem::new(Line::from(vec![
                Span::styled(prefix, style),
                Span::styled(format!("{} ", bp.icon), style),
                Span::styled(&bp.name, style),
                Span::styled(
                    format!("  {}", bp.description),
                    Style::default().fg(COLOR_DIM),
                ),
            ]))
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(border_color)),
        )
        .highlight_style(
            Style::default()
                .bg(Color::Rgb(60, 60, 80))
                .add_modifier(Modifier::BOLD),
        );

    let mut state = ListState::default();
    state.select(Some(app.selected_blueprint));
    f.render_stateful_widget(list, area, &mut state);
}

fn render_drafts_panel(f: &mut Frame, app: &App, area: Rect) {
    let is_focused = matches!(app.panel_focus, crate::app::PanelFocus::Drafts);

    let border_color = if is_focused {
        COLOR_ACCENT
    } else {
        COLOR_BORDER
    };

    let title = format!(" 📝 Drafts ({}) ", app.drafts.len());

    if app.drafts.is_empty() {
        let empty = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(
                "No drafts yet!",
                Style::default().fg(COLOR_DIM),
            )),
            Line::from(Span::styled(
                "Select a blueprint to start",
                Style::default().fg(COLOR_DIM),
            )),
        ])
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(border_color)),
        );
        f.render_widget(empty, area);
        return;
    }

    let items: Vec<ListItem> = app
        .drafts
        .iter()
        .enumerate()
        .map(|(i, (_filename, draft))| {
            let selected = i == app.selected_draft && is_focused;
            let style = if selected {
                Style::default()
                    .fg(COLOR_ACCENT)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(COLOR_TEXT)
            };
            let prefix = if selected { "▶ " } else { "  " };
            ListItem::new(Line::from(vec![
                Span::styled(prefix, style),
                Span::styled("📋 ", style),
                Span::styled(&draft.name, style),
                Span::styled(
                    format!("  [{}]", draft.blueprint_name),
                    Style::default().fg(COLOR_DIM),
                ),
            ]))
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(border_color)),
        )
        .highlight_style(
            Style::default()
                .bg(Color::Rgb(60, 60, 80))
                .add_modifier(Modifier::BOLD),
        );

    let mut state = ListState::default();
    state.select(Some(app.selected_draft));
    f.render_stateful_widget(list, area, &mut state);
}

// ─────────────────────────────────────────────
// Form screen (split: form left, preview right)
// ─────────────────────────────────────────────

fn render_form_split(f: &mut Frame, app: &App, area: Rect) {
    let blueprint = match &app.current_blueprint {
        Some(bp) => bp,
        None => return,
    };

    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Navbar
            Constraint::Min(0),    // Content
            Constraint::Length(3), // Footer
        ])
        .split(area);

    render_navbar(f, app, blueprint, main_chunks[0]);

    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(main_chunks[1]);

    render_form_left(f, app, content_chunks[0]);
    render_preview_right(f, app, content_chunks[1]);

    let footer = Paragraph::new(Line::from(vec![
        Span::styled(" ⌨  ", Style::default().fg(COLOR_ACCENT)),
        Span::styled(
            "↑/↓: Fields | Tab: Next | Enter: Edit/Toggle | Space: Button | ←/→: Sections | s: Draft | g: Generate | Esc: Back",
            Style::default().fg(COLOR_TEXT),
        ),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(COLOR_BORDER)),
    );
    f.render_widget(footer, main_chunks[2]);
}

// ─────────────────────────────────────────────
// Navbar (scrollable tabs)
// ─────────────────────────────────────────────

fn render_navbar(f: &mut Frame, app: &App, blueprint: &crate::blueprint::Blueprint, area: Rect) {
    let total_sections = blueprint.sections.len();
    if total_sections == 0 {
        return;
    }

    let available_width = area.width.saturating_sub(4) as usize;

    // Calculate tab widths for ALL sections
    let mut tab_widths: Vec<usize> = Vec::with_capacity(total_sections);
    for section in &blueprint.sections {
        let tab_text = format!("{} {}", section.icon, section.title);
        let tab_width = tab_text.chars().count() + 4;
        tab_widths.push(tab_width);
    }

    // Determine visible count
    let mut cumulative_width: usize = 0;
    let mut visible_count: usize = 0;
    for &width in &tab_widths {
        if cumulative_width + width <= available_width {
            cumulative_width += width;
            visible_count += 1;
        } else {
            break;
        }
    }

    if visible_count == 0 {
        visible_count = 1;
    }

    // Calculate scroll offset to keep current section visible
    let current = app.form.current_section;
    let mut scroll_offset = app.navbar_scroll_offset;

    if current < scroll_offset {
        scroll_offset = current;
    } else if current >= scroll_offset + visible_count {
        scroll_offset = current.saturating_sub(visible_count - 1);
    }

    let max_scroll = total_sections.saturating_sub(visible_count);
    scroll_offset = scroll_offset.min(max_scroll);

    // Build visible tabs
    let mut visible_titles: Vec<Line> = Vec::new();
    let mut selected_index = 0;

    let end_index = (scroll_offset + visible_count).min(total_sections);
    for i in scroll_offset..end_index {
        if i >= blueprint.sections.len() {
            break;
        }

        let section = &blueprint.sections[i];
        let is_selected = i == app.form.current_section;
        if is_selected {
            selected_index = visible_titles.len();
        }

        let style = if is_selected {
            Style::default()
                .fg(COLOR_ACCENT)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(COLOR_DIM)
        };

        visible_titles.push(Line::from(Span::styled(
            format!("{} {}", section.icon, section.title),
            style,
        )));
    }

    // Build navbar title with scroll indicators
    let can_scroll_left = scroll_offset > 0;
    let can_scroll_right = scroll_offset + visible_count < total_sections;

    let navbar_title = if total_sections > visible_count {
        let left_indicator = if can_scroll_left { "◀ " } else { "  " };
        let right_indicator = if can_scroll_right { " ▶" } else { "  " };
        format!("{}📝 {}{}", left_indicator, blueprint.name, right_indicator)
    } else {
        format!("📝 {}", blueprint.name)
    };

    let tabs = Tabs::new(visible_titles)
        .block(
            Block::default()
                .title(navbar_title)
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(COLOR_BORDER))
                .style(Style::default().bg(COLOR_SIDEBAR)),
        )
        .select(selected_index)
        .highlight_style(
            Style::default()
                .fg(COLOR_ACCENT)
                .add_modifier(Modifier::BOLD),
        )
        .divider(Span::styled(" │ ", Style::default().fg(COLOR_BORDER)));

    f.render_widget(tabs, area);
}

// ─────────────────────────────────────────────
// Form left panel
// ─────────────────────────────────────────────

fn render_form_left(f: &mut Frame, app: &App, area: Rect) {
    let blueprint = match &app.current_blueprint {
        Some(bp) => bp,
        None => return,
    };

    // Loading overlay
    if app.is_loading {
        let loading = Paragraph::new(vec![
            Line::from(""),
            Line::from(""),
            Line::from(Span::styled(
                format!("⏳ {}", app.loading_message),
                Style::default()
                    .fg(COLOR_WARNING)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Please wait...",
                Style::default().fg(COLOR_DIM),
            )),
        ])
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .title("Loading")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(COLOR_WARNING))
                .style(Style::default().bg(COLOR_SIDEBAR)),
        );
        f.render_widget(loading, area);
        return;
    }

    let section = match blueprint.sections.get(app.form.current_section) {
        Some(s) => s,
        None => return,
    };

    let form_block = Block::default()
        .title(format!(
            "📋 {} (Section {}/{})",
            section.title,
            app.form.current_section + 1,
            app.form.total_sections
        ))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(COLOR_BORDER))
        .style(Style::default().bg(COLOR_SIDEBAR));

    let inner = form_block.inner(area);
    f.render_widget(form_block, area);

    // Collect editable fields
    let editable_fields: Vec<(usize, &crate::blueprint::Field)> = section
        .fields
        .iter()
        .enumerate()
        .filter(|(_, f)| !matches!(f.field_type, FieldType::SectionBreak) && !f.hidden)
        .collect();

    if editable_fields.is_empty() {
        f.render_widget(
            Paragraph::new("No fields in this section.").style(Style::default().fg(COLOR_DIM)),
            inner,
        );
        return;
    }

    let total_fields = editable_fields.len();

    // Calculate field heights
    let field_heights: Vec<u16> = editable_fields
        .iter()
        .map(|(_, field)| match field.field_type {
            FieldType::Checkbox => 3,
            FieldType::Textarea => 6,
            FieldType::CrateInput => 5,
            FieldType::ListBuilder => 12,
            FieldType::CrateSearch => 12,
            FieldType::ActionButton => 5,
            FieldType::Multiselect => (field.options.len() as u16 + 2).max(3),
            _ => 3,
        })
        .collect();

    // Calculate how many fields fit on screen
    let available_height = inner.height;
    let max_fields_to_render = 20;
    let mut visible_count = 0;
    let mut cumulative_height: u16 = 0;

    for &height in &field_heights {
        if cumulative_height + height <= available_height && visible_count < max_fields_to_render {
            cumulative_height += height;
            visible_count += 1;
        } else {
            break;
        }
    }

    if visible_count == 0 {
        visible_count = 1;
    }

    // ── Self-correct scroll offset to ALWAYS show selected field ──
    let selected = app.form.selected_field.min(total_fields.saturating_sub(1));
    let mut scroll_offset = app.form.scroll_offset;

    // If selected field is above visible window, scroll up
    if selected < scroll_offset {
        scroll_offset = selected;
    }
    // If selected field is below visible window, scroll down
    else if selected >= scroll_offset + visible_count {
        scroll_offset = selected.saturating_sub(visible_count - 1);
    }

    // Clamp scroll offset
    let max_scroll = total_fields.saturating_sub(visible_count);
    scroll_offset = scroll_offset.min(max_scroll);

    // Render visible fields
    let mut y_offset = inner.y;
    let mut rendered = 0;

    for i in scroll_offset..total_fields {
        if rendered >= visible_count || rendered >= max_fields_to_render {
            break;
        }

        let (_original_idx, field) = &editable_fields[i];
        let field_height = field_heights.get(i).copied().unwrap_or(3);

        if y_offset + field_height > inner.y + inner.height {
            break;
        }

        let field_area = Rect {
            x: inner.x,
            y: y_offset,
            width: inner.width,
            height: field_height,
        };

        // Use filtered index 'i' for focus comparison (not original index)
        let is_focused = i == app.form.selected_field;
        let is_editing = is_focused && app.form.is_editing();
        let value = app.form.values.get(&field.key).cloned().unwrap_or_default();

        // Render widget based on type
        match field.field_type {
            FieldType::Text | FieldType::Textarea => {
                let mut widget = TextInput::new(&field.label, value.as_str(), &field.placeholder);
                widget.focused = is_focused;
                widget.editing = is_editing;
                widget.cursor_pos = if is_editing { app.form.cursor_pos } else { 0 };
                widget.multiline = matches!(field.field_type, FieldType::Textarea);
                widget.scroll_offset = if is_editing {
                    app.form.text_scroll_offset
                } else {
                    0
                };
                widget.render(f, field_area);
            }
            FieldType::Checkbox => {
                let mut widget = Checkbox::new(&field.label, value.as_bool());
                widget.focused = is_focused;
                widget.render(f, field_area);
            }
            FieldType::Select => {
                let options: Vec<(String, String)> = field
                    .options
                    .iter()
                    .map(|opt| (opt.value.clone(), opt.label.clone()))
                    .collect();
                let mut widget = Select::new(&field.label, options);
                widget.focused = is_focused;
                widget.selected = field
                    .options
                    .iter()
                    .position(|opt| opt.value == value.as_str())
                    .unwrap_or(0);
                widget.render(f, field_area);
            }
            FieldType::Multiselect => {
                let options: Vec<(String, String)> = field
                    .options
                    .iter()
                    .map(|opt| (opt.value.clone(), opt.label.clone()))
                    .collect();
                let mut widget = MultiSelect::new(&field.label, options);
                widget.focused = is_focused;
                let selected_values = value.as_vec();
                widget.selected = field
                    .options
                    .iter()
                    .map(|opt| selected_values.contains(&opt.value))
                    .collect();
                widget.cursor = app
                    .form
                    .list_selected
                    .min(widget.options.len().saturating_sub(1));
                widget.render(f, field_area);
            }
            FieldType::CrateInput => {
                let crate_name = field
                    .crate_name
                    .as_deref()
                    .or(field.search_crate.as_deref())
                    .unwrap_or("");
                let mut widget =
                    CrateInput::new(&field.label, crate_name, value.as_str(), &field.placeholder);
                widget.focused = is_focused;
                widget.editing = is_editing;
                widget.button_focused = is_focused && app.form.sub_focus == 1;
                widget.cursor_pos = if is_editing { app.form.cursor_pos } else { 0 };
                widget.loading = app.is_loading;
                widget.render(f, field_area);
            }
            FieldType::ListBuilder => {
                let items = value.as_vec();
                // Use field-specific input value, NOT get_current_key()
                let input_value = app.form.get_list_input_value_for_key(&field.key);
                let mut widget = ListBuilder::new(&field.label, items, &field.placeholder);
                widget.focused = is_focused;
                widget.input_value = input_value;
                widget.input_editing = is_editing;
                widget.button_focused = is_focused && app.form.sub_focus == 1;
                widget.list_focused = is_focused && app.form.sub_focus == 2;
                widget.list_selected = app.form.list_selected;
                widget.list_scroll_offset = app.form.list_scroll_offset;
                widget.cursor_pos = if is_editing { app.form.cursor_pos } else { 0 };
                widget.render(f, field_area);
            }
            FieldType::CrateSearch => {
                let target_key = field
                    .target_list_key
                    .as_deref()
                    .unwrap_or("tech.additional_crates");
                let added_crates = app.form.get_target_list(target_key);
                // Use field-specific input value
                let input_value = app.form.get_list_input_value_for_key(&field.key);
                let mut widget = CrateSearch::new(&field.label, added_crates, &field.placeholder);
                widget.focused = is_focused;
                widget.input_value = input_value;
                widget.input_editing = is_editing;
                widget.button_focused = is_focused && app.form.sub_focus == 1;
                widget.list_focused = is_focused && app.form.sub_focus == 2;
                widget.list_selected = app.form.list_selected;
                widget.cursor_pos = if is_editing { app.form.cursor_pos } else { 0 };
                widget.loading = app.is_loading;
                widget.render(f, field_area);
            }
            FieldType::ActionButton => {
                let button_text = field.button_text.as_deref().unwrap_or("Action");
                let display_text = if is_focused {
                    format!("{} [Space]", button_text)
                } else {
                    button_text.to_string()
                };

                // Button width: text length + padding, capped at 40
                let text_len = display_text.chars().count() as u16 + 10;
                let button_width = text_len.min(40).min(field_area.width);

                // Center horizontally within the available area
                let x_offset = (field_area.width.saturating_sub(button_width)) / 2;

                let button_area = Rect {
                    x: field_area.x + x_offset,
                    y: field_area.y,
                    width: button_width,
                    height: field_area.height,
                };

                // Pass empty label to hide the label text
                let mut widget = Button::new("Generate Prompt", &display_text);
                widget.focused = is_focused;
                widget.render(f, button_area);
            }
            FieldType::SearchCrate => {
                let mut widget = TextInput::new(&field.label, value.as_str(), &field.placeholder);
                widget.focused = is_focused;
                widget.editing = is_editing;
                widget.cursor_pos = if is_editing { app.form.cursor_pos } else { 0 };
                widget.render(f, field_area);
            }
            _ => {}
        }

        y_offset += field_height;
        rendered += 1;
    }

    // Scroll indicator
    if total_fields > visible_count {
        let end_shown = (scroll_offset + rendered).min(total_fields);
        let scroll_info = format!(" {}-{} of {} ", scroll_offset + 1, end_shown, total_fields);
        let indicator = Paragraph::new(scroll_info)
            .style(Style::default().fg(COLOR_DIM).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Right);

        let indicator_area = Rect {
            x: inner.x + inner.width.saturating_sub(20),
            y: area.y,
            width: 20,
            height: 1,
        };
        f.render_widget(indicator, indicator_area);
    }
}

// ─────────────────────────────────────────────
// Preview right panel
// ─────────────────────────────────────────────

fn render_preview_right(f: &mut Frame, app: &App, area: Rect) {
    let preview_block = Block::default()
        .title("👁️ Prompt Preview (Scroll with mouse)")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(COLOR_SUCCESS))
        .style(Style::default().bg(Color::Rgb(20, 20, 30)));

    let inner = preview_block.inner(area);
    f.render_widget(preview_block, area);

    if inner.height == 0 || inner.width == 0 {
        return;
    }

    let lines = generate_styled_preview(app);
    let total_lines = lines.len();
    let visible_lines = inner.height as usize;

    let max_scroll = total_lines.saturating_sub(visible_lines) as u16;
    let scroll = app.preview_scroll.min(max_scroll);

    let paragraph = Paragraph::new(lines)
        .wrap(Wrap { trim: false })
        .scroll((scroll, 0));

    f.render_widget(paragraph, inner);
}

fn generate_styled_preview(app: &App) -> Vec<Line<'static>> {
    let mut lines: Vec<Line<'static>> = Vec::new();

    if let Some(blueprint) = &app.current_blueprint {
        lines.push(Line::from(Span::styled(
            format!("{} {}", blueprint.icon, blueprint.name),
            Style::default()
                .fg(COLOR_ACCENT)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));

        for section in &blueprint.sections {
            lines.push(Line::from(Span::styled(
                format!("{} {}", section.icon, section.title),
                Style::default()
                    .fg(COLOR_WARNING)
                    .add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(Span::styled(
                "─".repeat(50),
                Style::default().fg(COLOR_BORDER),
            )));

            for field in &section.fields {
                if matches!(
                    field.field_type,
                    FieldType::SectionBreak | FieldType::ActionButton
                ) {
                    continue;
                }
                if field.hidden {
                    continue;
                }

                let value = app.form.values.get(&field.key).cloned().unwrap_or_default();
                if value.is_empty() {
                    continue;
                }

                let label_owned = field.label.clone();

                match &field.field_type {
                    FieldType::Checkbox => {
                        if value.as_bool() {
                            lines.push(Line::from(vec![
                                Span::styled("  ✓ ", Style::default().fg(COLOR_SUCCESS)),
                                Span::styled(label_owned, Style::default().fg(COLOR_TEXT)),
                            ]));
                        }
                    }
                    FieldType::Select => {
                        let val_owned = value.as_str().to_string();
                        lines.push(Line::from(vec![
                            Span::styled("  • ", Style::default().fg(COLOR_ACCENT)),
                            Span::styled(
                                format!("{}: ", label_owned),
                                Style::default().fg(COLOR_DIM),
                            ),
                            Span::styled(val_owned, Style::default().fg(COLOR_TEXT)),
                        ]));
                    }
                    FieldType::Multiselect => {
                        let values = value.as_vec();
                        if !values.is_empty() {
                            lines.push(Line::from(vec![
                                Span::styled("  • ", Style::default().fg(COLOR_ACCENT)),
                                Span::styled(
                                    format!("{}: ", label_owned),
                                    Style::default().fg(COLOR_DIM),
                                ),
                                Span::styled(values.join(", "), Style::default().fg(COLOR_TEXT)),
                            ]));
                        }
                    }
                    FieldType::ListBuilder => {
                        let items = value.as_vec();
                        if !items.is_empty() {
                            lines.push(Line::from(vec![Span::styled(
                                format!("  {}:", label_owned),
                                Style::default().fg(COLOR_DIM),
                            )]));
                            for item in &items {
                                let item_owned = item.to_string();
                                lines.push(Line::from(vec![
                                    Span::styled("    • ", Style::default().fg(COLOR_ACCENT)),
                                    Span::styled(item_owned, Style::default().fg(COLOR_TEXT)),
                                ]));
                            }
                        }
                    }
                    FieldType::Textarea => {
                        lines.push(Line::from(vec![Span::styled(
                            format!("  {}:", label_owned),
                            Style::default().fg(COLOR_DIM),
                        )]));
                        for line in value.as_str().lines() {
                            lines.push(Line::from(vec![
                                Span::styled("    ", Style::default().fg(COLOR_DIM)),
                                Span::styled(line.to_string(), Style::default().fg(COLOR_TEXT)),
                            ]));
                        }
                    }
                    FieldType::CrateInput => {
                        let val_owned = value.as_str().to_string();
                        if !val_owned.is_empty() {
                            lines.push(Line::from(vec![
                                Span::styled("  • ", Style::default().fg(COLOR_ACCENT)),
                                Span::styled(
                                    format!("{}: ", label_owned),
                                    Style::default().fg(COLOR_DIM),
                                ),
                                Span::styled(val_owned, Style::default().fg(COLOR_SUCCESS)),
                            ]));
                        }
                    }
                    _ => {
                        let val_owned = value.as_str().to_string();
                        if !val_owned.is_empty() {
                            lines.push(Line::from(vec![
                                Span::styled("  • ", Style::default().fg(COLOR_ACCENT)),
                                Span::styled(
                                    format!("{}: ", label_owned),
                                    Style::default().fg(COLOR_DIM),
                                ),
                                Span::styled(val_owned, Style::default().fg(COLOR_TEXT)),
                            ]));
                        }
                    }
                }
            }

            lines.push(Line::from(""));
        }
    }

    lines
}

// ─────────────────────────────────────────────
// Review screen
// ─────────────────────────────────────────────

fn render_review(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(area);

    let header = Block::default()
        .title("👀 Review & Generate")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(COLOR_SUCCESS))
        .style(Style::default().bg(COLOR_SIDEBAR));
    f.render_widget(header, chunks[0]);

    let blueprint = match &app.current_blueprint {
        Some(bp) => bp,
        None => return,
    };

    let filled_count = app.form.values.values().filter(|v| !v.is_empty()).count();
    let total_count = app.form.values.len();

    let summary = vec![
        Line::from(vec![
            Span::styled("Blueprint: ", Style::default().fg(COLOR_DIM)),
            Span::styled(
                &blueprint.name,
                Style::default()
                    .fg(COLOR_ACCENT)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Fields filled: ", Style::default().fg(COLOR_DIM)),
            Span::styled(
                format!("{}/{}", filled_count, total_count),
                Style::default().fg(if filled_count > 0 {
                    COLOR_SUCCESS
                } else {
                    COLOR_WARNING
                }),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Press ", Style::default().fg(COLOR_DIM)),
            Span::styled(
                "Enter",
                Style::default()
                    .fg(COLOR_SUCCESS)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" or ", Style::default().fg(COLOR_DIM)),
            Span::styled(
                "g",
                Style::default()
                    .fg(COLOR_SUCCESS)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                " to generate and save prompt",
                Style::default().fg(COLOR_DIM),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Press ", Style::default().fg(COLOR_DIM)),
            Span::styled(
                "e",
                Style::default()
                    .fg(COLOR_ACCENT)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" to edit", Style::default().fg(COLOR_DIM)),
        ]),
    ];

    let paragraph = Paragraph::new(summary)
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(COLOR_BORDER)),
        )
        .wrap(Wrap { trim: false });
    f.render_widget(paragraph, chunks[1]);

    let footer = Paragraph::new(Line::from(vec![
        Span::styled(" ⌨  ", Style::default().fg(COLOR_ACCENT)),
        Span::styled(
            "Enter/g: Generate | e: Edit | Esc: Back",
            Style::default().fg(COLOR_TEXT),
        ),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(COLOR_BORDER)),
    );
    f.render_widget(footer, chunks[2]);
}

// ─────────────────────────────────────────────
// Success modal (solid background)
// ─────────────────────────────────────────────

fn render_success_modal(f: &mut Frame, app: &App, area: Rect) {
    // Fill entire screen with solid dark overlay using spaces
    let mut overlay_content = String::new();
    for _ in 0..area.height {
        overlay_content.push_str(&" ".repeat(area.width as usize));
        overlay_content.push('\n');
    }
    let overlay = Paragraph::new(overlay_content).style(Style::default().bg(Color::Rgb(5, 5, 10)));
    f.render_widget(overlay, area);

    // Calculate modal size
    let modal_width = 85u16.min(area.width.saturating_sub(4));
    let modal_height = 14u16.min(area.height.saturating_sub(4));

    let modal_x = area.x + (area.width.saturating_sub(modal_width)) / 2;
    let modal_y = area.y + (area.height.saturating_sub(modal_height)) / 2;
    let modal_area = Rect::new(modal_x, modal_y, modal_width, modal_height);

    // Determine colors
    let (title, border_color, modal_bg) = if app.last_error.is_some() {
        ("✗ Error", COLOR_DANGER, Color::Rgb(35, 20, 25))
    } else {
        ("✓ Success", COLOR_SUCCESS, Color::Rgb(20, 35, 25))
    };

    // Fill modal area with solid background
    let mut modal_fill_content = String::new();
    for _ in 0..modal_area.height {
        modal_fill_content.push_str(&" ".repeat(modal_area.width as usize));
        modal_fill_content.push('\n');
    }
    let modal_fill = Paragraph::new(modal_fill_content).style(Style::default().bg(modal_bg));
    f.render_widget(modal_fill, modal_area);

    // Modal border block
    let modal_block = Block::default()
        .title(format!(" {} ", title))
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(border_color))
        .style(Style::default().bg(modal_bg));

    let inner = modal_block.inner(modal_area);
    f.render_widget(modal_block, modal_area);

    if inner.height == 0 || inner.width == 0 {
        return;
    }

    // Fill inner content area with solid background
    let mut inner_fill_content = String::new();
    for _ in 0..inner.height {
        inner_fill_content.push_str(&" ".repeat(inner.width as usize));
        inner_fill_content.push('\n');
    }
    let inner_fill = Paragraph::new(inner_fill_content).style(Style::default().bg(modal_bg));
    f.render_widget(inner_fill, inner);

    // Build modal content
    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(""));

    if let Some(error) = &app.last_error {
        lines.push(Line::from(Span::styled(
            error.clone(),
            Style::default()
                .fg(COLOR_DANGER)
                .bg(modal_bg)
                .add_modifier(Modifier::BOLD),
        )));
    } else {
        lines.push(Line::from(Span::styled(
            "Prompt generated successfully!",
            Style::default()
                .fg(COLOR_SUCCESS)
                .bg(modal_bg)
                .add_modifier(Modifier::BOLD),
        )));

        if let Some(copy_msg) = &app.last_copy_result {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                copy_msg.clone(),
                Style::default().fg(COLOR_ACCENT).bg(modal_bg),
            )));
        }

        if let Some(path) = &app.last_output_path {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "Saved to:",
                Style::default().fg(COLOR_DIM).bg(modal_bg),
            )));
            lines.push(Line::from(Span::styled(
                format!("  {}", path),
                Style::default()
                    .fg(COLOR_WARNING)
                    .bg(modal_bg)
                    .add_modifier(Modifier::BOLD),
            )));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled(" Press ", Style::default().fg(COLOR_DIM).bg(modal_bg)),
        Span::styled(
            "Enter",
            Style::default()
                .fg(COLOR_ACCENT)
                .bg(modal_bg)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            " to return  •  ",
            Style::default().fg(COLOR_DIM).bg(modal_bg),
        ),
        Span::styled(
            "o",
            Style::default()
                .fg(COLOR_WARNING)
                .bg(modal_bg)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            " open prompts folder  ",
            Style::default().fg(COLOR_DIM).bg(modal_bg),
        ),
        Span::styled(
            "m",
            Style::default()
                .fg(COLOR_ACCENT)
                .bg(modal_bg)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            " for menu  •  ",
            Style::default().fg(COLOR_DIM).bg(modal_bg),
        ),
        Span::styled(
            "q",
            Style::default()
                .fg(COLOR_ACCENT)
                .bg(modal_bg)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" to quit", Style::default().fg(COLOR_DIM).bg(modal_bg)),
    ]));

    let modal_content = Paragraph::new(lines)
        .style(Style::default().bg(modal_bg))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: false });

    f.render_widget(modal_content, inner);
}

// ─────────────────────────────────────────────
// Error modal (solid background)
// ─────────────────────────────────────────────

fn render_error_modal(f: &mut Frame, app: &App, area: Rect) {
    let title = app.error_title.as_deref().unwrap_or("Error");
    let message = app
        .error_message
        .as_deref()
        .unwrap_or("An unknown error occurred");
    let details = app.error_details.as_deref();

    // Fill entire screen with solid dark overlay
    let mut overlay_content = String::new();
    for _ in 0..area.height {
        overlay_content.push_str(&" ".repeat(area.width as usize));
        overlay_content.push('\n');
    }
    let overlay = Paragraph::new(overlay_content).style(Style::default().bg(Color::Rgb(5, 5, 10)));
    f.render_widget(overlay, area);

    // Calculate modal size
    let message_lines = message.lines().count().max(1);
    let details_lines = details.map(|d| d.lines().count()).unwrap_or(0);
    let total_content_lines = message_lines + details_lines + 4;

    let modal_width = 70u16.min(area.width.saturating_sub(4));
    let modal_height = (total_content_lines as u16 + 6)
        .min(20u16)
        .min(area.height.saturating_sub(4));

    let modal_x = area.x + (area.width.saturating_sub(modal_width)) / 2;
    let modal_y = area.y + (area.height.saturating_sub(modal_height)) / 2;
    let modal_area = Rect::new(modal_x, modal_y, modal_width, modal_height);

    let modal_bg = Color::Rgb(35, 20, 25);

    // Fill modal area with solid background
    let mut modal_fill_content = String::new();
    for _ in 0..modal_area.height {
        modal_fill_content.push_str(&" ".repeat(modal_area.width as usize));
        modal_fill_content.push('\n');
    }
    let modal_fill = Paragraph::new(modal_fill_content).style(Style::default().bg(modal_bg));
    f.render_widget(modal_fill, modal_area);

    // Modal border block
    let modal_block = Block::default()
        .title(format!(" ✗ {} ", title))
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(COLOR_DANGER))
        .style(Style::default().bg(modal_bg));

    let inner = modal_block.inner(modal_area);
    f.render_widget(modal_block, modal_area);

    if inner.height == 0 || inner.width == 0 {
        return;
    }

    // Fill inner content area
    let mut inner_fill_content = String::new();
    for _ in 0..inner.height {
        inner_fill_content.push_str(&" ".repeat(inner.width as usize));
        inner_fill_content.push('\n');
    }
    let inner_fill = Paragraph::new(inner_fill_content).style(Style::default().bg(modal_bg));
    f.render_widget(inner_fill, inner);

    // Build modal content
    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(""));

    lines.push(Line::from(vec![
        Span::styled(
            "  ⚠  ",
            Style::default()
                .fg(COLOR_DANGER)
                .bg(modal_bg)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            message,
            Style::default()
                .fg(COLOR_TEXT)
                .bg(modal_bg)
                .add_modifier(Modifier::BOLD),
        ),
    ]));

    if let Some(details_text) = details {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "  Details:",
            Style::default().fg(COLOR_DIM).bg(modal_bg),
        )));
        for line in details_text.lines() {
            lines.push(Line::from(Span::styled(
                format!("    {}", line),
                Style::default().fg(COLOR_DIM).bg(modal_bg),
            )));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("  Press ", Style::default().fg(COLOR_DIM).bg(modal_bg)),
        Span::styled(
            "Enter",
            Style::default()
                .fg(COLOR_ACCENT)
                .bg(modal_bg)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" / ", Style::default().fg(COLOR_DIM).bg(modal_bg)),
        Span::styled(
            "Esc",
            Style::default()
                .fg(COLOR_ACCENT)
                .bg(modal_bg)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" / ", Style::default().fg(COLOR_DIM).bg(modal_bg)),
        Span::styled(
            "Space",
            Style::default()
                .fg(COLOR_ACCENT)
                .bg(modal_bg)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" to dismiss", Style::default().fg(COLOR_DIM).bg(modal_bg)),
    ]));

    let modal_content = Paragraph::new(lines)
        .style(Style::default().bg(modal_bg))
        .wrap(Wrap { trim: false });

    f.render_widget(modal_content, inner);
}

// ─────────────────────────────────────────────
// Mouse handlers
// ─────────────────────────────────────────────

pub fn handle_form_mouse(app: &mut App, mouse: &crossterm::event::MouseEvent) {
    use crossterm::event::{MouseButton, MouseEventKind};

    let terminal_width = 120u16;
    let form_width = terminal_width / 2;

    match mouse.kind {
        MouseEventKind::Down(MouseButton::Left) => {
            // Form area click (left half, below navbar)
            if mouse.column < form_width && mouse.row >= 3 {
                let field_row = mouse.row.saturating_sub(5);
                let field_height = 3u16;
                let field_index = (field_row / field_height) as usize + app.form.scroll_offset;
                let field_count = app.form.current_section_field_count();
                if field_index < field_count {
                    app.form.selected_field = field_index;
                }
            }
        }
        MouseEventKind::ScrollUp => {
            if mouse.row < 3 {
                app.navbar_scroll_left();
            } else if mouse.column < form_width {
                if app.form.is_editing() && app.form.is_multiline() {
                    app.form.scroll_text_up(1);
                } else {
                    app.form.scroll_up(1);
                }
            } else {
                app.preview_scroll = app.preview_scroll.saturating_sub(3);
            }
        }
        MouseEventKind::ScrollDown => {
            if mouse.row < 3 {
                app.navbar_scroll_right();
            } else if mouse.column < form_width {
                if app.form.is_editing() && app.form.is_multiline() {
                    app.form.scroll_text_down(1);
                } else {
                    app.form.scroll_down(1);
                }
            } else {
                app.preview_scroll = app.preview_scroll.saturating_add(3);
            }
        }
        _ => {}
    }
}

#[allow(dead_code)]
pub fn handle_success_mouse(app: &mut App, _mouse: &crossterm::event::MouseEvent) {
    app.current_screen = Screen::Form;
}

fn render_exit_confirm_modal(f: &mut Frame, app: &App, area: Rect) {
    // Fill entire screen with solid dark overlay
    let mut overlay_content = String::new();
    for _ in 0..area.height {
        overlay_content.push_str(&" ".repeat(area.width as usize));
        overlay_content.push('\n');
    }
    let overlay = Paragraph::new(overlay_content).style(Style::default().bg(Color::Rgb(5, 5, 10)));
    f.render_widget(overlay, area);

    // Modal size
    let modal_width = 50u16.min(area.width.saturating_sub(4));
    let modal_height = 11u16.min(area.height.saturating_sub(4));

    let modal_x = area.x + (area.width.saturating_sub(modal_width)) / 2;
    let modal_y = area.y + (area.height.saturating_sub(modal_height)) / 2;
    let modal_area = Rect::new(modal_x, modal_y, modal_width, modal_height);

    let modal_bg = Color::Rgb(30, 30, 45);

    // Fill modal area
    let mut modal_fill_content = String::new();
    for _ in 0..modal_area.height {
        modal_fill_content.push_str(&" ".repeat(modal_area.width as usize));
        modal_fill_content.push('\n');
    }
    let modal_fill = Paragraph::new(modal_fill_content).style(Style::default().bg(modal_bg));
    f.render_widget(modal_fill, modal_area);

    // Modal block
    let modal_block = Block::default()
        .title(" ⚠ Unsaved Changes ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(COLOR_WARNING))
        .style(Style::default().bg(modal_bg));

    let inner = modal_block.inner(modal_area);
    f.render_widget(modal_block, modal_area);

    if inner.height == 0 || inner.width == 0 {
        return;
    }

    // Fill inner area
    let mut inner_fill_content = String::new();
    for _ in 0..inner.height {
        inner_fill_content.push_str(&" ".repeat(inner.width as usize));
        inner_fill_content.push('\n');
    }
    let inner_fill = Paragraph::new(inner_fill_content).style(Style::default().bg(modal_bg));
    f.render_widget(inner_fill, inner);

    // Build content
    let options = [
        ("💾 Save Draft & Exit", "s"),
        ("🚪 Exit Without Saving", "x"),
        ("↩  Cancel", "Esc"),
    ];

    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(Span::styled(
        "  Do you want to save your progress?",
        Style::default().fg(COLOR_TEXT).bg(modal_bg),
    )));
    lines.push(Line::from(""));

    for (i, (text, shortcut)) in options.iter().enumerate() {
        let is_selected = i == app.exit_confirm_selection;
        let prefix = if is_selected { "  ▶ " } else { "    " };
        let style = if is_selected {
            Style::default()
                .fg(COLOR_ACCENT)
                .bg(modal_bg)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(COLOR_DIM).bg(modal_bg)
        };
        let shortcut_style = Style::default().fg(COLOR_DIM).bg(modal_bg);

        lines.push(Line::from(vec![
            Span::styled(prefix, style),
            Span::styled(*text, style),
            Span::styled(format!("  [{}]", shortcut), shortcut_style),
        ]));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "  ↑/↓: Navigate | Enter: Select | Esc: Cancel",
        Style::default().fg(COLOR_DIM).bg(modal_bg),
    )));

    let modal_content = Paragraph::new(lines)
        .style(Style::default().bg(modal_bg))
        .wrap(Wrap { trim: false });

    f.render_widget(modal_content, inner);
}
