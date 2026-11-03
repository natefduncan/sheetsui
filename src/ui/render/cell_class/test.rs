use ironcalc::base::{cell::CellValue, types::CellType};
use ratatui::style::{Color, Modifier, Style};

use super::*;
use crate::ui::theme::{parse, Slot, Theme};

fn typed_theme() -> Theme {
    let text = "\
style cell          fg #111111 bg #222222
style cell.number   fg red
style cell.string   fg green
style cell.boolean  fg yellow
style cell.date     fg cyan
style cell.negative fg magenta
style cell.error    fg blue
style selection     bg lightblue
style cursor        reverse true
";
    let (theme, warnings) = parse(text, &Theme::blank("test"));
    assert_eq!(warnings, vec![]);
    theme
}

#[test]
fn classify_each_type() {
    let n = |v| classify(&CellValue::Number(v), CellType::Number, "General");
    assert_eq!(Slot::CellNumber, n(1.5));
    assert_eq!(Slot::CellNumber, n(0.0));
    assert_eq!(Slot::CellNegative, n(-2.0));
    assert_eq!(
        Slot::CellString,
        classify(&CellValue::String("a".into()), CellType::Text, "General")
    );
    assert_eq!(
        Slot::CellBoolean,
        classify(&CellValue::Boolean(true), CellType::LogicalValue, "General")
    );
    assert_eq!(
        Slot::CellError,
        classify(&CellValue::String("#DIV/0!".into()), CellType::ErrorValue, "General")
    );
    assert_eq!(
        Slot::CellDate,
        classify(&CellValue::Number(45000.0), CellType::Number, "yyyy-mm-dd")
    );
    assert_eq!(
        Slot::Cell,
        classify(&CellValue::Number(1.0), CellType::Array, "General")
    );
}

#[test]
fn classify_empty_cells_are_plain() {
    // IronCalc types missing and empty cells as Number.
    assert_eq!(
        Slot::Cell,
        classify(&CellValue::None, CellType::Number, "General")
    );
    // A styled empty cell, here with a date format, is still plain.
    assert_eq!(
        Slot::Cell,
        classify(&CellValue::None, CellType::Number, "yyyy-mm-dd")
    );
}

#[test]
fn classify_precedence() {
    // A negative date is a date.
    assert_eq!(
        Slot::CellDate,
        classify(&CellValue::Number(-5.0), CellType::Number, "m/d/yy")
    );
    // An error beats a date format.
    assert_eq!(
        Slot::CellError,
        classify(&CellValue::String("#N/A".into()), CellType::ErrorValue, "m/d/yy")
    );
    // Text with a date format is text.
    assert_eq!(
        Slot::CellString,
        classify(&CellValue::String("x".into()), CellType::Text, "m/d/yy")
    );
}

#[test]
fn date_formats() {
    for f in ["yyyy-mm-dd", "m/d/yy", "h:mm:ss"] {
        assert!(is_date_format(f), "{f} should be a date format");
    }
    for f in ["\"day \"0", "[Red]0.00", "General", "general", "0.00", "#,##0"] {
        assert!(!is_date_format(f), "{f} should not be a date format");
    }
}

#[test]
fn contrast_picks_black_or_white() {
    assert_eq!(Color::White, contrast_fg(0, 0, 0));
    assert_eq!(Color::White, contrast_fg(0, 0, 255));
    assert_eq!(Color::Black, contrast_fg(255, 255, 255));
    assert_eq!(Color::Black, contrast_fg(255, 255, 0));
}

#[test]
fn style_uses_type_slot_without_file_colors() {
    let t = typed_theme();
    let s = cell_style(&t, None, None, Slot::CellNumber, false, false);
    assert_eq!(Style::default().fg(Color::Red).bg(Color::Rgb(0x22, 0x22, 0x22)), s);
    let s = cell_style(&t, None, None, Slot::Cell, false, false);
    assert_eq!(
        Style::default()
            .fg(Color::Rgb(0x11, 0x11, 0x11))
            .bg(Color::Rgb(0x22, 0x22, 0x22)),
        s
    );
}

#[test]
fn file_font_color_wins() {
    let t = typed_theme();
    let font = Color::Rgb(1, 2, 3);
    let s = cell_style(&t, Some(font), None, Slot::CellNumber, false, false);
    assert_eq!(Some(font), s.fg);
    assert_eq!(Some(Color::Rgb(0x22, 0x22, 0x22)), s.bg);
    // With a fill too, the font still wins over the contrast color.
    let fill = Color::Rgb(250, 250, 250);
    let s = cell_style(&t, Some(font), Some(fill), Slot::CellNumber, false, false);
    assert_eq!(Some(font), s.fg);
    assert_eq!(Some(fill), s.bg);
}

#[test]
fn fill_without_font_gets_contrast_text() {
    let t = typed_theme();
    let dark = Color::Rgb(10, 10, 60);
    let s = cell_style(&t, None, Some(dark), Slot::CellNumber, false, false);
    assert_eq!(Some(Color::White), s.fg);
    assert_eq!(Some(dark), s.bg);
    let light = Color::Rgb(240, 240, 200);
    let s = cell_style(&t, None, Some(light), Slot::CellError, false, false);
    assert_eq!(Some(Color::Black), s.fg);
    assert_eq!(Some(light), s.bg);
}

#[test]
fn overlays_apply_on_top() {
    let t = typed_theme();
    let s = cell_style(&t, None, None, Slot::CellNumber, true, false);
    assert_eq!(Some(Color::LightBlue), s.bg);
    assert_eq!(Some(Color::Red), s.fg);
    let s = cell_style(&t, None, None, Slot::CellNumber, true, true);
    assert!(s.add_modifier.contains(Modifier::REVERSED));
    assert_eq!(Some(Color::LightBlue), s.bg);
    // A file fill is covered by the selection too.
    let s = cell_style(&t, None, Some(Color::Rgb(0, 0, 0)), Slot::Cell, true, false);
    assert_eq!(Some(Color::LightBlue), s.bg);
}
