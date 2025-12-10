use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
    Frame,
};

use crate::app::{App, Mode, Panel, SelectionHighlightMode};
use crate::colors::{theme, COLOR_PALETTE};

/// Render the entire UI
pub fn render(frame: &mut Frame, app: &App) {
    let size = frame.area();

    // Main background
    let bg_block = Block::default().style(Style::default().bg(theme::BG_PRIMARY));
    frame.render_widget(bg_block, size);

    // Calculate controls height based on width (stacked vs horizontal)
    let min_horizontal_width = 80;
    let controls_height = if size.width >= min_horizontal_width + 2 {
        4  // Horizontal: single row of panels
    } else {
        12 // Vertical: stacked panels (4 + 4 + 4)
    };

    // Main layout: header, content, controls, status bar
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),                    // Header
            Constraint::Min(4),                       // Editor (grows to fill)
            Constraint::Length(controls_height),     // Controls
            Constraint::Length(1),                    // Status bar
        ])
        .split(size);

    render_header(frame, chunks[0]);
    
    // Add horizontal and vertical margin around editor
    let editor_area = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),  // Top margin
            Constraint::Min(3),     // Editor
        ])
        .split(
            Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Length(2),  // Left margin
                    Constraint::Min(10),    // Editor
                    Constraint::Length(2),  // Right margin
                ])
                .split(chunks[1])[1]
        )[1];
    
    render_editor(frame, app, editor_area);
    render_controls(frame, app, chunks[2]);
    render_status_bar(frame, app, chunks[3]);
}

fn render_header(frame: &mut Frame, area: Rect) {
    let title = vec![
        Span::styled("Terminal ", Style::default().fg(theme::TEXT_PRIMARY)),
        Span::styled("Text ", Style::default().fg(theme::ACCENT_PRIMARY)),
        Span::styled("Styler", Style::default().fg(theme::TEXT_PRIMARY)),
    ];

    let header = Paragraph::new(Line::from(title))
        .style(Style::default().bg(theme::BG_PRIMARY))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(theme::BORDER_DEFAULT))
                .style(Style::default().bg(theme::BG_PRIMARY)),
        )
        .alignment(ratatui::layout::Alignment::Center);

    frame.render_widget(header, area);
}

fn render_editor(frame: &mut Frame, app: &App, area: Rect) {
    let is_focused = app.active_panel == Panel::Editor;
    let border_color = if is_focused {
        theme::BORDER_FOCUSED
    } else {
        theme::BORDER_DEFAULT
    };

    // Build styled text with cursor
    let mut spans: Vec<Span> = Vec::new();
    let mut selection_line_spans: Vec<Span> = Vec::new();
    let use_underline_mode = app.selection_highlight_mode == SelectionHighlightMode::Underline
        && app.mode == Mode::Selecting;
    
    if app.text.is_empty() {
        // Show placeholder text
        let cursor_style = Style::default()
            .bg(theme::ACCENT_PRIMARY)
            .fg(theme::BG_PRIMARY);
        
        if app.mode == Mode::Typing {
            spans.push(Span::styled("▌", cursor_style));
        }
        spans.push(Span::styled(
            " Type 'i' to insert text...",
            Style::default().fg(theme::TEXT_MUTED),
        ));
    } else {
        for (i, styled_char) in app.text.iter().enumerate() {
            let mut style = Style::default()
                .fg(styled_char.style.fg)
                .bg(styled_char.style.bg);

            // Apply modifiers
            if styled_char.style.bold {
                style = style.add_modifier(Modifier::BOLD);
            }
            if styled_char.style.italic {
                style = style.add_modifier(Modifier::ITALIC);
            }
            if styled_char.style.underline {
                style = style.add_modifier(Modifier::UNDERLINED);
            }
            if styled_char.style.strikethrough {
                style = style.add_modifier(Modifier::CROSSED_OUT);
            }
            if styled_char.style.dim_level > 0 {
                style = style.add_modifier(Modifier::DIM);
            }

            // Selection highlight based on mode
            let is_selected = app.is_selected(i);
            let is_cursor = i == app.cursor_pos && is_focused;

            if use_underline_mode {
                // Underline mode: build selection indicator line
                if is_cursor {
                    selection_line_spans.push(Span::styled(
                        "+",
                        Style::default().fg(theme::ACCENT_PRIMARY).add_modifier(Modifier::BOLD),
                    ));
                } else if is_selected {
                    selection_line_spans.push(Span::styled(
                        "─",
                        Style::default().fg(theme::ACCENT_SECONDARY),
                    ));
                } else {
                    selection_line_spans.push(Span::styled(" ", Style::default()));
                }
                // Cursor still gets subtle highlight
                if is_cursor {
                    style = style.add_modifier(Modifier::BOLD);
                }
            } else {
                // Reversed mode
                if is_selected {
                    style = style.add_modifier(Modifier::REVERSED);
                }
                if is_cursor {
                    style = style.bg(theme::ACCENT_PRIMARY).fg(theme::BG_PRIMARY);
                }
            }

            spans.push(Span::styled(styled_char.ch.to_string(), style));
        }

        // Cursor at end of text
        if app.cursor_pos >= app.text.len() && is_focused {
            if use_underline_mode {
                selection_line_spans.push(Span::styled(
                    "+",
                    Style::default().fg(theme::ACCENT_PRIMARY).add_modifier(Modifier::BOLD),
                ));
            }
            let cursor_style = Style::default()
                .bg(theme::ACCENT_PRIMARY)
                .fg(theme::BG_PRIMARY);
            spans.push(Span::styled("▌", cursor_style));
        }
    }

    let mode_indicator = match app.mode {
        Mode::Normal => "NORMAL",
        Mode::Typing => "INSERT",
        Mode::Selecting => "VISUAL",
    };

    let highlight_indicator = if app.mode == Mode::Selecting {
        match app.selection_highlight_mode {
            SelectionHighlightMode::Reversed => " │ Ctrl+H: underline",
            SelectionHighlightMode::Underline => " │ Ctrl+H: reversed",
        }
    } else {
        ""
    };

    let title = format!(" Editor [{}]{} ", mode_indicator, highlight_indicator);

    // Build lines for paragraph
    let lines = if use_underline_mode && !selection_line_spans.is_empty() {
        vec![Line::from(spans), Line::from(selection_line_spans)]
    } else {
        vec![Line::from(spans)]
    };

    let editor = Paragraph::new(lines)
        .style(Style::default().bg(theme::BG_PRIMARY))
        .block(
            Block::default()
                .title(Span::styled(
                    title,
                    Style::default()
                        .fg(if is_focused { theme::ACCENT_PRIMARY } else { theme::TEXT_SECONDARY })
                        .add_modifier(Modifier::BOLD),
                ))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(border_color))
                .style(Style::default().bg(theme::BG_PRIMARY)),
        )
        .wrap(Wrap { trim: false });

    frame.render_widget(editor, area);
}

