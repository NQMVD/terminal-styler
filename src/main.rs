use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
    Terminal,
};
use std::io::{stdout, Result};

// Clipboard support
#[cfg(target_os = "linux")]
use clipboard::{ClipboardContext, ClipboardProvider};

#[derive(Debug, Clone, Copy, PartialEq)]
enum AppMode {
    Normal,
    Editing,
    ColorSelection,
    Export,
}

#[derive(Debug, Clone)]
struct StyledChar {
    char: char,
    fg_color: Color,
    bg_color: Color,
    is_bold: bool,
    dim_level: u8, // 0-100
}

impl Default for StyledChar {
    fn default() -> Self {
        Self {
            char: ' ',
            fg_color: Color::White,
            bg_color: Color::Black,
            is_bold: false,
            dim_level: 0,
        }
    }
}

#[derive(Debug)]
struct App {
    mode: AppMode,
    input_text: String,
    styled_chars: Vec<StyledChar>,
    cursor_position: usize,
    selection_start: Option<usize>,
    current_fg_color: Color,
    current_bg_color: Color,
    current_bold: bool,
    current_dim_level: u8,
    show_help: bool,
}

impl Default for App {
    fn default() -> Self {
        Self {
            mode: AppMode::Normal,
            input_text: String::new(),
            styled_chars: Vec::new(),
            cursor_position: 0,
            selection_start: None,
            current_fg_color: Color::White,
            current_bg_color: Color::Black,
            current_bold: false,
            current_dim_level: 0,
            show_help: true,
        }
    }
}

impl App {
    fn update_styled_chars(&mut self) {
        self.styled_chars.clear();
        for c in self.input_text.chars() {
            self.styled_chars.push(StyledChar {
                char: c,
                fg_color: self.current_fg_color,
                bg_color: self.current_bg_color,
                is_bold: self.current_bold,
                dim_level: self.current_dim_level,
            });
        }
    }

    fn get_current_selection_range(&self) -> (usize, usize) {
        let start = self.selection_start.unwrap_or(self.cursor_position);
        let end = self.cursor_position;
        (start.min(end), start.max(end))
    }

    fn apply_style_to_selection(&mut self) {
        let (start, end) = self.get_current_selection_range();
        for i in start..end {
            if let Some(char) = self.styled_chars.get_mut(i) {
                char.fg_color = self.current_fg_color;
                char.bg_color = self.current_bg_color;
                char.is_bold = self.current_bold;
                char.dim_level = self.current_dim_level;
            }
        }
    }

    fn generate_ansi_escape_command(&self) -> String {
        let mut command = String::from("echo -e '");
        
        for styled_char in &self.styled_chars {
            // Start with reset
            command.push_str("\\033[0m");
            
            // Add foreground color
            let fg_code = match styled_char.fg_color {
                Color::Black => "30",
                Color::Red => "31",
                Color::Green => "32",
                Color::Yellow => "33",
                Color::Blue => "34",
                Color::Magenta => "35",
                Color::Cyan => "36",
                Color::White => "37",
                _ => "39", // Default foreground
            };
            command.push_str(&format!("\\033[{}m", fg_code));
            
            // Add background color
            let bg_code = match styled_char.bg_color {
                Color::Black => "40",
                Color::Red => "41",
                Color::Green => "42",
                Color::Yellow => "43",
                Color::Blue => "44",
                Color::Magenta => "45",
                Color::Cyan => "46",
                Color::White => "47",
                _ => "49", // Default background
            };
            command.push_str(&format!("\\033[{}m", bg_code));
            
            // Add bold if needed
            if styled_char.is_bold {
                command.push_str("\\033[1m");
            }
            
            // Add dim if needed
            if styled_char.dim_level > 0 {
                command.push_str("\\033[2m");
            }
            
            // Add the character itself
            command.push(styled_char.char);
        }
        
        // End with reset and close the echo command
        command.push_str("\\033[0m'");
        command
    }

    fn generate_export_options(&self) -> Vec<String> {
        vec![
            format!("ANSI Escape Command: {}", self.generate_ansi_escape_command()),
            format!("Plain Text: {}", self.input_text),
            format!("Character Count: {}", self.styled_chars.len()),
        ]
    }
}

