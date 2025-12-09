use crate::app::{App, Mode, Panel};
use crate::colors::COLOR_PALETTE;
use crate::export::copy_to_clipboard;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Handle key events and update app state
pub fn handle_key_event(app: &mut App, key: KeyEvent) {
    // Global quit with Ctrl+C or Ctrl+Q
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        match key.code {
            KeyCode::Char('c') | KeyCode::Char('q') => {
                app.should_quit = true;
                return;
            }
            _ => {}
        }
    }

    match app.active_panel {
        Panel::Editor => handle_editor_input(app, key),
        Panel::FgColor => handle_color_picker_input(app, key, true),
        Panel::BgColor => handle_color_picker_input(app, key, false),
        Panel::Formatting => handle_formatting_input(app, key),
    }
}

fn handle_editor_input(app: &mut App, key: KeyEvent) {
    match app.mode {
        Mode::Normal | Mode::Typing => handle_normal_typing_input(app, key),
        Mode::Selecting => handle_selecting_input(app, key),
    }
}

fn handle_normal_typing_input(app: &mut App, key: KeyEvent) {
    match key.code {
        // Quit
        KeyCode::Char('q') if app.mode == Mode::Normal && app.text.is_empty() => {
            app.should_quit = true;
        }
        
        // Panel navigation
        KeyCode::Tab => {
            app.active_panel = app.active_panel.next();
            app.clear_status();
        }
        KeyCode::BackTab => {
            app.active_panel = app.active_panel.prev();
            app.clear_status();
        }

        // Cursor movement (vim-style and arrows)
        KeyCode::Left | KeyCode::Char('h') if app.mode == Mode::Normal => {
            app.move_left();
        }
        KeyCode::Right | KeyCode::Char('l') if app.mode == Mode::Normal => {
            app.move_right();
        }
        KeyCode::Home | KeyCode::Char('0') if app.mode == Mode::Normal => {
            app.move_to_start();
        }
        KeyCode::End | KeyCode::Char('$') if app.mode == Mode::Normal => {
            app.move_to_end();
        }

        // Arrow keys always work for movement
        KeyCode::Left => app.move_left(),
        KeyCode::Right => app.move_right(),
        KeyCode::Home => app.move_to_start(),
        KeyCode::End => app.move_to_end(),

        // Enter typing mode
        KeyCode::Char('i') if app.mode == Mode::Normal => {
            app.mode = Mode::Typing;
            app.set_status("-- INSERT --");
        }
        KeyCode::Char('a') if app.mode == Mode::Normal => {
            app.mode = Mode::Typing;
            app.move_right();
            app.set_status("-- INSERT --");
        }

        // Start selection
        KeyCode::Char('v') if app.mode == Mode::Normal => {
            app.start_selection();
            app.set_status("-- VISUAL --");
        }

        // Export
        KeyCode::Char('e') if app.mode == Mode::Normal => {
            match copy_to_clipboard(app) {
                Ok(_) => app.set_status("✓ Copied to clipboard!"),
                Err(e) => app.set_status(format!("✗ Copy failed: {}", e)),
            }
        }

        // Exit insert mode
        KeyCode::Esc => {
            app.mode = Mode::Normal;
            app.clear_selection();
            app.clear_status();
        }

        // Backspace
        KeyCode::Backspace => {
            app.delete_char();
        }

        // Delete
        KeyCode::Delete => {
            app.delete_char_forward();
        }

        // Type characters in typing mode
        KeyCode::Char(c) if app.mode == Mode::Typing => {
            app.insert_char(c);
        }

        _ => {}
    }
}

fn handle_selecting_input(app: &mut App, key: KeyEvent) {
    match key.code {
        // Movement extends selection
        KeyCode::Left | KeyCode::Char('h') => app.move_left(),
        KeyCode::Right | KeyCode::Char('l') => app.move_right(),
        KeyCode::Home | KeyCode::Char('0') => app.move_to_start(),
        KeyCode::End | KeyCode::Char('$') => app.move_to_end(),

        // Apply style to selection
        KeyCode::Enter => {
            app.apply_style();
            app.set_status("Style applied");
        }

        // Cancel selection
        KeyCode::Esc | KeyCode::Char('v') => {
            app.clear_selection();
            app.clear_status();
        }

        // Panel switch - apply style first
        KeyCode::Tab => {
            app.active_panel = app.active_panel.next();
        }
        KeyCode::BackTab => {
            app.active_panel = app.active_panel.prev();
        }

        _ => {}
    }
}

fn handle_color_picker_input(app: &mut App, key: KeyEvent, is_foreground: bool) {
    let color_index = if is_foreground {
        &mut app.fg_color_index
    } else {
        &mut app.bg_color_index
    };

    match key.code {
        // Navigate colors
        KeyCode::Left | KeyCode::Char('h') => {
            if *color_index > 0 {
                *color_index -= 1;
            }
        }
        KeyCode::Right | KeyCode::Char('l') => {
            if *color_index < COLOR_PALETTE.len() - 1 {
                *color_index += 1;
            }
        }
        KeyCode::Up | KeyCode::Char('k') => {
            if *color_index >= 8 {
                *color_index -= 8;
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if *color_index + 8 < COLOR_PALETTE.len() {
                *color_index += 8;
            }
        }

        // Select color and apply
        KeyCode::Enter => {
            let (color, name) = COLOR_PALETTE[*color_index];
            if is_foreground {
                app.current_fg = color;
                app.set_status(format!("FG: {}", name));
            } else {
                app.current_bg = color;
                app.set_status(format!("BG: {}", name));
            }
            app.apply_style();
        }

        // Panel navigation
        KeyCode::Tab => {
            app.active_panel = app.active_panel.next();
            app.clear_status();
        }
        KeyCode::BackTab => {
            app.active_panel = app.active_panel.prev();
            app.clear_status();
        }

        KeyCode::Esc => {
            app.active_panel = Panel::Editor;
            app.clear_status();
        }

        _ => {}
    }
}

fn handle_formatting_input(app: &mut App, key: KeyEvent) {
    match key.code {
        // Toggle bold
        KeyCode::Char('b') | KeyCode::Char('B') => {
            app.toggle_bold();
            app.set_status(if app.current_bold { "Bold: ON" } else { "Bold: OFF" });
        }

        // Cycle dim
        KeyCode::Char('d') | KeyCode::Char('D') => {
            app.cycle_dim();
            app.set_status(format!("Dim level: {}", app.current_dim));
        }

        // Export shortcut
        KeyCode::Char('e') | KeyCode::Char('E') => {
            match copy_to_clipboard(app) {
                Ok(_) => app.set_status("✓ Copied to clipboard!"),
                Err(e) => app.set_status(format!("✗ Copy failed: {}", e)),
            }
        }

        // Panel navigation
        KeyCode::Tab => {
            app.active_panel = app.active_panel.next();
            app.clear_status();
        }
        KeyCode::BackTab => {
            app.active_panel = app.active_panel.prev();
            app.clear_status();
        }

        KeyCode::Esc => {
            app.active_panel = Panel::Editor;
            app.clear_status();
        }

        _ => {}
    }
}
