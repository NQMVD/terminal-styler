//! Import functionality for ANSI escape codes and RON format

use crate::app::{App, CharStyle, StyledChar};
use anyhow::{anyhow, Result};
use arboard::Clipboard;
use pest::Parser;
use pest_derive::Parser;
use ratatui::style::Color;
use serde::{Deserialize, Serialize};

#[derive(Parser)]
#[grammar = "ansi.pest"]
struct AnsiParser;

/// Serializable version of CharStyle for RON export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableStyle {
    pub fg: SerializableColor,
    pub bg: SerializableColor,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strikethrough: bool,
    pub dim_level: u8,
}

/// Serializable color representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SerializableColor {
    Reset,
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    DarkGray,
    LightRed,
    LightGreen,
    LightYellow,
    LightBlue,
    LightMagenta,
    LightCyan,
    Gray,
    Rgb(u8, u8, u8),
    Indexed(u8),
}

/// Serializable styled character for RON export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableChar {
    pub ch: char,
    pub style: SerializableStyle,
}

/// Serializable document for RON export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyledDocument {
    pub version: u8,
    pub chars: Vec<SerializableChar>,
}

impl From<Color> for SerializableColor {
    fn from(color: Color) -> Self {
        match color {
            Color::Reset => SerializableColor::Reset,
            Color::Black => SerializableColor::Black,
            Color::Red => SerializableColor::Red,
            Color::Green => SerializableColor::Green,
            Color::Yellow => SerializableColor::Yellow,
            Color::Blue => SerializableColor::Blue,
            Color::Magenta => SerializableColor::Magenta,
            Color::Cyan => SerializableColor::Cyan,
            Color::White => SerializableColor::White,
            Color::DarkGray => SerializableColor::DarkGray,
            Color::LightRed => SerializableColor::LightRed,
            Color::LightGreen => SerializableColor::LightGreen,
            Color::LightYellow => SerializableColor::LightYellow,
            Color::LightBlue => SerializableColor::LightBlue,
            Color::LightMagenta => SerializableColor::LightMagenta,
            Color::LightCyan => SerializableColor::LightCyan,
            Color::Gray => SerializableColor::Gray,
            Color::Rgb(r, g, b) => SerializableColor::Rgb(r, g, b),
            Color::Indexed(i) => SerializableColor::Indexed(i),
        }
    }
}

impl From<SerializableColor> for Color {
    fn from(color: SerializableColor) -> Self {
        match color {
            SerializableColor::Reset => Color::Reset,
            SerializableColor::Black => Color::Black,
            SerializableColor::Red => Color::Red,
            SerializableColor::Green => Color::Green,
            SerializableColor::Yellow => Color::Yellow,
            SerializableColor::Blue => Color::Blue,
            SerializableColor::Magenta => Color::Magenta,
            SerializableColor::Cyan => Color::Cyan,
            SerializableColor::White => Color::White,
            SerializableColor::DarkGray => Color::DarkGray,
            SerializableColor::LightRed => Color::LightRed,
            SerializableColor::LightGreen => Color::LightGreen,
            SerializableColor::LightYellow => Color::LightYellow,
            SerializableColor::LightBlue => Color::LightBlue,
            SerializableColor::LightMagenta => Color::LightMagenta,
            SerializableColor::LightCyan => Color::LightCyan,
            SerializableColor::Gray => Color::Gray,
            SerializableColor::Rgb(r, g, b) => Color::Rgb(r, g, b),
            SerializableColor::Indexed(i) => Color::Indexed(i),
        }
    }
}

impl From<&CharStyle> for SerializableStyle {
    fn from(style: &CharStyle) -> Self {
        SerializableStyle {
            fg: style.fg.into(),
            bg: style.bg.into(),
            bold: style.bold,
            italic: style.italic,
            underline: style.underline,
            strikethrough: style.strikethrough,
            dim_level: style.dim_level,
        }
    }
}

