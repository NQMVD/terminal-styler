# Terminal Text Styler

An interactive terminal text styling tool built with Ratatui in Rust.

## Features

- **Text Editing**: Type and edit text directly in the terminal
- **Color Styling**: Apply foreground and background colors to text
- **Text Formatting**: Toggle bold formatting and adjust dimness levels
- **Text Selection**: Select ranges of text for styling
- **Real-time Preview**: See styled text preview alongside the editor
- **User-friendly Interface**: Help overlay with keyboard shortcuts
- **ANSI Export**: Generate echo commands with ANSI escape codes to reproduce styled text anywhere

## Controls

### Normal Mode
- `e`: Enter edit mode
- `c`: Enter color selection mode  
- `x`: Export styled text (ANSI escape codes)
- `h`: Toggle help overlay
- `←`/`→`: Move cursor
- `Space`: Start text selection
- `q`: Quit application

### Edit Mode
- `ESC`: Exit edit mode
- `←`/`→`: Move cursor
- `Backspace`/`Delete`: Delete characters
- Any character: Insert text

### Color Selection Mode
- `f`: Cycle foreground color
- `b`: Cycle background color
- `B`: Toggle bold formatting
- `d`: Increase dim level
- `a`: Apply style to selection
- `ESC`: Cancel and return to normal mode

### Export Mode
- `c`: Copy ANSI escape command to clipboard (uses clipboard crate)
- `s`: Show export options in console
- `ESC`: Return to normal mode

## Installation

1. Ensure you have Rust installed: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
2. Clone this repository
3. Run `cargo build --release`
4. Run the application with `cargo run`

## Usage

1. Launch the application
2. Press `e` to enter edit mode and type your text
3. Use arrow keys to navigate
4. Press `Space` to start a selection
5. Press `c` to enter color selection mode
6. Use `f`/`b` to cycle colors, `B` for bold, `d` for dim
7. Press `a` to apply styles to your selection
8. See real-time preview in the right panel
9. Press `x` to export your styled text as ANSI escape codes
10. Press `q` to quit

## Technical Details

- Built with Ratatui 0.26.0, Crossterm 0.27.0, and Clipboard 0.5.0
- Uses Rust's ownership system for efficient text handling
- Implements proper terminal cleanup on exit
- Includes comprehensive unit tests
- Cross-platform clipboard support for easy copying of export commands

## Future Enhancements

- Save/load styled text to/from files
- More color options and custom color selection
- Additional text formatting options (italic, underline)
- Copy styled text to clipboard
- Export to HTML or other formats