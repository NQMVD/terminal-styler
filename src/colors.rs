use ratatui::style::Color;

/// Available colors for the palette (0-indexed for number key selection)
/// Index 0 is "None/Transparent" for background, uses default for foreground
pub const COLOR_PALETTE: &[(Color, &str, char)] = &[
    (Color::Reset, "None", '0'),
    (Color::Black, "Black", '1'),
    (Color::Red, "Red", '2'),
    (Color::Green, "Green", '3'),
    (Color::Yellow, "Yellow", '4'),
    (Color::Blue, "Blue", '5'),
    (Color::Magenta, "Magenta", '6'),
    (Color::Cyan, "Cyan", '7'),
    (Color::White, "White", '8'),
    (Color::DarkGray, "DarkGray", '9'),
    (Color::LightRed, "LightRed", 'a'),
    (Color::LightGreen, "LightGreen", 'b'),
    (Color::LightYellow, "LightYellow", 'c'),
    (Color::LightBlue, "LightBlue", 'd'),
    (Color::LightMagenta, "LightMagenta", 'e'),
    (Color::LightCyan, "LightCyan", 'f'),
    (Color::Gray, "Gray", 'g'),
];

/// Get color index from char key
pub fn color_index_from_key(key: char) -> Option<usize> {
    COLOR_PALETTE.iter().position(|(_, _, k)| *k == key.to_ascii_lowercase())
}

/// Get ANSI code for foreground color
pub fn fg_ansi_code(color: Color) -> String {
    match color {
        Color::Black => "30".to_string(),
        Color::Red => "31".to_string(),
        Color::Green => "32".to_string(),
        Color::Yellow => "33".to_string(),
        Color::Blue => "34".to_string(),
        Color::Magenta => "35".to_string(),
        Color::Cyan => "36".to_string(),
        Color::White => "37".to_string(),
        Color::DarkGray => "90".to_string(),
        Color::LightRed => "91".to_string(),
        Color::LightGreen => "92".to_string(),
        Color::LightYellow => "93".to_string(),
        Color::LightBlue => "94".to_string(),
        Color::LightMagenta => "95".to_string(),
        Color::LightCyan => "96".to_string(),
        Color::Gray => "97".to_string(),
        Color::Reset => "39".to_string(),
        Color::Rgb(r, g, b) => format!("38;2;{};{};{}", r, g, b),
        Color::Indexed(i) => format!("38;5;{}", i),
    }
}

/// Get ANSI code for background color
pub fn bg_ansi_code(color: Color) -> String {
    match color {
        Color::Black => "40".to_string(),
        Color::Red => "41".to_string(),
        Color::Green => "42".to_string(),
        Color::Yellow => "43".to_string(),
        Color::Blue => "44".to_string(),
        Color::Magenta => "45".to_string(),
        Color::Cyan => "46".to_string(),
        Color::White => "47".to_string(),
        Color::DarkGray => "100".to_string(),
        Color::LightRed => "101".to_string(),
        Color::LightGreen => "102".to_string(),
        Color::LightYellow => "103".to_string(),
        Color::LightBlue => "104".to_string(),
        Color::LightMagenta => "105".to_string(),
        Color::LightCyan => "106".to_string(),
        Color::Gray => "107".to_string(),
        Color::Reset => "49".to_string(),
        Color::Rgb(r, g, b) => format!("48;2;{};{};{}", r, g, b),
        Color::Indexed(i) => format!("48;5;{}", i),
    }
}

/// Get ANSI code for bold
pub fn bold_ansi_code(bold: bool) -> Option<&'static str> {
    if bold { Some("1") } else { None }
}

/// Get ANSI code for dim level
pub fn dim_ansi_code(level: u8) -> Option<&'static str> {
    match level {
        1..=3 => Some("2"), // ANSI dim code
        _ => None,
    }
}

/// Get ANSI code for italic
pub fn italic_ansi_code(italic: bool) -> Option<&'static str> {
    if italic { Some("3") } else { None }
}

/// Get ANSI code for underline
pub fn underline_ansi_code(underline: bool) -> Option<&'static str> {
    if underline { Some("4") } else { None }
}

/// Get ANSI code for strikethrough
pub fn strikethrough_ansi_code(strikethrough: bool) -> Option<&'static str> {
    if strikethrough { Some("9") } else { None }
}

/// Theme colors for the UI (Anthropic/Claude inspired)
pub mod theme {
    use ratatui::style::Color;

    // Background colors
    pub const BG_PRIMARY: Color = Color::Rgb(26, 26, 26);      // #1a1a1a
    pub const BG_SECONDARY: Color = Color::Rgb(35, 35, 35);    // #232323

    // Accent colors (warm orange/amber)
    pub const ACCENT_PRIMARY: Color = Color::Rgb(217, 119, 6);   // Amber-600
    pub const ACCENT_SECONDARY: Color = Color::Rgb(245, 158, 11); // Amber-500

    // Text colors
    pub const TEXT_PRIMARY: Color = Color::Rgb(250, 250, 250);   // #fafafa
    pub const TEXT_SECONDARY: Color = Color::Rgb(163, 163, 163); // #a3a3a3
    pub const TEXT_MUTED: Color = Color::Rgb(115, 115, 115);     // #737373

    // Border colors
    pub const BORDER_DEFAULT: Color = Color::Rgb(64, 64, 64);    // #404040
    pub const BORDER_FOCUSED: Color = Color::Rgb(217, 119, 6);   // Amber-600
    
    // Status colors
    pub const SUCCESS: Color = Color::Rgb(34, 197, 94);          // Green-500
    pub const ERROR: Color = Color::Rgb(239, 68, 68);            // Red-500
}
