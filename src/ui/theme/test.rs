use ratatui::style::{Color, Modifier, Style};

use super::{color::parse_color, parse, parse::Warning, Slot, Theme};

fn blank() -> Theme {
    Theme::blank("test")
}

fn style_of(text: &str, slot: Slot) -> Style {
    let (theme, warnings) = parse(text, &blank());
    assert_eq!(warnings, vec![], "unexpected warnings for {text:?}");
    theme.get(slot)
}

#[test]
fn slot_names_round_trip() {
    assert_eq!(29, Slot::COUNT);
    for slot in Slot::ALL {
        assert_eq!(Some(*slot), Slot::from_name(&slot.to_string()));
    }
    assert_eq!(None, Slot::from_name("nope"));
}

#[test]
fn default_theme_has_no_warnings_and_old_colors() {
    let (_, warnings) = parse(super::DEFAULT_THEME, &blank());
    assert_eq!(warnings, vec![]);
    let t = Theme::default();
    assert_eq!(
        Style::default().fg(Color::White).bg(Color::Rgb(35, 33, 54)),
        t.get(Slot::Cell)
    );
    assert_eq!(
        Style::default()
            .fg(Color::White)
            .bg(Color::Rgb(57, 61, 71))
            .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        t.get(Slot::HeaderEven)
    );
    assert_eq!(
        Style::default()
            .fg(Color::Black)
            .bg(Color::Rgb(165, 169, 160))
            .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        t.get(Slot::HeaderOdd)
    );
    assert_eq!(
        Style::default().fg(Color::White).bg(Color::Rgb(57, 61, 71)),
        t.get(Slot::Cursor)
    );
    assert_eq!(
        Style::default().fg(Color::Black).bg(Color::LightBlue),
        t.get(Slot::Selection)
    );
    assert_eq!(
        Style::default().add_modifier(Modifier::REVERSED),
        t.get(Slot::TabActive)
    );
    assert_eq!(Style::default().bg(Color::Black), t.get(Slot::Dialog));
    assert_eq!(Style::default().fg(Color::Blue), t.get(Slot::Heading));
    assert_eq!(
        Style::default().add_modifier(Modifier::UNDERLINED),
        t.get(Slot::Link)
    );
    // Slots the default theme does not set stay unset.
    for slot in [
        Slot::HeaderRow,
        Slot::Border,
        Slot::Title,
        Slot::Tab,
        Slot::Status,
        Slot::Command,
        Slot::Edit,
        Slot::Info,
        Slot::Error,
        Slot::DialogBorder,
        Slot::Hint,
        Slot::CellNumber,
    ] {
        assert_eq!(Style::default(), t.get(slot), "{slot}");
    }
}

#[test]
fn comments_and_blank_lines_are_skipped() {
    let (theme, warnings) = parse("\n# a comment\n   \n  # indented\n", &blank());
    assert_eq!(warnings, vec![]);
    assert_eq!(blank(), theme);
}

#[test]
fn every_ansi_name_parses() {
    for (name, color) in [
        ("red", Color::Red),
        ("blue", Color::Blue),
        ("green", Color::Green),
        ("magenta", Color::Magenta),
        ("cyan", Color::Cyan),
        ("white", Color::White),
        ("yellow", Color::Yellow),
        ("black", Color::Black),
        ("gray", Color::Gray),
        ("grey", Color::Gray),
        ("lightred", Color::LightRed),
        ("lightblue", Color::LightBlue),
        ("lightgreen", Color::LightGreen),
        ("lightmagenta", Color::LightMagenta),
        ("lightcyan", Color::LightCyan),
        ("lightyellow", Color::LightYellow),
        ("darkgrey", Color::DarkGray),
        ("darkgray", Color::DarkGray),
    ] {
        assert_eq!(
            Style::default().fg(color),
            style_of(&format!("style cell fg {name}"), Slot::Cell)
        );
    }
}

#[test]
fn hex_terminal_and_bg() {
    assert_eq!(
        Style::default().fg(Color::Rgb(1, 2, 255)).bg(Color::Reset),
        style_of("style cell fg #0102FF bg terminal", Slot::Cell)
    );
}

#[test]
fn every_attribute_parses() {
    for (key, m) in [
        ("bold", Modifier::BOLD),
        ("italic", Modifier::ITALIC),
        ("underline", Modifier::UNDERLINED),
        ("dim", Modifier::DIM),
        ("reverse", Modifier::REVERSED),
    ] {
        assert_eq!(
            Style::default().add_modifier(m),
            style_of(&format!("style cell {key} true"), Slot::Cell)
        );
    }
}

