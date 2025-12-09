use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
    Frame,
};

use crate::app::{App, Mode, Panel};
use crate::colors::{theme, COLOR_PALETTE};

/// Render the entire UI
pub fn render(frame: &mut Frame, app: &App) {
    let size = frame.area();

    // Main background
    let bg_block = Block::default().style(Style::default().bg(theme::BG_PRIMARY));
    frame.render_widget(bg_block, size);

    // Main layout: header, content, status bar
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Min(6),     // Main content
            Constraint::Length(3),  // Controls
            Constraint::Length(1),  // Status bar
        ])
        .split(size);

    render_header(frame, chunks[0]);
    render_editor(frame, app, chunks[1]);
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
        .style(Style::default().bg(theme::BG_SECONDARY))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(theme::BORDER_DEFAULT))
                .style(Style::default().bg(theme::BG_SECONDARY)),
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

            // Apply bold
            if styled_char.style.bold {
                style = style.add_modifier(Modifier::BOLD);
            }

            // Apply dim (ratatui uses DIM modifier)
            if styled_char.style.dim_level > 0 {
                style = style.add_modifier(Modifier::DIM);
            }

            // Selection highlight
            if app.is_selected(i) {
                style = style.bg(theme::ACCENT_SECONDARY);
            }

            // Cursor position
            if i == app.cursor_pos && is_focused {
                style = style.bg(theme::ACCENT_PRIMARY).fg(theme::BG_PRIMARY);
            }

            spans.push(Span::styled(styled_char.ch.to_string(), style));
        }

        // Cursor at end of text
        if app.cursor_pos >= app.text.len() && is_focused {
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

    let title = format!(" Editor [{}] ", mode_indicator);

    let editor = Paragraph::new(Line::from(spans))
        .style(Style::default().bg(theme::BG_ELEVATED))
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
                .style(Style::default().bg(theme::BG_ELEVATED)),
        )
        .wrap(Wrap { trim: false });

    frame.render_widget(editor, area);
}

fn render_controls(frame: &mut Frame, app: &App, area: Rect) {
    // Split into three columns: FG color, BG color, Formatting
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(35),
            Constraint::Percentage(35),
            Constraint::Percentage(30),
        ])
        .split(area);

    render_color_picker(frame, app, chunks[0], "Foreground", true);
    render_color_picker(frame, app, chunks[1], "Background", false);
    render_formatting_panel(frame, app, chunks[2]);
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

    // Create color palette display (2 rows x 8 colors)
    let mut line1_spans: Vec<Span> = vec![Span::raw(" ")];
    let mut line2_spans: Vec<Span> = vec![Span::raw(" ")];

    for (i, (color, _name)) in COLOR_PALETTE.iter().enumerate() {
        let is_selected = i == selected_index;
        let is_current = *color == current_color;

        let display = if is_selected && is_focused {
            "▓▓"
        } else if is_current {
            "██"
        } else {
            "░░"
        };

        let style = Style::default().fg(*color);
        let span = Span::styled(display, style);

        if i < 8 {
            line1_spans.push(span);
        } else {
            line2_spans.push(span);
        }
    }

    let text = vec![Line::from(line1_spans), Line::from(line2_spans)];

    let picker = Paragraph::new(text)
        .style(Style::default().bg(theme::BG_SECONDARY))
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
                .style(Style::default().bg(theme::BG_SECONDARY)),
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

    // Bold indicator
    let bold_style = if app.current_bold {
        Style::default()
            .fg(theme::ACCENT_PRIMARY)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme::TEXT_MUTED)
    };
    let bold_span = Span::styled(
        if app.current_bold { " [B] Bold ✓ " } else { " [B] Bold   " },
        bold_style,
    );

    // Dim indicator with levels
    let dim_display = match app.current_dim {
        0 => "░░░░",
        1 => "▒░░░",
        2 => "▓▒░░",
        3 => "█▓▒░",
        _ => "░░░░",
    };
    let dim_span = Span::styled(
        format!(" [D] Dim {} ", dim_display),
        if app.current_dim > 0 {
            Style::default().fg(theme::ACCENT_SECONDARY)
        } else {
            Style::default().fg(theme::TEXT_MUTED)
        },
    );

    // Export button
    let export_span = Span::styled(
        " [E] Export ",
        Style::default().fg(theme::SUCCESS),
    );

    let lines = vec![
        Line::from(vec![bold_span, dim_span]),
        Line::from(vec![export_span]),
    ];

    let panel = Paragraph::new(lines)
        .style(Style::default().bg(theme::BG_SECONDARY))
        .block(
            Block::default()
                .title(Span::styled(
                    " Actions ",
                    Style::default()
                        .fg(if is_focused { theme::ACCENT_PRIMARY } else { theme::TEXT_SECONDARY })
                        .add_modifier(Modifier::BOLD),
                ))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(border_color))
                .style(Style::default().bg(theme::BG_SECONDARY)),
        );

    frame.render_widget(panel, area);
}

fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let help_text = match app.active_panel {
        Panel::Editor => match app.mode {
            Mode::Normal => "i:insert │ v:select │ e:export │ Tab:next panel │ Ctrl+Q:quit",
            Mode::Typing => "Esc:normal │ ←→:move │ Backspace:delete",
            Mode::Selecting => "←→:extend │ Enter:apply style │ Esc:cancel",
        },
        Panel::FgColor | Panel::BgColor => "←→↑↓:navigate │ Enter:select │ Tab:next │ Esc:editor",
        Panel::Formatting => "B:bold │ D:dim │ E:export │ Tab:next │ Esc:editor",
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
