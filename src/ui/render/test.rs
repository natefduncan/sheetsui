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

const SLOT_TEST_THEME: &str = "\
style border        fg #010101
style title         fg #020202
style tab           fg #030303
style tab.active    fg #040404
style status        fg #050505
style command       fg #060606
style edit          fg #070707
style hint          fg #080808
style dialog.border fg #090909
";

fn rgb(n: u8) -> Color {
    Color::Rgb(n, n, n)
}

/// Finds the first cell of `text` on screen.
fn find_text(buf: &ratatui::buffer::Buffer, text: &str) -> (u16, u16) {
    let chars: Vec<String> = text.chars().map(|c| c.to_string()).collect();
    for y in 0..buf.area.height {
        for x in 0..buf.area.width.saturating_sub(chars.len() as u16 - 1) {
            if chars
                .iter()
                .enumerate()
                .all(|(i, c)| buf[(x + i as u16, y)].symbol() == c)
            {
                return (x, y);
            }
        }
    }
    panic!("{text:?} on screen");
}

fn draw_with_slot_theme(
    keys: &[(crossterm::event::KeyCode, crossterm::event::KeyModifiers)],
    width: u16,
    height: u16,
) -> ratatui::buffer::Buffer {
    use crossterm::event::{Event, KeyEvent};
    use ratatui::{backend::TestBackend, Terminal};

    let mut ws = crate::ui::Workspace::new_empty("en", "America/New_York").expect("workspace");
    let (theme, warnings) = crate::ui::theme::parse(
        SLOT_TEST_THEME,
        &crate::ui::theme::Theme::blank("slots"),
    );
    assert!(warnings.is_empty(), "{warnings:?}");
    // Goes through the same path as `:theme`, so the text area is restyled.
    ws.set_theme(theme);
    for (code, mods) in keys {
        ws.handle_input(Event::Key(KeyEvent::new(*code, *mods)))
            .expect("handle key");
    }
    let mut term = Terminal::new(TestBackend::new(width, height)).expect("terminal");
    term.draw(|f| super::draw(f, &mut ws)).expect("draw");
    term.backend().buffer().clone()
}

#[test]
fn test_frame_slots_style_their_widgets() {
    let buf = draw_with_slot_theme(&[], 60, 14);
    // Outer block: border corner and file name title.
    assert_eq!(rgb(1), buf[(0, 0)].fg);
    let (tx, ty) = find_text(&buf, "Untitled.sui");
    assert_eq!(rgb(2), buf[(tx, ty)].fg);
    // Status: modality and row,col share the bottom row.
    let (sx, sy) = find_text(&buf, "navigate");
    assert_eq!(rgb(5), buf[(sx, sy)].fg);
    let (cx, cy) = find_text(&buf, "1,1");
    assert_eq!(rgb(5), buf[(cx, cy)].fg);
    // Active tab, and the tab bar to its right.
    let (ax, ay) = find_text(&buf, "Sheet1 0");
    assert_eq!(rgb(4), buf[(ax, ay)].fg);
    assert_eq!(rgb(3), buf[(55, ay)].fg);
    // Hint, and the edit box interior on the same row.
    let (hx, hy) = find_text(&buf, "ALT-h");
    assert_eq!(rgb(8), buf[(hx, hy)].fg);
    assert_eq!(rgb(7), buf[(3, hy)].fg);
    // Edit box border and viewport border/title.
    assert_eq!(rgb(1), buf[(2, hy - 1)].fg);
    // The sheet name also appears in the tab bar, so look below it.
    let (vx, vy) = find_text_below(&buf, "Sheet1", ay);
    assert_eq!(rgb(2), buf[(vx, vy)].fg);
    assert_eq!(rgb(1), buf[(2, vy)].fg);
}

fn find_text_below(buf: &ratatui::buffer::Buffer, text: &str, y_from: u16) -> (u16, u16) {
    for y in (y_from + 1)..buf.area.height {
        for x in 0..buf.area.width {
            if buf[(x, y)].symbol() == &text[..1] {
                return (x, y);
            }
        }
    }
    panic!("{text:?} below row {y_from}");
}

