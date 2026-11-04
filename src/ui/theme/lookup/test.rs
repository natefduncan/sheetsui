use std::path::Path;

use ratatui::style::Color;

use super::*;
use crate::ui::theme::Slot;

fn temp_dir(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("sheetui-lookup-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn write(dir: &Path, file: &str, text: &str) -> PathBuf {
    let path = dir.join(file);
    std::fs::write(&path, text).unwrap();
    path
}

#[test]
fn builtins_parse_with_zero_warnings() {
    for (name, _) in BUILTIN {
        let (theme, warnings) = load(name, None).unwrap();
        assert_eq!(warnings, vec![], "{name}");
        assert_eq!(&theme.name, name);
    }
}

#[test]
fn light_and_terminal_differ_from_default() {
    let default = Theme::default();
    let (light, _) = load("light", None).unwrap();
    let (terminal, _) = load("terminal", None).unwrap();
    assert_ne!(default.get(Slot::Cell), light.get(Slot::Cell));
    assert_eq!(Some(Color::Reset), terminal.get(Slot::Cell).fg);
    assert_eq!(Some(Color::Reset), terminal.get(Slot::Cell).bg);
}

#[test]
fn flag_beats_config_beats_default() {
    let (t, w) = resolve(Some("light"), Some("terminal"), None);
    assert_eq!(("light", 0), (t.name.as_str(), w.len()));
    let (t, w) = resolve(None, Some("terminal"), None);
    assert_eq!(("terminal", 0), (t.name.as_str(), w.len()));
    let (t, w) = resolve(None, None, None);
    assert_eq!(("default", 0), (t.name.as_str(), w.len()));
}

#[test]
fn user_theme_shadows_builtin_and_starts_from_default() {
    let dir = temp_dir("shadow");
    write(&dir, "light.theme", "style cell fg red\n");
    let (t, w) = resolve(Some("light"), None, Some(&dir));
    assert_eq!(w, vec![]);
    assert_eq!(Some(Color::Red), t.get(Slot::Cell).fg);
    // Unset slots keep the default value.
    assert_eq!(Theme::default().get(Slot::Cursor), t.get(Slot::Cursor));
    assert_eq!(Theme::default().get(Slot::Cell).bg, t.get(Slot::Cell).bg);
}

#[test]
fn path_value_loads_that_file() {
    let dir = temp_dir("path");
    let path = write(&dir, "mine.theme", "style cell fg green\n");
    let (t, w) = resolve(Some(path.to_str().unwrap()), None, None);
    assert_eq!(w, vec![]);
    assert_eq!("mine", t.name);
    assert_eq!(Some(Color::Green), t.get(Slot::Cell).fg);
}

#[test]
fn missing_theme_falls_back_with_warning() {
    let dir = temp_dir("missing");
    let (t, w) = resolve(Some("nope"), None, Some(&dir));
    assert_eq!(Theme::default(), t);
    assert_eq!(1, w.len());
    assert_eq!(0, w[0].line);
    let (t, w) = resolve(Some("/no/such/dir/x.theme"), None, None);
    assert_eq!(Theme::default(), t);
    assert_eq!(1, w.len());
}

#[test]
fn unreadable_file_falls_back_with_warning() {
    let dir = temp_dir("unreadable");
    // A directory named like a theme file cannot be read as text.
    std::fs::create_dir(dir.join("bad.theme")).unwrap();
    let (t, w) = resolve(Some("bad"), None, Some(&dir));
    assert_eq!(Theme::default(), t);
    assert_eq!(1, w.len());
    // Not valid UTF-8.
    std::fs::write(dir.join("bin.theme"), [0xff, 0xfe, 0x00]).unwrap();
    let (t, w) = resolve(Some("bin"), None, Some(&dir));
    assert_eq!(Theme::default(), t);
    assert_eq!(1, w.len());
}

#[test]
fn read_config_is_lenient() {
    let dir = temp_dir("config");
    assert_eq!(None, read_config(&dir), "missing file");
    write(&dir, "config", "# nothing\nother 1\n\n");
    assert_eq!(None, read_config(&dir), "no theme line");
    write(&dir, "config", "other 1\n  theme   light  \ntheme dracula\n");
    assert_eq!(Some("light".to_owned()), read_config(&dir));
    write(&dir, "config", "theme\ntheme terminal extra\n");
    assert_eq!(Some("terminal".to_owned()), read_config(&dir), "bare theme line skipped");
}

#[test]
fn list_has_builtins_and_user_themes_sorted() {
    let dir = temp_dir("list");
    write(&dir, "mine.theme", "");
    write(&dir, "light.theme", "");
    write(&dir, "notes.txt", "");
    let listed = list(Some(&dir));
    assert_eq!(
        listed,
        vec![
            ("default".to_owned(), Source::Builtin),
            ("dracula".to_owned(), Source::Builtin),
            ("light".to_owned(), Source::User),
            ("mine".to_owned(), Source::User),
            ("papercolor-dark".to_owned(), Source::Builtin),
            ("terminal".to_owned(), Source::Builtin),
        ]
    );
    assert_eq!(BUILTIN.len(), list(None).len());
}

#[test]
fn palette_themes_set_every_slot() {
    for name in ["dracula", "papercolor-dark"] {
        let (theme, _) = load(name, None).unwrap();
        for slot in Slot::ALL {
            assert_ne!(ratatui::style::Style::default(), theme.get(*slot), "{name}: {slot}");
        }
    }
}

#[test]
fn every_builtin_sets_the_search_slots() {
    for (name, text) in BUILTIN {
        // Parsed alone, so a value inherited from `default` does not count.
        let (theme, _) = parse(text, &Theme::blank(name));
        let search = theme.get(Slot::Search);
        let current = theme.get(Slot::SearchCurrent);
        assert_ne!(ratatui::style::Style::default(), search, "{name}");
        assert_ne!(search, current, "{name}");
        assert_ne!(theme.get(Slot::Cursor), current, "{name}");
        assert_ne!(theme.get(Slot::Selection), search, "{name}");
    }
}

#[test]
fn a_user_theme_without_the_search_slots_falls_back_to_default() {
    let dir = temp_dir("search-fallback");
    write(&dir, "plain.theme", "style cell fg red\n");
    let (theme, warnings) = load("plain", Some(&dir)).unwrap();
    assert_eq!(warnings, vec![]);
    let default = Theme::default();
    assert_eq!(default.get(Slot::Search), theme.get(Slot::Search));
    assert_eq!(default.get(Slot::SearchCurrent), theme.get(Slot::SearchCurrent));
    let _ = std::fs::remove_dir_all(&dir);
}