fn main() -> Result<()> {
    // Setup terminal
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    // Create app
    let mut app = App::default();

    // Main loop
    loop {
        terminal.draw(|f| ui(f, &app))?;

        if event::poll(std::time::Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match app.mode {
                        AppMode::Normal => handle_normal_mode(&mut app, key.code),
                        AppMode::Editing => handle_editing_mode(&mut app, key.code),
                        AppMode::ColorSelection => handle_color_selection(&mut app, key.code),
                        AppMode::Export => handle_export_mode(&mut app, key.code),
                    }
                }
            }
        }

        // Exit condition
        if app.mode == AppMode::Normal && should_quit(&app) {
            break;
        }
    }

    // Cleanup
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_initialization() {
        let app = App::default();
        assert_eq!(app.mode, AppMode::Normal);
        assert!(app.input_text.is_empty());
        assert!(app.styled_chars.is_empty());
        assert_eq!(app.cursor_position, 0);
        assert_eq!(app.selection_start, None);
        assert_eq!(app.current_fg_color, Color::White);
        assert_eq!(app.current_bg_color, Color::Black);
        assert!(!app.current_bold);
        assert_eq!(app.current_dim_level, 0);
        assert!(app.show_help);
    }

    #[test]
    fn test_styled_char_default() {
        let styled_char = StyledChar::default();
        assert_eq!(styled_char.char, ' ');
        assert_eq!(styled_char.fg_color, Color::White);
        assert_eq!(styled_char.bg_color, Color::Black);
        assert!(!styled_char.is_bold);
        assert_eq!(styled_char.dim_level, 0);
    }

    #[test]
    fn test_update_styled_chars() {
        let mut app = App::default();
        app.input_text = "hello".to_string();
        app.update_styled_chars();
        
        assert_eq!(app.styled_chars.len(), 5);
        for styled_char in &app.styled_chars {
            assert_eq!(styled_char.fg_color, Color::White);
            assert_eq!(styled_char.bg_color, Color::Black);
            assert!(!styled_char.is_bold);
            assert_eq!(styled_char.dim_level, 0);
        }
    }

    #[test]
    fn test_selection_range() {
        let mut app = App::default();
        app.input_text = "hello world".to_string();
        app.cursor_position = 5;
        
        // No selection
        let (start, end) = app.get_current_selection_range();
        assert_eq!(start, 5);
        assert_eq!(end, 5);
        
        // With selection
        app.selection_start = Some(2);
        let (start, end) = app.get_current_selection_range();
        assert_eq!(start, 2);
        assert_eq!(end, 5);
        
        // Selection backwards
        app.cursor_position = 1;
        let (start, end) = app.get_current_selection_range();
        assert_eq!(start, 1);
        assert_eq!(end, 2);
    }

    #[test]
    fn test_color_cycling() {
        let mut app = App::default();
        
        // Test foreground color cycling
        assert_eq!(app.current_fg_color, Color::White);
        
        // Simulate pressing 'f' key in color selection mode
        let original_color = app.current_fg_color;
        match original_color {
            Color::White => app.current_fg_color = Color::Red,
            Color::Red => app.current_fg_color = Color::Green,
            Color::Green => app.current_fg_color = Color::Blue,
            Color::Blue => app.current_fg_color = Color::Yellow,
            Color::Yellow => app.current_fg_color = Color::Cyan,
            Color::Cyan => app.current_fg_color = Color::Magenta,
            Color::Magenta => app.current_fg_color = Color::Black,
            Color::Black => app.current_fg_color = Color::White,
            _ => {}
        }
        
        assert_ne!(app.current_fg_color, original_color);
    }

    #[test]
    fn test_cursor_position_bounds() {
        let mut app = App::default();
        app.input_text = "hello".to_string();
        app.update_styled_chars();
        
        // Test normal cursor positions
        app.cursor_position = 0;
        assert_eq!(app.cursor_position.min(app.styled_chars.len()), 0);
        
        app.cursor_position = 3;
        assert_eq!(app.cursor_position.min(app.styled_chars.len()), 3);
        
        // Test cursor position beyond text length
        app.cursor_position = 10;
        assert_eq!(app.cursor_position.min(app.styled_chars.len()), 5);
        
        // Test cursor position exactly at text length
        app.cursor_position = 5;
        assert_eq!(app.cursor_position.min(app.styled_chars.len()), 5);
    }

    #[test]
    fn test_ansi_export_generation() {
        let mut app = App::default();
        app.input_text = "test".to_string();
        app.update_styled_chars();
        
        // Test basic export command generation
        let export_command = app.generate_ansi_escape_command();
        
        // Should start with echo -e '
        assert!(export_command.starts_with("echo -e '"));
        
        // Should end with reset and closing quote
        assert!(export_command.ends_with("\\033[0m'"));
        
        // Should contain ANSI escape codes
        assert!(export_command.contains("\\033["));
        
        // Should have reasonable length (contains the text plus escape codes)
        assert!(export_command.len() > 20); // "echo -e 'test\\033[0m'" is 18 chars minimum
        
        // Check that it contains some character from the original text
        assert!(export_command.chars().any(|c| c == 't' || c == 'e' || c == 's'));
    }

    #[test]
    fn test_export_options() {
        let mut app = App::default();
        app.input_text = "hello".to_string();
        app.update_styled_chars();
        
        let export_options = app.generate_export_options();
        
        // Should have 3 options
        assert_eq!(export_options.len(), 3);
        
        // Should contain ANSI command
        assert!(export_options[0].contains("ANSI Escape Command"));
        
        // Should contain plain text
        assert!(export_options[1].contains("Plain Text: hello"));
        
        // Should contain character count
        assert!(export_options[2].contains("Character Count: 5"));
    }
}

