use ironcalc::base::Model;
use ratatui::style::Color;

use crate::ui::AppState;

use super::{Address, Book, Viewport, ViewportState};

#[test]
fn test_viewport_get_visible_columns() {
    let state = ViewportState::default();
    let book = Book::from_model(
        Model::new_empty("test", "en", "America/New_York", "en").expect("Failed to make model"),
    );
    let default_size = book.get_col_size(1).expect("Failed to get column size");
    let width = default_size * 12 / 2;
    let app_state = AppState::default();
    let viewport = Viewport::new(&book, Some(&app_state.range_select), &app_state.theme).with_selected(Address {
        sheet: 0,
        row: 1,
        col: 17,
    });
    let cols = viewport
        .get_visible_columns((width + 5) as u16, &state)
        .expect("Failed to get visible columns");
    assert_eq!(5, cols.len());
    assert_eq!(17, cols.last().expect("Failed to get last column").idx);
}

#[test]
fn test_viewport_get_visible_rows() {
    let state = ViewportState::default();
    let book = Book::from_model(
        Model::new_empty("test", "en", "America/New_York", "en").expect("Failed to make model"),
    );
    let height = 6;
    let app_state = AppState::default();
    let viewport = Viewport::new(&book, Some(&app_state.range_select), &app_state.theme).with_selected(Address {
        sheet: 0,
        row: 17,
        col: 1,
    });
    let rows = viewport.get_visible_rows(height as u16, &state);
    assert_eq!(height - 1, rows.len());
    assert_eq!(
        17 - (height - 2),
        *rows.first().expect("Failed to get first row")
    );
    assert_eq!(17, *rows.last().expect("Failed to get last row"));
}

#[test]
fn test_viewport_visible_columns_after_length_change() {
    let state = ViewportState::default();
    let mut book = Book::from_model(
        Model::new_empty("test", "en", "America/New_York", "en").expect("Failed to make model"),
    );
    let default_size = book.get_col_size(1).expect("Failed to get column size");
    let width = default_size * 12 / 2;
    {
        let app_state = AppState::default();
        let viewport = Viewport::new(&book, Some(&app_state.range_select), &app_state.theme).with_selected(Address {
            sheet: 0,
            row: 1,
            col: 17,
        });
        let cols = viewport
            .get_visible_columns((width + 5) as u16, &state)
            .expect("Failed to get visible columns");
        assert_eq!(5, cols.len());
        assert_eq!(17, cols.last().expect("Failed to get last column").idx);
    }

    book.set_col_size(1, default_size * 6)
        .expect("Failed to set column size");
    {
        let app_state = AppState::default();
        let viewport = Viewport::new(&book, Some(&app_state.range_select), &app_state.theme).with_selected(Address {
            sheet: 0,
            row: 1,
            col: 1,
        });
        let cols = viewport
            .get_visible_columns((width + 5) as u16, &state)
            .expect("Failed to get visible columns");
        assert_eq!(1, cols.len());
        assert_eq!(1, cols.last().expect("Failed to get last column").idx);
    }
}

#[test]
fn test_color_mapping() {
    for (s, c) in [
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
        ("#35f15b", Color::Rgb(53, 241, 91)),
    ]
    .map(|(s, c)| (Some(s.to_string()), c))
    {
        assert_eq!(super::viewport::map_color(s.as_ref(), Color::Gray), c);
    }
}

#[test]
fn test_draw_shows_the_command_cursor() {
    use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
    use ratatui::{backend::TestBackend, Terminal};

    let mut ws = crate::ui::Workspace::new_empty("en", "America/New_York").expect("workspace");
    for code in [
        KeyCode::Char(':'),
        KeyCode::Char('a'),
        KeyCode::Char('b'),
        KeyCode::Left,
    ] {
        ws.handle_input(Event::Key(KeyEvent::new(code, KeyModifiers::empty())))
            .expect("handle key");
    }
    let mut term = Terminal::new(TestBackend::new(60, 14)).expect("terminal");
    term.draw(|f| super::draw(f, &mut ws)).expect("draw");
    // Find the prompt text "ab" on screen; the cursor must be on the "b".
    let buf = term.backend().buffer().clone();
    let (x, y) = (0..buf.area.height)
        .find_map(|y| {
            (0..buf.area.width - 1)
                .find(|&x| buf[(x, y)].symbol() == "a" && buf[(x + 1, y)].symbol() == "b")
                .map(|x| (x, y))
        })
        .expect("prompt text on screen");
    let mut backend = term.backend().clone();
    use ratatui::backend::Backend;
    assert_eq!(
        backend.get_cursor_position().expect("cursor"),
        ratatui::layout::Position::new(x + 1, y)
    );
}

#[test]
fn test_default_theme_draws_the_old_colors() {
    use ratatui::{backend::TestBackend, style::Modifier, Terminal};

    let mut ws = crate::ui::Workspace::new_empty("en", "America/New_York").expect("workspace");
    let mut term = Terminal::new(TestBackend::new(60, 14)).expect("terminal");
    term.draw(|f| super::draw(f, &mut ws)).expect("draw");
    let buf = term.backend().buffer().clone();
    let find = |sym: &str, y_from: u16| {
        (y_from..buf.area.height)
            .find_map(|y| {
                (0..buf.area.width)
                    .find(|&x| buf[(x, y)].symbol() == sym)
                    .map(|x| (x, y))
            })
            .unwrap_or_else(|| panic!("{sym} on screen"))
    };
    // Column header row: "A" is odd-indexed (col 1), "B" even.
    let (ax, ay) = find("A", 8);
    let odd = &buf[(ax, ay)];
    assert_eq!(Color::Black, odd.fg);
    assert_eq!(Color::Rgb(165, 169, 160), odd.bg);
    assert!(odd.modifier.contains(Modifier::BOLD | Modifier::UNDERLINED));
    let (bx, by) = find("B", 8);
    let even = &buf[(bx, by)];
    assert_eq!(Color::White, even.fg);
    assert_eq!(Color::Rgb(57, 61, 71), even.bg);
    // The corner cell above the row numbers is not underlined.
    let corner = &buf[(3, ay)];
    assert!(!corner.modifier.contains(Modifier::UNDERLINED));
    // Cursor cell A1 is below the A header.
    let cursor = &buf[(ax + 1, ay + 1)];
    assert_eq!(Color::White, cursor.fg);
    assert_eq!(Color::Rgb(57, 61, 71), cursor.bg);
    // A plain cell.
    let plain = &buf[(bx + 1, ay + 2)];
    assert_eq!(Color::White, plain.fg);
    assert_eq!(Color::Rgb(35, 33, 54), plain.bg);
}

#[test]
fn test_dialog_draws_the_dialog_slot() {
    use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
    use ratatui::{backend::TestBackend, Terminal};

    let mut ws = crate::ui::Workspace::new_empty("en", "America/New_York").expect("workspace");
    ws.handle_input(Event::Key(KeyEvent::new(
        KeyCode::Char('h'),
        KeyModifiers::ALT,
    )))
    .expect("open help");
    let mut term = Terminal::new(TestBackend::new(130, 20)).expect("terminal");
    term.draw(|f| super::draw(f, &mut ws)).expect("draw");
    let buf = term.backend().buffer().clone();
    // Inside the dialog border.
    assert_eq!(Color::Black, buf[(65, 10)].bg);
}
