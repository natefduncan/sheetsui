//! Color parsing shared by theme files and workbook colors.
use std::collections::HashMap;

use ratatui::style::Color;

/// Parses a theme color: `#RRGGBB`, an ANSI name, `terminal`, or a palette
/// name. A palette name wins over an ANSI name.
pub fn parse_color(s: &str, palette: &HashMap<String, Color>) -> Option<Color> {
    if let Some(c) = palette.get(s) {
        return Some(*c);
    }
    if s.eq_ignore_ascii_case("terminal") {
        return Some(Color::Reset);
    }
    parse_workbook_color(s)
}

/// Parses a color as a workbook gives it: an ANSI name, `#RRGGBB` or `rgb(...)`.
pub fn parse_workbook_color(s: &str) -> Option<Color> {
    let candidate = s.to_lowercase();
    let named = match candidate.as_str() {
        "red" => Color::Red,
        "blue" => Color::Blue,
        "green" => Color::Green,
        "magenta" => Color::Magenta,
        "cyan" => Color::Cyan,
        "white" => Color::White,
        "yellow" => Color::Yellow,
        "black" => Color::Black,
        "gray" | "grey" => Color::Gray,
        "lightred" => Color::LightRed,
        "lightblue" => Color::LightBlue,
        "lightgreen" => Color::LightGreen,
        "lightmagenta" => Color::LightMagenta,
        "lightcyan" => Color::LightCyan,
        "lightyellow" => Color::LightYellow,
        "darkgrey" | "darkgray" => Color::DarkGray,
        _ => {
            // TODO(jeremy): Should we support more syntaxes than hex string?
            // hsl(...) ??
            let rgb = if candidate.starts_with('#') {
                colorsys::Rgb::from_hex_str(&candidate).ok()?
            } else if candidate.starts_with("rgb(") {
                <colorsys::Rgb as std::str::FromStr>::from_str(&candidate).ok()?
            } else {
                return None;
            };
            // Note that the colorsys rgb model clamps the f64 values to no more
            // than 255.0 so the below casts are safe.
            Color::Rgb(rgb.red() as u8, rgb.green() as u8, rgb.blue() as u8)
        }
    };
    Some(named)
}
