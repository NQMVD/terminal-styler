use ratatui::style::Color;

/// Represents styling for a single character
#[derive(Clone, Debug, PartialEq)]
pub struct CharStyle {
    pub fg: Color,
    pub bg: Color,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strikethrough: bool,
    pub dim_level: u8, // 0-3: 0 = none, 1-3 = increasing dimness
}

impl Default for CharStyle {
    fn default() -> Self {
        Self {
            fg: Color::Reset,
            bg: Color::Reset,
            bold: false,
            italic: false,
            underline: false,
            strikethrough: false,
            dim_level: 0,
        }
    }
}

/// A single character with its styling
#[derive(Clone, Debug)]
pub struct StyledChar {
    pub ch: char,
    pub style: CharStyle,
}

impl StyledChar {
    pub fn new(ch: char) -> Self {
        Self {
            ch,
            style: CharStyle::default(),
        }
    }

    pub fn with_style(ch: char, style: CharStyle) -> Self {
        Self { ch, style }
    }
}

/// Current input/interaction mode
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Typing,
    Selecting,
}

/// Which panel is currently focused
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Panel {
    Editor,
    FgColor,
    BgColor,
    Formatting,
}

impl Panel {
    pub fn next(&self) -> Self {
        match self {
            Panel::Editor => Panel::FgColor,
            Panel::FgColor => Panel::BgColor,
            Panel::BgColor => Panel::Formatting,
            Panel::Formatting => Panel::Editor,
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            Panel::Editor => Panel::Formatting,
            Panel::FgColor => Panel::Editor,
            Panel::BgColor => Panel::FgColor,
            Panel::Formatting => Panel::BgColor,
        }
    }
}

/// How to display selection highlighting
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum SelectionHighlightMode {
    /// Reversed colors on selected text
    #[default]
    Reversed,
    /// Underline-style: dashes on separate line, plus for cursor
    Underline,
}

/// Main application state
pub struct App {
    /// The styled text buffer
    pub text: Vec<StyledChar>,
    /// Current cursor position
    pub cursor_pos: usize,
    /// Optional selection range (start, end) - inclusive
    pub selection: Option<(usize, usize)>,
    /// Selection anchor when in selecting mode
    pub selection_anchor: Option<usize>,
    /// Currently selected foreground color
    pub current_fg: Color,
    /// Currently selected background color  
    pub current_bg: Color,
    /// Bold toggle
    pub current_bold: bool,
    /// Italic toggle
    pub current_italic: bool,
    /// Underline toggle
    pub current_underline: bool,
    /// Strikethrough toggle
    pub current_strikethrough: bool,
    /// Dim level (0-3)
    pub current_dim: u8,
    /// Current input mode
    pub mode: Mode,
    /// Currently focused panel
    pub active_panel: Panel,
    /// Color picker index for foreground
    pub fg_color_index: usize,
    /// Color picker index for background
    pub bg_color_index: usize,
    /// Status message to display
    pub status_message: Option<String>,
    /// Should the app quit?
    pub should_quit: bool,
    /// Selection highlight display mode
    pub selection_highlight_mode: SelectionHighlightMode,
}

impl Default for App {
    fn default() -> Self {
        Self {
            text: Vec::new(),
            cursor_pos: 0,
            selection: None,
            selection_anchor: None,
            current_fg: Color::Reset,
            current_bg: Color::Reset,
            current_bold: false,
            current_italic: false,
            current_underline: false,
            current_strikethrough: false,
            current_dim: 0,
            mode: Mode::Normal,
            active_panel: Panel::Editor,
            fg_color_index: 0, // None/Reset
            bg_color_index: 0, // None/Reset
            status_message: None,
            should_quit: false,
            selection_highlight_mode: SelectionHighlightMode::default(),
        }
    }
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert a character at the cursor position
    pub fn insert_char(&mut self, ch: char) {
        let styled = StyledChar::with_style(
            ch,
            CharStyle {
                fg: self.current_fg,
                bg: self.current_bg,
                bold: self.current_bold,
                italic: self.current_italic,
                underline: self.current_underline,
                strikethrough: self.current_strikethrough,
                dim_level: self.current_dim,
            },
        );

        if self.cursor_pos >= self.text.len() {
            self.text.push(styled);
        } else {
            self.text.insert(self.cursor_pos, styled);
        }
        self.cursor_pos += 1;
        self.clear_selection();
    }

    /// Delete the character before the cursor
    pub fn delete_char(&mut self) {
        if self.cursor_pos > 0 && !self.text.is_empty() {
            self.cursor_pos -= 1;
            self.text.remove(self.cursor_pos);
            self.clear_selection();
        }
    }

    /// Delete the character at the cursor
    pub fn delete_char_forward(&mut self) {
        if self.cursor_pos < self.text.len() {
            self.text.remove(self.cursor_pos);
            self.clear_selection();
        }
    }

