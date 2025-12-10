use crate::app::{App, StyledChar};
use crate::colors::{
    bg_ansi_code, bold_ansi_code, dim_ansi_code, fg_ansi_code,
    italic_ansi_code, strikethrough_ansi_code, underline_ansi_code,
};
use anyhow::Result;
use arboard::Clipboard;

/// Generate an echo command with ANSI escape codes for the styled text
pub fn generate_echo_command(text: &[StyledChar]) -> String {
    if text.is_empty() {
        return r#"echo -e """#.to_string();
    }

    let mut output = String::from(r#"echo -e ""#);
    let mut current_codes: Vec<String> = Vec::new();

    for styled_char in text {
        let mut new_codes: Vec<String> = Vec::new();

        // Foreground color
        new_codes.push(fg_ansi_code(styled_char.style.fg));

        // Background color (only if not reset)
        let bg_code = bg_ansi_code(styled_char.style.bg);
        if bg_code != "49" {
            new_codes.push(bg_code);
        }

        // Bold
        if let Some(bold) = bold_ansi_code(styled_char.style.bold) {
            new_codes.push(bold.to_string());
        }

        // Italic
        if let Some(italic) = italic_ansi_code(styled_char.style.italic) {
            new_codes.push(italic.to_string());
        }

        // Underline
        if let Some(underline) = underline_ansi_code(styled_char.style.underline) {
            new_codes.push(underline.to_string());
        }

        // Strikethrough
        if let Some(strike) = strikethrough_ansi_code(styled_char.style.strikethrough) {
            new_codes.push(strike.to_string());
        }

        // Dim
        if let Some(dim) = dim_ansi_code(styled_char.style.dim_level) {
            new_codes.push(dim.to_string());
        }

        // Only emit escape sequence if codes changed
        if new_codes != current_codes {
            // Reset first, then apply new codes
            let codes = new_codes.join(";");
            output.push_str(&format!(r#"\033[0;{}m"#, codes));
            current_codes = new_codes;
        }

        // Escape special characters
        match styled_char.ch {
            '\n' => output.push_str(r#"\n"#),
            '"' => output.push_str(r#"\""#),
            '\\' => output.push_str(r#"\\"#),
            '$' => output.push_str(r#"\$"#),
            '`' => output.push_str(r#"\`"#),
            '!' => output.push_str(r#"\!"#),
            _ => output.push(styled_char.ch),
        }
    }

    // Reset at the end
    output.push_str(r#"\033[0m""#);
    output
}

/// Copy the echo command to clipboard
pub fn copy_to_clipboard(app: &App) -> Result<()> {
    let command = generate_echo_command(&app.text);
    let mut clipboard = Clipboard::new()?;
    clipboard.set_text(&command)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::CharStyle;
    use ratatui::style::Color;

    #[test]
    fn test_generate_empty() {
        let text: Vec<StyledChar> = vec![];
        let result = generate_echo_command(&text);
        assert_eq!(result, r#"echo -e """#);
    }

    #[test]
    fn test_generate_simple() {
        let text: Vec<StyledChar> = vec![
            StyledChar::new('H'),
            StyledChar::new('i'),
        ];
        let result = generate_echo_command(&text);
        assert!(result.starts_with(r#"echo -e ""#));
        assert!(result.ends_with(r#"\033[0m""#));
        assert!(result.contains("Hi"));
    }

    #[test]
    fn test_generate_with_bold() {
        let text: Vec<StyledChar> = vec![
            StyledChar::with_style('B', CharStyle {
                fg: Color::Red,
                bg: Color::Reset,
                bold: true,
                italic: false,
                underline: false,
                strikethrough: false,
                dim_level: 0,
            }),
        ];
        let result = generate_echo_command(&text);
        assert!(result.contains("1")); // Bold code
        assert!(result.contains("31")); // Red foreground
    }

    #[test]
    fn test_generate_with_decorations() {
        let text: Vec<StyledChar> = vec![
            StyledChar::with_style('X', CharStyle {
                fg: Color::White,
                bg: Color::Reset,
                bold: false,
                italic: true,
                underline: true,
                strikethrough: true,
                dim_level: 0,
            }),
        ];
        let result = generate_echo_command(&text);
        assert!(result.contains("3")); // Italic code
        assert!(result.contains("4")); // Underline code
        assert!(result.contains("9")); // Strikethrough code
    }

    #[test]
    fn test_generate_multiline() {
        let text: Vec<StyledChar> = vec![
            StyledChar::new('H'),
            StyledChar::new('i'),
            StyledChar::new('\n'),
            StyledChar::new('!'),
        ];
        let result = generate_echo_command(&text);
        assert!(result.contains(r#"\n"#)); // Newline is escaped
        assert!(result.starts_with(r#"echo -e ""#));
        assert!(result.ends_with(r#"\033[0m""#));
    }
}
