mod app;
mod colors;
mod export;
mod fx;
mod import;
mod input;
mod mouse;
mod ui;

use std::io;
use std::panic;
use std::time::{Duration, Instant};

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyEventKind, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::backend::CrosstermBackend;
use ratatui::{Terminal, layout::Rect};

use app::App;
use fx::FxManager;
use input::handle_key_event;
use mouse::handle_mouse_event;

const FPS: usize = 60;

fn main() -> Result<()> {
    // Set up panic hook to restore terminal on crash
    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        let _ = restore_terminal();
        original_hook(panic_info);
    }));

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    // Run the app
    let result = run_app(&mut terminal);

    // Restore terminal
    restore_terminal()?;

    result
}

fn restore_terminal() -> Result<()> {
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;
    Ok(())
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    let mut app = App::new();
    let mut fx_manager = FxManager::new();
    
    // Trigger startup animation
    fx_manager.trigger_startup();
    
    let mut last_frame = Instant::now();

    loop {
        let elapsed = last_frame.elapsed();
        last_frame = Instant::now();

        // Draw UI with effects
        terminal.draw(|frame| {
            ui::render(frame, &app);
            fx_manager.render(frame, frame.area(), elapsed.into());
        })?;

        // Handle events (60 FPS timing)
        if event::poll(Duration::from_millis(1000 / FPS as u64))? {
            match event::read()? {
                Event::Key(key) => {
                    // Only handle key press events (not release or repeat)
                    if key.kind == KeyEventKind::Press {
                        handle_key_event(&mut app, key);
                    }
                }
                Event::Mouse(mouse_event) => {
                    // Get terminal area for coordinate mapping
                    let size = terminal.size().unwrap_or_else(|_| ratatui::layout::Size { width: 80, height: 24 });
                    let terminal_area = Rect { x: 0, y: 0, width: size.width, height: size.height };
                    handle_mouse_event(&mut app, mouse_event, terminal_area);
                }
                _ => {}
            }
        }

        // Check if we should quit
        if app.should_quit {
            break;
        }
    }

    Ok(())
}