fn render_controls(frame: &mut Frame, app: &App, area: Rect) {
    // Responsive layout: stack vertically if narrow (< 80 cols), horizontal otherwise
    let min_horizontal_width = 80;
    
    if area.width >= min_horizontal_width {
        // Horizontal layout: three columns, fixed height
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(35),
                Constraint::Percentage(35),
                Constraint::Percentage(30),
            ])
            .split(area);

        render_color_picker(frame, app, chunks[0], "Foreground [F]", true);
        render_color_picker(frame, app, chunks[1], "Background [G]", false);
        render_formatting_panel(frame, app, chunks[2]);
    } else {
        // Vertical layout: stack panels with fixed heights
        // Total height needed: 4 + 4 + 4 = 12 lines
        let total_needed = 12u16;
        let available = area.height;
        
        // Calculate how much space we have and adjust if needed
        let (fg_h, bg_h, fmt_h) = if available >= total_needed {
            (4, 4, 4)
        } else if available >= 9 {
            (3, 3, available.saturating_sub(6).max(3))
        } else {
            // Very cramped, minimize everything
            let each = available / 3;
            (each, each, available.saturating_sub(each * 2))
        };
        
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(fg_h),
                Constraint::Length(bg_h),
                Constraint::Length(fmt_h),
            ])
            .split(area);

        render_color_picker(frame, app, chunks[0], "FG [F]", true);
        render_color_picker(frame, app, chunks[1], "BG [G]", false);
        render_formatting_panel(frame, app, chunks[2]);
    }
}