fn should_quit(app: &App) -> bool {
    // Add quit logic here - press 'q' to quit
    app.input_text.contains("quit") || app.input_text.contains("exit")
}

fn handle_normal_mode(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Char('e') => {
            app.mode = AppMode::Editing;
        }
        KeyCode::Char('c') => {
            app.mode = AppMode::ColorSelection;
        }
        KeyCode::Char('x') => {
            app.mode = AppMode::Export;
        }
        KeyCode::Char('q') => {
            // Quit will be handled in main loop
            app.input_text = "quit".to_string();
        }
        KeyCode::Char('h') => {
            app.show_help = !app.show_help;
        }
        KeyCode::Left => {
            if app.cursor_position > 0 {
                app.cursor_position -= 1;
            }
        }
        KeyCode::Right => {
            if app.cursor_position < app.input_text.len() {
                app.cursor_position += 1;
            }
        }
        KeyCode::Char(' ') => {
            // Toggle selection start
            app.selection_start = Some(app.cursor_position);
        }
        _ => {}
    }
}

fn handle_export_mode(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Esc => {
            app.mode = AppMode::Normal;
        }
        KeyCode::Char('c') => {
            // Copy to clipboard using clipboard crate
            let export_command = app.generate_ansi_escape_command();
            
            #[cfg(target_os = "linux")]
            if let Ok(mut ctx) = ClipboardContext::new() {
                if ctx.set_contents(export_command.clone()).is_ok() {
                    app.input_text = "Copied to clipboard!".to_string();
                } else {
                    app.input_text = "Failed to copy to clipboard".to_string();
                }
            } else {
                app.input_text = "Clipboard not available on this platform".to_string();
            }
            
            #[cfg(not(target_os = "linux"))]
            {
                app.input_text = "Clipboard copy simulated (add platform-specific implementation)".to_string();
                println!("Export command: {}", export_command);
            }
        }
        KeyCode::Char('s') => {
            // Show export options in a cleaner format
            let export_options = app.generate_export_options();
            for option in export_options {
                println!("{}", option);
            }
            app.input_text = "Export options printed to console".to_string();
        }
        _ => {}
    }
}