#[test]
fn false_removes_a_modifier_from_the_base() {
    let base = Theme::default();
    let (theme, warnings) = parse("style header.even underline false", &base);
    assert_eq!(warnings, vec![]);
    let s = theme.get(Slot::HeaderEven);
    assert!(!s.add_modifier.contains(Modifier::UNDERLINED));
    assert!(s.add_modifier.contains(Modifier::BOLD));
    assert_eq!(Some(Color::White), s.fg);
}

#[test]
fn palette_names_resolve() {
    let text = "palette pink #FF00AA\npalette alias pink\nstyle cell fg pink bg alias";
    assert_eq!(
        Style::default()
            .fg(Color::Rgb(255, 0, 170))
            .bg(Color::Rgb(255, 0, 170)),
        style_of(text, Slot::Cell)
    );
    assert_eq!(
        Some(Color::Rgb(1, 2, 3)),
        parse_color(
            "x",
            &[("x".to_owned(), Color::Rgb(1, 2, 3))].into_iter().collect()
        )
    );
}

fn warns(text: &str) -> Vec<Warning> {
    parse(text, &blank()).1
}

fn w(line: usize, msg: &str) -> Warning {
    Warning {
        source: String::new(),
        line,
        msg: msg.to_owned(),
    }
}

#[test]
fn malformed_lines_warn_with_line_numbers() {
    assert_eq!(
        vec![
            w(2, "unknown line type 'bogus'"),
            w(3, "expected: style <slot> <key> <value> ..."),
            w(4, "expected: palette <name> <color>"),
        ],
        warns("# ok\nbogus line\nstyle\npalette x\n")
    );
}

#[test]
fn unknown_slot_skips_the_line() {
    let (theme, warnings) = parse("style cell fg red\nstyle nope fg red", &blank());
    assert_eq!(vec![w(2, "unknown slot 'nope'")], warnings);
    assert_eq!(Style::default().fg(Color::Red), theme.get(Slot::Cell));
}

#[test]
fn unknown_key_and_color_skip_only_that_part() {
    let (theme, warnings) = parse("\nstyle cell wobble 1 fg nocolor bg red bold true", &blank());
    assert_eq!(
        vec![w(2, "unknown key 'wobble'"), w(2, "unknown color 'nocolor'")],
        warnings
    );
    assert_eq!(
        Style::default().bg(Color::Red).add_modifier(Modifier::BOLD),
        theme.get(Slot::Cell)
    );
}

#[test]
fn palette_used_before_definition_warns() {
    let (theme, warnings) = parse("style cell fg later\npalette later red\nstyle cell bg later", &blank());
    assert_eq!(vec![w(1, "unknown color 'later'")], warnings);
    assert_eq!(Style::default().bg(Color::Red), theme.get(Slot::Cell));
}

#[test]
fn bad_palette_color_warns() {
    assert_eq!(vec![w(1, "unknown color 'zzz'")], warns("palette a zzz"));
}

#[test]
fn bad_boolean_and_dangling_key_warn() {
    assert_eq!(
        vec![
            w(1, "expected true or false for 'bold', got 'yes'"),
            w(2, "key 'bold' has no value"),
        ],
        warns("style cell bold yes\nstyle cell bold")
    );
}

#[test]
fn unset_slots_keep_the_base() {
    let base = Theme::default();
    let (theme, warnings) = parse("style cell fg red", &base);
    assert_eq!(vec![] as Vec<Warning>, warnings);
    for slot in Slot::ALL {
        if *slot != Slot::Cell {
            assert_eq!(base.get(*slot), theme.get(*slot), "{slot}");
        }
    }
    // A partly set slot keeps the rest of the base's style.
    assert_eq!(
        Style::default().fg(Color::Red).bg(Color::Rgb(35, 33, 54)),
        theme.get(Slot::Cell)
    );
}

#[test]
fn warning_display_shows_source_and_line() {
    let w = |source: &str, line| Warning {
        source: source.to_owned(),
        line,
        msg: "bad".to_owned(),
    };
    assert_eq!("keys:7: bad", w("keys", 7).to_string());
    assert_eq!("keys: bad", w("keys", 0).to_string());
    assert_eq!("line 7: bad", w("", 7).to_string());
}
