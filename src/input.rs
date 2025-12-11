use crate::app::{App, Mode, Panel};
use crate::colors::{color_index_from_key, COLOR_PALETTE};
use crate::export::copy_to_clipboard;
use crate::import::{export_ron_to_clipboard, import_from_clipboard};
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
            KeyCode::Char('h') => {
                app.toggle_selection_highlight_mode();
                let mode_name = match app.selection_highlight_mode {
                    crate::app::SelectionHighlightMode::Reversed => "Reversed",
                    crate::app::SelectionHighlightMode::Underline => "Underline",
                };
                app.set_status(format!("Selection highlight: {}", mode_name));
                return;
            }
            KeyCode::Char('i') => {
                // Import from clipboard (auto-detect ANSI vs RON)
                match import_from_clipboard(app) {
                    Ok(msg) => app.set_status(format!("✓ {}", msg)),
                    Err(e) => app.set_status(format!("✗ Import failed: {}", e)),
                }
                return;
            }
            KeyCode::Char('e') => {
                // Export to RON format
                match export_ron_to_clipboard(app) {
                    Ok(_) => app.set_status("✓ Copied RON to clipboard!"),
                    Err(e) => app.set_status(format!("✗ RON export failed: {}", e)),
                }
                return;
            }
            _ => {}
        }
    }

    // Global panel shortcuts (f/b/d/r) when not in typing mode
    if app.mode != Mode::Typing {
        match key.code {
            KeyCode::Char('f') | KeyCode::Char('F') => {
                app.active_panel = Panel::FgColor;
                app.set_status("Foreground color");
                return;
            }
            KeyCode::Char('g') | KeyCode::Char('G') => {
                app.active_panel = Panel::BgColor;
                app.set_status("Background color");
                return;
            }
            KeyCode::Char('d') | KeyCode::Char('D') => {
                app.active_panel = Panel::Formatting;
                app.set_status("Decorations");
                return;
            }
            KeyCode::Char('r') | KeyCode::Char('R') => {
                app.reset_style();
                if app.selection.is_some() {
                    app.apply_style();
                    app.set_status("Reset style applied");
                } else {
                    app.set_status("Style reset");
                }
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
        KeyCode::Up | KeyCode::Char('k') if app.mode == Mode::Normal => {
            app.move_up();
        }
        KeyCode::Down | KeyCode::Char('j') if app.mode == Mode::Normal => {
            app.move_down();
        }
        KeyCode::Home | KeyCode::Char('0') if app.mode == Mode::Normal => {
            app.move_to_line_start();
        }
        KeyCode::End | KeyCode::Char('$') if app.mode == Mode::Normal => {
            app.move_to_line_end();
        }

        // Arrow keys always work for movement
        KeyCode::Left => app.move_left(),
        KeyCode::Right => app.move_right(),
        KeyCode::Up => app.move_up(),
        KeyCode::Down => app.move_down(),
        KeyCode::Home => app.move_to_line_start(),
        KeyCode::End => app.move_to_line_end(),

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

        // Start selection - load character style into panels
        KeyCode::Char('v') if app.mode == Mode::Normal => {
            app.load_style_from_cursor();
            app.start_selection();
            app.set_status("-- VISUAL --");
        }

        // Paste (yank buffer)
        KeyCode::Char('p') if app.mode == Mode::Normal => {
            app.paste();
            app.set_status("Pasted");
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

        // Enter key inserts newline in typing mode
        KeyCode::Enter if app.mode == Mode::Typing => {
            app.insert_char('\n');
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
        KeyCode::Up | KeyCode::Char('k') => app.move_up(),
        KeyCode::Down | KeyCode::Char('j') => app.move_down(),
        KeyCode::Home | KeyCode::Char('0') => app.move_to_line_start(),
        KeyCode::End | KeyCode::Char('$') => app.move_to_line_end(),

        // Yank (copy) selection
        KeyCode::Char('y') => {
            app.yank();
            app.set_status("Yanked");
            app.clear_selection();
        }

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
        // Number/letter key selection (0-9, a-g)
        KeyCode::Char(c) if color_index_from_key(c).is_some() => {
            if let Some(idx) = color_index_from_key(c) {
                *color_index = idx;
                let (color, name, _) = COLOR_PALETTE[idx];
                if is_foreground {
                    app.current_fg = color;
                    app.set_status(format!("FG: {}", name));
                } else {
                    app.current_bg = color;
                    app.set_status(format!("BG: {}", name));
                }
                app.apply_style();
            }
        }

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
            if *color_index >= 9 {
                *color_index -= 9;
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if *color_index + 9 < COLOR_PALETTE.len() {
                *color_index += 9;
            }
        }

        // Select color and apply
        KeyCode::Enter => {
            let (color, name, _) = COLOR_PALETTE[*color_index];
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
        KeyCode::Char('b') | KeyCode::Char('B') | KeyCode::Char('1') => {
            app.toggle_bold();
            app.set_status(if app.current_bold { "Bold: ON" } else { "Bold: OFF" });
        }

        // Toggle italic
        KeyCode::Char('i') | KeyCode::Char('I') | KeyCode::Char('2') => {
            app.toggle_italic();
            app.set_status(if app.current_italic { "Italic: ON" } else { "Italic: OFF" });
        }

        // Toggle underline
        KeyCode::Char('u') | KeyCode::Char('U') | KeyCode::Char('3') => {
            app.toggle_underline();
            app.set_status(if app.current_underline { "Underline: ON" } else { "Underline: OFF" });
        }

        // Toggle strikethrough
        KeyCode::Char('s') | KeyCode::Char('S') | KeyCode::Char('4') => {
            app.toggle_strikethrough();
            app.set_status(if app.current_strikethrough { "Strikethrough: ON" } else { "Strikethrough: OFF" });
        }

        // Cycle dim
        KeyCode::Char('m') | KeyCode::Char('M') | KeyCode::Char('5') => {
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