impl From<SerializableStyle> for CharStyle {
    fn from(style: SerializableStyle) -> Self {
        CharStyle {
            fg: style.fg.into(),
            bg: style.bg.into(),
            bold: style.bold,
            italic: style.italic,
            underline: style.underline,
            strikethrough: style.strikethrough,
            dim_level: style.dim_level,
        }
    }
}

impl From<&StyledChar> for SerializableChar {
    fn from(sc: &StyledChar) -> Self {
        SerializableChar {
            ch: sc.ch,
            style: (&sc.style).into(),
        }
    }
}

impl From<SerializableChar> for StyledChar {
    fn from(sc: SerializableChar) -> Self {
        StyledChar::with_style(sc.ch, sc.style.into())
    }
}

/// Current style state during ANSI parsing
#[derive(Debug, Clone, Default)]
struct ParseState {
    fg: Color,
    bg: Color,
    bold: bool,
    italic: bool,
    underline: bool,
    strikethrough: bool,
    dim: bool,
}

impl ParseState {
    fn reset(&mut self) {
        *self = Self::default();
    }

    fn to_char_style(&self) -> CharStyle {
        CharStyle {
            fg: self.fg,
            bg: self.bg,
            bold: self.bold,
            italic: self.italic,
            underline: self.underline,
            strikethrough: self.strikethrough,
            dim_level: if self.dim { 1 } else { 0 },
        }
    }
}

/// Apply a single SGR parameter to the parse state
fn apply_sgr_param(state: &mut ParseState, params: &[u32], index: &mut usize) {
    if *index >= params.len() {
        return;
    }

    match params[*index] {
        0 => state.reset(),
        1 => state.bold = true,
        2 => state.dim = true,
        3 => state.italic = true,
        4 => state.underline = true,
        9 => state.strikethrough = true,
        22 => {
            state.bold = false;
            state.dim = false;
        }
        23 => state.italic = false,
        24 => state.underline = false,
        29 => state.strikethrough = false,
        // Standard foreground colors (30-37)
        30 => state.fg = Color::Black,
        31 => state.fg = Color::Red,
        32 => state.fg = Color::Green,
        33 => state.fg = Color::Yellow,
        34 => state.fg = Color::Blue,
        35 => state.fg = Color::Magenta,
        36 => state.fg = Color::Cyan,
        37 => state.fg = Color::White,
        38 => {
            // Extended foreground color
            *index += 1;
            if *index < params.len() {
                match params[*index] {
                    5 => {
                        // 256-color mode
                        *index += 1;
                        if *index < params.len() {
                            state.fg = Color::Indexed(params[*index] as u8);
                        }
                    }
                    2 => {
                        // RGB mode
                        if *index + 3 < params.len() {
                            let r = params[*index + 1] as u8;
                            let g = params[*index + 2] as u8;
                            let b = params[*index + 3] as u8;
                            state.fg = Color::Rgb(r, g, b);
                            *index += 3;
                        }
                    }
                    _ => {}
                }
            }
        }
        39 => state.fg = Color::Reset,
        // Standard background colors (40-47)
        40 => state.bg = Color::Black,
        41 => state.bg = Color::Red,
        42 => state.bg = Color::Green,
        43 => state.bg = Color::Yellow,
        44 => state.bg = Color::Blue,
        45 => state.bg = Color::Magenta,
        46 => state.bg = Color::Cyan,
        47 => state.bg = Color::White,
        48 => {
            // Extended background color
            *index += 1;
            if *index < params.len() {
                match params[*index] {
                    5 => {
                        // 256-color mode
                        *index += 1;
                        if *index < params.len() {
                            state.bg = Color::Indexed(params[*index] as u8);
                        }
                    }
                    2 => {
                        // RGB mode
                        if *index + 3 < params.len() {
                            let r = params[*index + 1] as u8;
                            let g = params[*index + 2] as u8;
                            let b = params[*index + 3] as u8;
                            state.bg = Color::Rgb(r, g, b);
                            *index += 3;
                        }
                    }
                    _ => {}
                }
            }
        }
        49 => state.bg = Color::Reset,
        // Bright foreground colors (90-97)
        90 => state.fg = Color::DarkGray,
        91 => state.fg = Color::LightRed,
        92 => state.fg = Color::LightGreen,
        93 => state.fg = Color::LightYellow,
        94 => state.fg = Color::LightBlue,
        95 => state.fg = Color::LightMagenta,
        96 => state.fg = Color::LightCyan,
        97 => state.fg = Color::Gray,
        // Bright background colors (100-107)
        100 => state.bg = Color::DarkGray,
        101 => state.bg = Color::LightRed,
        102 => state.bg = Color::LightGreen,
        103 => state.bg = Color::LightYellow,
        104 => state.bg = Color::LightBlue,
        105 => state.bg = Color::LightMagenta,
        106 => state.bg = Color::LightCyan,
        107 => state.bg = Color::Gray,
        _ => {}
    }
}