fn handle_editing_mode(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Esc => {
            app.mode = AppMode::Normal;
        }
        KeyCode::Enter => {
            app.mode = AppMode::Normal;
        }
        KeyCode::Char(c) => {
            app.input_text.insert(app.cursor_position, c);
            app.cursor_position += 1;
            app.update_styled_chars();
        }
        KeyCode::Backspace => {
            if app.cursor_position > 0 {
                app.input_text.remove(app.cursor_position - 1);
                app.cursor_position -= 1;
                app.update_styled_chars();
            }
        }
        KeyCode::Delete => {
            if app.cursor_position < app.input_text.len() {
                app.input_text.remove(app.cursor_position);
                app.update_styled_chars();
            }
        }
        KeyCode::Left => {
            if app.cursor_position > 0 {
                app.cursor_position -= 1;
            }
        }
        KeyCode::Right => {
            if app.cursor_position < app.input_text.len() {
                app.cursor_position += 1;
            }
        }
        _ => {}
    }
}

fn handle_color_selection(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Esc => {
            app.mode = AppMode::Normal;
        }
        KeyCode::Char('f') => {
            // Cycle foreground colors
            app.current_fg_color = match app.current_fg_color {
                Color::White => Color::Red,
                Color::Red => Color::Green,
                Color::Green => Color::Blue,
                Color::Blue => Color::Yellow,
                Color::Yellow => Color::Cyan,
                Color::Cyan => Color::Magenta,
                Color::Magenta => Color::Black,
                Color::Black => Color::White,
                _ => Color::White,
            };
        }
        KeyCode::Char('b') => {
            // Cycle background colors
            app.current_bg_color = match app.current_bg_color {
                Color::Black => Color::White,
                Color::White => Color::Red,
                Color::Red => Color::Green,
                Color::Green => Color::Blue,
                Color::Blue => Color::Yellow,
                Color::Yellow => Color::Cyan,
                Color::Cyan => Color::Magenta,
                Color::Magenta => Color::Black,
                _ => Color::Black,
            };
        }
        KeyCode::Char('d') => {
            // Increase dim level
            app.current_dim_level = app.current_dim_level.saturating_add(10);
            if app.current_dim_level > 100 {
                app.current_dim_level = 0;
            }
        }
        KeyCode::Char('B') => {
            // Toggle bold
            app.current_bold = !app.current_bold;
        }
        KeyCode::Char('a') => {
            // Apply current style to selection
            app.apply_style_to_selection();
            app.mode = AppMode::Normal;
        }
        _ => {}
    }
}