fn render_color_picker(frame: &mut Frame, app: &App, area: Rect, title: &str, is_foreground: bool) {
    let is_focused = if is_foreground {
        app.active_panel == Panel::FgColor
    } else {
        app.active_panel == Panel::BgColor
    };

    let border_color = if is_focused {
        theme::BORDER_FOCUSED
    } else {
        theme::BORDER_DEFAULT
    };

    let selected_index = if is_foreground {
        app.fg_color_index
    } else {
        app.bg_color_index
    };

    let current_color = if is_foreground {
        app.current_fg
    } else {
        app.current_bg
    };

    // Create color palette display (2 rows: first row 0-8, second row 9-16)
    let mut line1_spans: Vec<Span> = vec![Span::raw(" ")];
    let mut line2_spans: Vec<Span> = vec![Span::raw(" ")];

    for (i, (color, _name, key)) in COLOR_PALETTE.iter().enumerate() {
        let is_selected = i == selected_index;
        let is_current = *color == current_color;

        // Show key and color block
        let key_char = format!("{}", key);
        let block_display = if is_selected && is_focused {
            "▓"
        } else if is_current {
            "█"
        } else {
            "░"
        };

        let key_style = Style::default().fg(theme::TEXT_MUTED);
        let color_style = Style::default().fg(*color);
        
        let combined = format!("{}{} ", key_char, block_display);
        
        // For Reset/None color, show a special indicator
        let span = if *color == ratatui::style::Color::Reset {
            Span::styled(
                format!("{}◌ ", key_char),
                if is_selected && is_focused {
                    Style::default().fg(theme::ACCENT_PRIMARY)
                } else {
                    key_style
                },
            )
        } else {
            Span::styled(combined, color_style)
        };

        if i < 9 {
            line1_spans.push(span);
        } else {
            line2_spans.push(span);
        }
    }

    let text = vec![Line::from(line1_spans), Line::from(line2_spans)];

    let picker = Paragraph::new(text)
        .style(Style::default().bg(theme::BG_PRIMARY))
        .block(
            Block::default()
                .title(Span::styled(
                    format!(" {} ", title),
                    Style::default()
                        .fg(if is_focused { theme::ACCENT_PRIMARY } else { theme::TEXT_SECONDARY })
                        .add_modifier(Modifier::BOLD),
                ))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(border_color))
                .style(Style::default().bg(theme::BG_PRIMARY)),
        );

    frame.render_widget(picker, area);
}

fn render_formatting_panel(frame: &mut Frame, app: &App, area: Rect) {
    let is_focused = app.active_panel == Panel::Formatting;
    let border_color = if is_focused {
        theme::BORDER_FOCUSED
    } else {
        theme::BORDER_DEFAULT
    };

    // Helper to create decoration indicator
    let make_indicator = |key: &str, label: &str, active: bool| -> Span {
        let style = if active {
            Style::default().fg(theme::ACCENT_PRIMARY).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme::TEXT_MUTED)
        };
        Span::styled(format!("[{}]{} ", key, if active { "✓" } else { label }), style)
    };

    // Dim indicator with levels
    let dim_display = match app.current_dim {
        0 => "░",
        1 => "▒",
        2 => "▓",
        3 => "█",
        _ => "░",
    };

    let lines = vec![
        Line::from(vec![
            make_indicator("B", "old", app.current_bold),
            make_indicator("I", "talic", app.current_italic),
            make_indicator("U", "nder", app.current_underline),
        ]),
        Line::from(vec![
            make_indicator("S", "trike", app.current_strikethrough),
            Span::styled(
                format!("[M]Dim{} ", dim_display),
                if app.current_dim > 0 {
                    Style::default().fg(theme::ACCENT_SECONDARY)
                } else {
                    Style::default().fg(theme::TEXT_MUTED)
                },
            ),
            Span::styled("[E]xport", Style::default().fg(theme::SUCCESS)),
        ]),
    ];

    let panel = Paragraph::new(lines)
        .style(Style::default().bg(theme::BG_PRIMARY))
        .block(
            Block::default()
                .title(Span::styled(
                    " Decorations [D] ",
                    Style::default()
                        .fg(if is_focused { theme::ACCENT_PRIMARY } else { theme::TEXT_SECONDARY })
                        .add_modifier(Modifier::BOLD),
                ))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(border_color))
                .style(Style::default().bg(theme::BG_PRIMARY)),
        );

    frame.render_widget(panel, area);
}

fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let help_text = match app.active_panel {
        Panel::Editor => match app.mode {
            Mode::Normal => "i:insert │ v:select │ e:export │ f/g/d:panels │ Ctrl+Q:quit",
            Mode::Typing => "Esc:normal │ ←→:move │ Backspace:delete",
            Mode::Selecting => "←→:extend │ Enter:apply │ Esc:cancel",
        },
        Panel::FgColor | Panel::BgColor => "0-9,a-g:select │ ←→↑↓:nav │ Enter:apply │ Esc:editor",
        Panel::Formatting => "B/I/U/S/M:toggle │ E:export │ Esc:editor",
    };

    let mut spans = vec![
        Span::styled(" ", Style::default()),
        Span::styled(help_text, Style::default().fg(theme::TEXT_MUTED)),
    ];

    // Add status message if present
    if let Some(ref msg) = app.status_message {
        spans.push(Span::styled(" │ ", Style::default().fg(theme::BORDER_DEFAULT)));
        
        let msg_style = if msg.starts_with('✓') {
            Style::default().fg(theme::SUCCESS)
        } else if msg.starts_with('✗') {
            Style::default().fg(theme::ERROR)
        } else {
            Style::default().fg(theme::ACCENT_SECONDARY)
        };
        
        spans.push(Span::styled(msg.clone(), msg_style));
    }

    let status = Paragraph::new(Line::from(spans))
        .style(Style::default().bg(theme::BG_PRIMARY));

    frame.render_widget(status, area);
}
