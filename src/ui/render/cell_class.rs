//! Rules that pick the style of a grid cell: which type slot a value falls
//! under, and how file colors, type colors and overlays combine.
//!
//! These are plain functions so the rules can be tested without a workbook.
//! The viewport only gathers the inputs and calls them.

use ironcalc::base::{
    cell::CellValue, formatter::parser::Parser, types::CellType,
};
use ratatui::style::{Color, Style};

use crate::ui::theme::{Slot, Theme};

/// Picks the slot for a cell from its value, type and number format.
///
/// An empty value is checked first because IronCalc types a missing cell, and
/// an empty one, as `Number`.
pub fn classify(value: &CellValue, ty: CellType, num_fmt: &str) -> Slot {
    if matches!(value, CellValue::None) {
        return Slot::Cell;
    }
    match ty {
        CellType::ErrorValue => Slot::CellError,
        CellType::LogicalValue => Slot::CellBoolean,
        CellType::Number => match value {
            CellValue::Number(_) if is_date_format(num_fmt) => Slot::CellDate,
            CellValue::Number(n) if *n < 0.0 => Slot::CellNegative,
            _ => Slot::CellNumber,
        },
        CellType::Text => Slot::CellString,
        CellType::Array | CellType::CompoundData => Slot::Cell,
    }
}

/// Whether a number format shows its number as a date or time.
pub fn is_date_format(fmt: &str) -> bool {
    if fmt.eq_ignore_ascii_case("general") {
        return false;
    }
    let mut parser = Parser::new(fmt);
    parser.parse();
    parser.parts.iter().any(|p| p.is_date())
}

/// Black or white text, whichever reads better on the given background.
pub fn contrast_fg(r: u8, g: u8, b: u8) -> Color {
    let luminance = 0.2126 * f64::from(r) + 0.7152 * f64::from(g) + 0.0722 * f64::from(b);
    if luminance > 140.0 {
        Color::Black
    } else {
        Color::White
    }
}

/// The RGB value of a color, where it has one.
fn rgb_of(color: Color) -> Option<(u8, u8, u8)> {
    Some(match color {
        Color::Rgb(r, g, b) => (r, g, b),
        Color::Black => (0, 0, 0),
        Color::Red => (128, 0, 0),
        Color::Green => (0, 128, 0),
        Color::Yellow => (128, 128, 0),
        Color::Blue => (0, 0, 128),
        Color::Magenta => (128, 0, 128),
        Color::Cyan => (0, 128, 128),
        Color::Gray => (192, 192, 192),
        Color::DarkGray => (128, 128, 128),
        Color::LightRed => (255, 0, 0),
        Color::LightGreen => (0, 255, 0),
        Color::LightYellow => (255, 255, 0),
        Color::LightBlue => (0, 0, 255),
        Color::LightMagenta => (255, 0, 255),
        Color::LightCyan => (0, 255, 255),
        Color::White => (255, 255, 255),
        Color::Reset | Color::Indexed(_) => return None,
    })
}

/// Builds the color style of one cell.
///
/// The theme's `cell` and type slots come first. A fill from the file replaces
/// the background. A font color from the file replaces the foreground; with a
/// fill and no font color, the foreground is black or white by the fill's
/// brightness and the type slot's foreground is dropped. The selection and
/// cursor slots are patched on top.
pub fn cell_style(
    theme: &Theme,
    file_font: Option<Color>,
    file_fill: Option<Color>,
    class: Slot,
    in_selection: bool,
    is_cursor: bool,
) -> Style {
    let base = theme.get(Slot::Cell);
    let mut style = base.patch(theme.get(class));
    if let Some(fill) = file_fill {
        style.bg = Some(fill);
    }
    match (file_font, file_fill) {
        (Some(font), _) => style.fg = Some(font),
        (None, Some(fill)) => {
            style.fg = match rgb_of(fill) {
                Some((r, g, b)) => Some(contrast_fg(r, g, b)),
                None => base.fg,
            };
        }
        (None, None) => {}
    }
    if in_selection {
        style = style.patch(theme.get(Slot::Selection));
    }
    if is_cursor {
        style = style.patch(theme.get(Slot::Cursor));
    }
    style
}

#[cfg(test)]
mod test;