#[test]
fn test_command_slot_styles_the_prompt() {
    use crossterm::event::{KeyCode, KeyModifiers};
    let buf = draw_with_slot_theme(&[(KeyCode::Char(':'), KeyModifiers::empty())], 60, 14);
    let (x, y) = find_text(&buf, "Command");
    assert_eq!(rgb(6), buf[(x, y)].fg);
    // The modality in the status line is lowercase, on the bottom border.
    let (mx, my) = find_text(&buf, "command");
    assert_eq!(rgb(5), buf[(mx, my)].fg);
}

#[test]
fn test_dialog_slots_style_border_title_and_hint() {
    use crossterm::event::{KeyCode, KeyModifiers};
    let buf = draw_with_slot_theme(&[(KeyCode::Char('h'), KeyModifiers::ALT)], 130, 20);
    let (x, y) = find_text(&buf, "┌");
    assert_eq!(rgb(9), buf[(x, y)].fg);
    let (tx, ty) = find_text(&buf, "Help");
    assert_eq!(rgb(2), buf[(tx, ty)].fg);
    let (hx, hy) = find_text(&buf, "j,k or");
    assert_eq!(rgb(8), buf[(hx, hy)].fg);
}

#[test]
fn test_edit_slot_survives_a_new_text_area() {
    use crossterm::event::{KeyCode, KeyModifiers};
    // Moving the cursor rebuilds the text area from the theme.
    let buf = draw_with_slot_theme(&[(KeyCode::Char('l'), KeyModifiers::empty())], 60, 14);
    let (_, hy) = find_text(&buf, "ALT-h");
    assert_eq!(rgb(7), buf[(3, hy)].fg);
    assert_eq!(rgb(1), buf[(2, hy)].fg);
}

#[test]
fn test_cells_outside_a_range_selection_keep_their_colors() {
    use ratatui::{backend::TestBackend, widgets::StatefulWidget, Terminal};

    use crate::ui::theme::{parse, Slot, Theme};

    let mut book = Book::from_model(
        Model::new_empty("test", "en", "America/New_York", "en").expect("Failed to make model"),
    );
    let at = |row, col| Address { sheet: 0, row, col };
    book.update_cell(&at(3, 2), "-4").expect("set B3");
    book.update_cell(&at(3, 3), "text").expect("set C3");
    book.evaluate();
    let (theme, warnings) = parse(
        "style cell fg white bg #101010\n\
         style cell.negative fg red\n\
         style cell.string fg green\n\
         style selection fg black bg lightblue\n",
        &Theme::blank("t"),
    );
    assert!(warnings.is_empty(), "{warnings:?}");
    let range = crate::ui::RangeSelection {
        start: Some(at(5, 2)),
        end: Some(at(6, 3)),
        ..Default::default()
    };
    let width = book.get_col_size(1).expect("col size") as u16;
    let viewport = Viewport::new(&book, Some(&range), &theme).with_selected(at(1, 1));
    let mut term = Terminal::new(TestBackend::new(5 + width * 6, 10)).expect("terminal");
    term.draw(|f| {
        let mut state = ViewportState::default();
        viewport.render(f.area(), f.buffer_mut(), &mut state);
    })
    .expect("draw");
    let buf = term.backend().buffer();
    let cell = |row: u16, col: u16| &buf[(5 + width * (col - 1), row)];
    // Outside the range: type colors and the cell slot survive.
    assert_eq!(Color::Red, cell(3, 2).fg);
    assert_eq!(Color::Green, cell(3, 3).fg);
    assert_eq!(Color::Rgb(16, 16, 16), cell(3, 2).bg);
    assert_eq!(theme.get(Slot::Cell).bg, Some(cell(4, 4).bg));
    // Inside the range: the selection colors.
    assert_eq!(Color::LightBlue, cell(5, 2).bg);
    assert_eq!(Color::Black, cell(6, 3).fg);
}