/// Parse ANSI-styled text into StyledChars
pub fn parse_ansi(input: &str) -> Result<Vec<StyledChar>> {
    let pairs = AnsiParser::parse(Rule::ansi_text, input)
        .map_err(|e| anyhow!("Failed to parse ANSI: {}", e))?;

    let mut result = Vec::new();
    let mut state = ParseState::default();

    for pair in pairs {
        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::plain_char => {
                    let ch = inner.as_str().chars().next().unwrap();
                    result.push(StyledChar::with_style(ch, state.to_char_style()));
                }
                Rule::literal_escape => {
                    // Handle literal escape sequences like \n, \t, \r
                    let ch = match inner.as_str() {
                        "\\n" => '\n',
                        "\\t" => '\t',
                        "\\r" => '\r',
                        _ => continue,
                    };
                    result.push(StyledChar::with_style(ch, state.to_char_style()));
                }
                Rule::escape_sequence => {
                    // Find the sgr_params inside the escape sequence
                    for seq_inner in inner.into_inner() {
                        if seq_inner.as_rule() == Rule::sgr_params {
                            let params: Vec<u32> = seq_inner
                                .into_inner()
                                .filter(|p| p.as_rule() == Rule::param)
                                .filter_map(|p| p.as_str().parse().ok())
                                .collect();

                            // Apply all parameters
                            let mut i = 0;
                            while i < params.len() {
                                apply_sgr_param(&mut state, &params, &mut i);
                                i += 1;
                            }
                            
                            // Handle empty params (reset)
                            if params.is_empty() {
                                state.reset();
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }

    Ok(result)
}

/// Export styled text to RON format
pub fn export_ron(text: &[StyledChar]) -> Result<String> {
    let doc = StyledDocument {
        version: 1,
        chars: text.iter().map(|c| c.into()).collect(),
    };

    ron::ser::to_string_pretty(&doc, ron::ser::PrettyConfig::default())
        .map_err(|e| anyhow!("Failed to serialize to RON: {}", e))
}

/// Import styled text from RON format
pub fn import_ron(input: &str) -> Result<Vec<StyledChar>> {
    let doc: StyledDocument =
        ron::from_str(input).map_err(|e| anyhow!("Failed to parse RON: {}", e))?;

    Ok(doc.chars.into_iter().map(|c| c.into()).collect())
}

/// Detect if input is RON format (starts with opening paren or struct name)
pub fn is_ron_format(input: &str) -> bool {
    let trimmed = input.trim();
    trimmed.starts_with('(') || trimmed.starts_with("StyledDocument")
}

/// Detect if input is an echo command and extract the content inside quotes
/// Returns the inner content if it's an echo command, otherwise returns the original input
pub fn strip_echo_wrapper(input: &str) -> &str {
    let trimmed = input.trim();
    
    // Check for various echo command patterns
    // echo -e "..."
    // echo -e '...'
    // echo "..."
    // printf "..."
    
    let prefixes = [
        r#"echo -e ""#,
        r#"echo -e '"#,
        r#"echo ""#,
        r#"echo '"#,
        r#"printf ""#,
        r#"printf '"#,
    ];
    
    for prefix in prefixes {
        if trimmed.starts_with(prefix) {
            let after_prefix = &trimmed[prefix.len()..];
            // Find the matching closing quote
            let quote_char = prefix.chars().last().unwrap();
            
            // Find the last occurrence of the quote (handling escaped quotes)
            if let Some(end_pos) = after_prefix.rfind(quote_char) {
                return &after_prefix[..end_pos];
            }
        }
    }
    
    // Also handle $'...' syntax (bash ANSI-C quoting)
    if trimmed.starts_with("echo $'") || trimmed.starts_with("echo -e $'") {
        let start = if trimmed.starts_with("echo -e $'") {
            "echo -e $'".len()
        } else {
            "echo $'".len()
        };
        let after_prefix = &trimmed[start..];
        if let Some(end_pos) = after_prefix.rfind('\'') {
            return &after_prefix[..end_pos];
        }
    }
    
    input
}

/// Import from clipboard - auto-detect format (RON vs ANSI)
pub fn import_from_clipboard(app: &mut App) -> Result<String> {
    let mut clipboard = Clipboard::new()?;
    let content = clipboard.get_text()?;

    let (chars, format_name) = if is_ron_format(&content) {
        (import_ron(&content)?, "RON")
    } else {
        // Try to strip echo wrapper if present
        let stripped = strip_echo_wrapper(&content);
        let was_echo = stripped.len() != content.len();
        let format = if was_echo { "echo cmd" } else { "ANSI" };
        (parse_ansi(stripped)?, format)
    };

    let char_count = chars.len();
    app.text = chars;
    app.cursor_pos = app.text.len();
    app.clear_selection();

    Ok(format!("Imported {} chars ({})", char_count, format_name))
}

/// Export to RON and copy to clipboard
pub fn export_ron_to_clipboard(app: &App) -> Result<()> {
    let ron_str = export_ron(&app.text)?;
    let mut clipboard = Clipboard::new()?;
    clipboard.set_text(&ron_str)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_plain_text() {
        let result = parse_ansi("Hello").unwrap();
        assert_eq!(result.len(), 5);
        assert_eq!(result[0].ch, 'H');
    }

    #[test]
    fn test_parse_bold() {
        let result = parse_ansi("\x1b[1mBold\x1b[0m").unwrap();
        assert_eq!(result.len(), 4);
        assert!(result[0].style.bold);
    }

    #[test]
    fn test_parse_color() {
        let result = parse_ansi("\x1b[31mRed\x1b[0m").unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].style.fg, Color::Red);
    }

    #[test]
    fn test_parse_combined() {
        let result = parse_ansi("\x1b[1;31;44mText\x1b[0m").unwrap();
        assert_eq!(result.len(), 4);
        assert!(result[0].style.bold);
        assert_eq!(result[0].style.fg, Color::Red);
        assert_eq!(result[0].style.bg, Color::Blue);
    }

    #[test]
    fn test_ron_roundtrip() {
        let chars = vec![
            StyledChar::with_style(
                'A',
                CharStyle {
                    fg: Color::Red,
                    bg: Color::Blue,
                    bold: true,
                    italic: false,
                    underline: true,
                    strikethrough: false,
                    dim_level: 0,
                },
            ),
            StyledChar::with_style('B', CharStyle::default()),
        ];

        let ron_str = export_ron(&chars).unwrap();
        let imported = import_ron(&ron_str).unwrap();

        assert_eq!(imported.len(), 2);
        assert_eq!(imported[0].ch, 'A');
        assert_eq!(imported[0].style.fg, Color::Red);
        assert!(imported[0].style.bold);
    }

    #[test]
    fn test_is_ron_format() {
        assert!(is_ron_format("(version: 1, chars: [])"));
        assert!(is_ron_format("StyledDocument(...)"));
        assert!(!is_ron_format("\x1b[31mHello"));
        assert!(!is_ron_format("plain text"));
    }

    #[test]
    fn test_parse_literal_octal_escape() {
        // Literal \033 format (common in echo -e output when copied as text)
        let result = parse_ansi("\\033[31mRed\\033[0m").unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].style.fg, Color::Red);
        assert_eq!(result[0].ch, 'R');
    }

    #[test]
    fn test_parse_literal_hex_escape() {
        // Literal \x1b format
        let result = parse_ansi("\\x1b[1;32mBoldGreen\\x1b[0m").unwrap();
        assert_eq!(result.len(), 9);
        assert!(result[0].style.bold);
        assert_eq!(result[0].style.fg, Color::Green);
    }

    #[test]
    fn test_parse_literal_e_escape() {
        // Literal \e format (bash shorthand)
        let result = parse_ansi("\\e[44mBlue\\e[0m").unwrap();
        assert_eq!(result.len(), 4);
        assert_eq!(result[0].style.bg, Color::Blue);
    }

    #[test]
    fn test_strip_echo_wrapper_double_quotes() {
        let input = r#"echo -e "\033[31mHello\033[0m""#;
        let stripped = strip_echo_wrapper(input);
        assert_eq!(stripped, r#"\033[31mHello\033[0m"#);
    }

    #[test]
    fn test_strip_echo_wrapper_single_quotes() {
        let input = r#"echo -e '\033[31mHello\033[0m'"#;
        let stripped = strip_echo_wrapper(input);
        assert_eq!(stripped, r#"\033[31mHello\033[0m"#);
    }

    #[test]
    fn test_strip_echo_wrapper_no_e_flag() {
        let input = r#"echo "\033[31mHello\033[0m""#;
        let stripped = strip_echo_wrapper(input);
        assert_eq!(stripped, r#"\033[31mHello\033[0m"#);
    }

    #[test]
    fn test_strip_echo_wrapper_printf() {
        let input = r#"printf "\033[31mHello\033[0m""#;
        let stripped = strip_echo_wrapper(input);
        assert_eq!(stripped, r#"\033[31mHello\033[0m"#);
    }

    #[test]
    fn test_strip_echo_wrapper_not_echo() {
        let input = r#"\033[31mHello\033[0m"#;
        let stripped = strip_echo_wrapper(input);
        assert_eq!(stripped, input);
    }

    #[test]
    fn test_strip_echo_wrapper_ansi_c_quoting() {
        let input = r#"echo $'\033[31mHello\033[0m'"#;
        let stripped = strip_echo_wrapper(input);
        assert_eq!(stripped, r#"\033[31mHello\033[0m"#);
    }

    #[test]
    fn test_parse_multiline_literal() {
        // Test parsing literal \n newlines from echo command format
        let result = parse_ansi(r#"Line1\nLine2"#).unwrap();
        assert_eq!(result.len(), 11); // "Line1" + \n + "Line2"
        assert_eq!(result[5].ch, '\n'); // The newline character
        assert_eq!(result[6].ch, 'L');
    }

    #[test]
    fn test_parse_multiline_with_style() {
        // Test parsing multiline with ANSI styling
        let result = parse_ansi(r#"\033[31mRed\nLine\033[0m"#).unwrap();
        assert_eq!(result.len(), 8); // "Red" + \n + "Line"
        assert_eq!(result[0].style.fg, Color::Red);
        assert_eq!(result[3].ch, '\n');
        assert_eq!(result[4].style.fg, Color::Red); // Style persists after newline
    }
}