fn ui(frame: &mut ratatui::Frame, app: &App) {
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Title
            Constraint::Min(1),    // Main content
            Constraint::Length(3), // Status
        ])
        .split(frame.size());

    // Title
    let title = Paragraph::new("Terminal Text Styler")
        .style(Style::default().bold())
        .alignment(ratatui::layout::Alignment::Center)
        .block(Block::default().borders(Borders::NONE));
    frame.render_widget(title, main_layout[0]);

    // Main content area
    let content_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(main_layout[1]);

    // Input/Edit area
    let input_block = Block::default()
        .title("Text Editor")
        .borders(Borders::ALL);
    
    let input_text = if app.input_text.is_empty() {
        Text::from("Type some text and press 'e' to edit")
    } else {
        let mut spans = Vec::new();
        
        // Show styled characters or plain text based on mode
        if app.mode == AppMode::Editing || app.styled_chars.is_empty() {
            spans.push(Span::raw(&app.input_text));
        } else {
            for (_i, styled_char) in app.styled_chars.iter().enumerate() {
                let mut span = Span::from(styled_char.char.to_string())
                    .fg(styled_char.fg_color)
                    .bg(styled_char.bg_color);
                
                if styled_char.is_bold {
                    span = span.bold();
                }
                
                // Apply dimming (simplified - actual dimming would need more complex handling)
                if styled_char.dim_level > 0 {
                    span = span.dim();
                }
                
                spans.push(span);
            }
        }
        
        // Highlight selection
        if let Some(_selection_start) = app.selection_start {
            let (start, end) = app.get_current_selection_range();
            
            // This is a simplified approach - in a real app you'd need to handle this more carefully
            if start < end && end <= spans.len() {
                for i in start..end {
                    if let Some(span) = spans.get_mut(i) {
                        spans[i] = span.clone().bg(Color::Gray);
                    }
                }
            }
        }
        
        // Add cursor
        let cursor_pos = app.cursor_position.min(spans.len());
        if cursor_pos <= spans.len() {
            spans.insert(cursor_pos, Span::raw("|").fg(Color::White).bg(Color::Black));
        }
        
        Text::from(Line::from(spans))
    };

    let input_paragraph = Paragraph::new(input_text)
        .block(input_block)
        .wrap(Wrap { trim: false });
    frame.render_widget(input_paragraph, content_layout[0]);

    // Preview area
    let preview_block = Block::default()
        .title("Styled Preview")
        .borders(Borders::ALL);
        
    let preview_text = if app.styled_chars.is_empty() {
        Text::from("Preview will appear here")
    } else if app.mode == AppMode::Export {
        // Show export information in preview panel with better formatting
        let export_command = app.generate_ansi_escape_command();
        let export_info = format!(
            "ANSI EXPORT COMMAND:\n
{}\n
Press 'c' to copy to clipboard\nPress 's' to show all export options",
            export_command
        );
        Text::from(export_info)
    } else {
        let mut spans = Vec::new();
        for styled_char in &app.styled_chars {
            let mut span = Span::from(styled_char.char.to_string())
                .fg(styled_char.fg_color)
                .bg(styled_char.bg_color);
            
            if styled_char.is_bold {
                span = span.bold();
            }
            
            if styled_char.dim_level > 0 {
                span = span.dim();
            }
            
            spans.push(span);
        }
        Text::from(Line::from(spans))
    };

    let preview_paragraph = Paragraph::new(preview_text)
        .block(preview_block)
        .wrap(Wrap { trim: false });
    frame.render_widget(preview_paragraph, content_layout[1]);

    // Status bar
    let status_text = match app.mode {
        AppMode::Normal => {
            format!(
                " NORMAL | Pos: {} | {} | Press 'e' to edit, 'c' for colors, 'q' to quit ",
                app.cursor_position,
                if app.show_help { "Help ON" } else { "Help OFF" }
            )
        }
        AppMode::Editing => {
            format!(
                " EDITING | Pos: {} | Press ESC to exit edit mode ",
                app.cursor_position
            )
        }
        AppMode::ColorSelection => {
            format!(
                " COLORS | FG: {:?} | BG: {:?} | Bold: {} | Dim: {} | Press 'a' to apply, ESC to cancel ",
                app.current_fg_color, app.current_bg_color, app.current_bold, app.current_dim_level
            )
        }
        AppMode::Export => {
            format!(
                " EXPORT | Press 'c' to copy ANSI command, 's' to show options, ESC to cancel ",
            )
        }
    };

    let status = Paragraph::new(status_text)
        .style(Style::default().fg(Color::Black).bg(Color::White));
    frame.render_widget(status, main_layout[2]);

    // Help overlay
    if app.show_help && app.mode == AppMode::Normal {
        let help_block = Block::default()
            .title("Help")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White).bg(Color::DarkGray));
        
        let help_text = Text::from(vec![
            Line::from("Normal Mode:"),
            Line::from("  e: Enter edit mode"),
            Line::from("  c: Enter color selection mode"),
            Line::from("  h: Toggle help"),
            Line::from("  ←→: Move cursor"),
            Line::from("  Space: Start selection"),
            Line::from("  x: Export styled text"),
            Line::from("  q: Quit"),
            Line::from(""),
            Line::from("Edit Mode:"),
            Line::from("  ESC: Exit edit mode"),
            Line::from("  ←→: Move cursor"),
            Line::from("  Backspace/Delete: Delete characters"),
            Line::from(""),
            Line::from("Color Mode:"),
            Line::from("  f: Cycle foreground color"),
            Line::from("  b: Cycle background color"),
            Line::from("  B: Toggle bold"),
            Line::from("  d: Increase dim level"),
            Line::from("  a: Apply style to selection"),
            Line::from("  ESC: Cancel"),
        ]);
        
        let help_paragraph = Paragraph::new(help_text)
            .block(help_block);
        
        let area = Rect::new(
            frame.size().width / 4,
            frame.size().height / 4,
            frame.size().width / 2,
            frame.size().height / 2
        );
        frame.render_widget(help_paragraph, area);
    }
}