    /// Move cursor left
    pub fn move_left(&mut self) {
        if self.cursor_pos > 0 {
            self.cursor_pos -= 1;
            self.update_selection();
        }
    }

    /// Move cursor right
    pub fn move_right(&mut self) {
        if self.cursor_pos < self.text.len() {
            self.cursor_pos += 1;
            self.update_selection();
        }
    }

    /// Move cursor to start
    pub fn move_to_start(&mut self) {
        self.cursor_pos = 0;
        self.update_selection();
    }

    /// Move cursor to end
    pub fn move_to_end(&mut self) {
        self.cursor_pos = self.text.len();
        self.update_selection();
    }

    /// Start selection mode
    pub fn start_selection(&mut self) {
        self.mode = Mode::Selecting;
        self.selection_anchor = Some(self.cursor_pos);
        self.selection = Some((self.cursor_pos, self.cursor_pos));
    }

    /// Update selection based on current cursor position
    fn update_selection(&mut self) {
        if self.mode == Mode::Selecting {
            if let Some(anchor) = self.selection_anchor {
                let start = anchor.min(self.cursor_pos);
                let end = anchor.max(self.cursor_pos);
                self.selection = Some((start, end));
            }
        }
    }

    /// Clear selection
    pub fn clear_selection(&mut self) {
        self.selection = None;
        self.selection_anchor = None;
        if self.mode == Mode::Selecting {
            self.mode = Mode::Normal;
        }
    }

    /// Apply current style to selection or character at cursor
    pub fn apply_style(&mut self) {
        let style = CharStyle {
            fg: self.current_fg,
            bg: self.current_bg,
            bold: self.current_bold,
            italic: self.current_italic,
            underline: self.current_underline,
            strikethrough: self.current_strikethrough,
            dim_level: self.current_dim,
        };

        if let Some((start, end)) = self.selection {
            for i in start..=end.min(self.text.len().saturating_sub(1)) {
                self.text[i].style = style.clone();
            }
        } else if self.cursor_pos < self.text.len() {
            self.text[self.cursor_pos].style = style;
        }
    }

    /// Toggle bold
    pub fn toggle_bold(&mut self) {
        self.current_bold = !self.current_bold;
        self.apply_style();
    }

    /// Toggle italic
    pub fn toggle_italic(&mut self) {
        self.current_italic = !self.current_italic;
        self.apply_style();
    }

    /// Toggle underline
    pub fn toggle_underline(&mut self) {
        self.current_underline = !self.current_underline;
        self.apply_style();
    }

    /// Toggle strikethrough
    pub fn toggle_strikethrough(&mut self) {
        self.current_strikethrough = !self.current_strikethrough;
        self.apply_style();
    }

    /// Cycle dim level
    pub fn cycle_dim(&mut self) {
        self.current_dim = (self.current_dim + 1) % 4;
        self.apply_style();
    }

    /// Toggle selection highlight mode
    pub fn toggle_selection_highlight_mode(&mut self) {
        self.selection_highlight_mode = match self.selection_highlight_mode {
            SelectionHighlightMode::Reversed => SelectionHighlightMode::Underline,
            SelectionHighlightMode::Underline => SelectionHighlightMode::Reversed,
        };
    }

    /// Load style from character at cursor position into current settings
    pub fn load_style_from_cursor(&mut self) {
        use crate::colors::color_index_from_color;
        
        if self.cursor_pos < self.text.len() {
            let style = &self.text[self.cursor_pos].style;
            self.current_fg = style.fg;
            self.current_bg = style.bg;
            self.current_bold = style.bold;
            self.current_italic = style.italic;
            self.current_underline = style.underline;
            self.current_strikethrough = style.strikethrough;
            self.current_dim = style.dim_level;
            
            // Update color picker indices
            self.fg_color_index = color_index_from_color(style.fg);
            self.bg_color_index = color_index_from_color(style.bg);
        }
    }

    /// Reset current style to defaults
    pub fn reset_style(&mut self) {
        self.current_fg = Color::Reset;
        self.current_bg = Color::Reset;
        self.current_bold = false;
        self.current_italic = false;
        self.current_underline = false;
        self.current_strikethrough = false;
        self.current_dim = 0;
        self.fg_color_index = 0; // None/Reset
        self.bg_color_index = 0; // None/Reset
    }

    /// Set status message
    pub fn set_status(&mut self, msg: impl Into<String>) {
        self.status_message = Some(msg.into());
    }

    /// Clear status message
    pub fn clear_status(&mut self) {
        self.status_message = None;
    }

    /// Check if a position is within the current selection
    pub fn is_selected(&self, pos: usize) -> bool {
        if let Some((start, end)) = self.selection {
            pos >= start && pos <= end
        } else {
            false
        }
    }
}
