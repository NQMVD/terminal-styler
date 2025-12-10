mod app;
mod colors;
mod export;
mod fx;
mod import;
mod input;
mod ui;

use std::io;
use std::panic;
use std::time::{Duration, Instant};

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use app::App;
use fx::FxManager;
use input::handle_key_event;

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
    execute!(stdout, EnterAlternateScreen)?;
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
            if let Event::Key(key) = event::read()? {
                // Only handle key press events (not release or repeat)
                if key.kind == KeyEventKind::Press {
                    handle_key_event(&mut app, key);
                }
            }
        }

        // Check if we should quit
        if app.should_quit {
            break;
        }
    }

    Ok(())
}

