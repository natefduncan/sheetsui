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

#[test]
fn test_draw_shows_pending_keys_in_the_bottom_title() {
    use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
    use ratatui::{backend::TestBackend, Terminal};

    let mut ws = crate::ui::Workspace::new_empty("en", "America/New_York").expect("workspace");
    let bottom_row = |ws: &mut crate::ui::Workspace| {
        let mut term = Terminal::new(TestBackend::new(60, 14)).expect("terminal");
        term.draw(|f| super::draw(f, ws)).expect("draw");
        let buf = term.backend().buffer().clone();
        (0..buf.area.width)
            .map(|x| buf[(x, buf.area.height - 1)].symbol().to_string())
            .collect::<String>()
    };
    assert!(!bottom_row(&mut ws).contains(" g "));
    ws.handle_input(Event::Key(KeyEvent::new(KeyCode::Char('g'), KeyModifiers::empty())))
        .expect("handle key");
    let row = bottom_row(&mut ws);
    assert!(row.contains(" g "), "pending key missing from {row:?}");
    assert!(row.contains("navigate"), "mode missing from {row:?}");
    // And it goes away when the sequence ends.
    ws.handle_input(Event::Key(KeyEvent::new(KeyCode::Char('g'), KeyModifiers::empty())))
        .expect("handle key");
    assert!(!bottom_row(&mut ws).contains(" g "));
}

/// Draws a workspace on `x,y,x / z,x,z` after `commands` and returns the
/// background of each cell in rows 1-2, columns 1-3. A command that starts
/// with `/` or `:` is ended with Enter; any other is typed as keys.
fn search_backgrounds(commands: &[&str]) -> Vec<Vec<Color>> {
    use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
    use ratatui::{backend::TestBackend, Terminal};

    let book = Book::from_csv_reader("x,y,x\nz,x,z\n".as_bytes(), None).expect("csv");
    let mut ws = crate::ui::Workspace::new(book, std::path::PathBuf::from("s.sui"));
    let width = ws.book.get_col_size(1).expect("col size") as u16;
    for command in commands {
        let mut keys: Vec<KeyCode> = command.chars().map(KeyCode::Char).collect();
        if command.starts_with(['/', ':']) {
            keys.push(KeyCode::Enter);
        }
        for key in keys {
            ws.handle_input(Event::Key(KeyEvent::new(key, KeyModifiers::empty())))
                .expect("handle key");
        }
    }
    let mut term = Terminal::new(TestBackend::new(80, 20)).expect("terminal");
    term.draw(|f| super::draw(f, &mut ws)).expect("draw");
    let buf = term.backend().buffer().clone();
    // Row 1 starts after the sheet's border and the 5-wide row label.
    let (x, y) = find_text(&buf, "│1");
    let x0 = x + 1 + 5;
    (0..2)
        .map(|row| {
            (0..3)
                .map(|col| buf[(x0 + width * col, y + row)].bg)
                .collect()
        })
        .collect()
}

#[test]
fn test_hlsearch_draws_matches_with_the_search_slots() {
    use crate::ui::theme::{Slot, Theme};
    let theme = Theme::default();
    let bg = |slot| theme.get(slot).bg.expect("bg");
    let (search, current, cell, cursor) =
        (bg(Slot::Search), bg(Slot::SearchCurrent), bg(Slot::Cell), bg(Slot::Cursor));

    // Before a search nothing is highlighted.
    assert_eq!(
        vec![vec![cursor, cell, cell], vec![cell, cell, cell]],
        search_backgrounds(&[])
    );
    // `/x` moves to C1, which is the current match.
    assert_eq!(
        vec![vec![search, cell, current], vec![cell, search, cell]],
        search_backgrounds(&["/x"])
    );
    // `:noh` hides it; the cursor stays on C1.
    assert_eq!(
        vec![vec![cell, cell, cursor], vec![cell, cell, cell]],
        search_backgrounds(&["/x", ":noh"])
    );
    // So does `nohlsearch`.
    assert_eq!(
        vec![vec![cell, cell, cursor], vec![cell, cell, cell]],
        search_backgrounds(&["/x", ":set nohls"])
    );
    // The cursor on a cell that does not match keeps the cursor style.
    assert_eq!(
        vec![vec![search, cursor, search], vec![cell, search, cell]],
        search_backgrounds(&["/x", "h"])
    );
}

#[test]
fn test_viewport_frozen_rows_stay_while_scrolled() {
    let mut book = Book::from_model(
        Model::new_empty("test", "en", "America/New_York", "en").expect("Failed to make model"),
    );
    book.set_frozen(0, 2, 0).unwrap();
    let app_state = AppState::default();
    let at = |row| Address { sheet: 0, row, col: 1 };
    let viewport = Viewport::new(&book, Some(&app_state.range_select), &app_state.theme)
        .with_selected(at(500));
    let rows = viewport.get_visible_rows(8, &ViewportState::default());
    // Header, two frozen rows and the line leave four rows to scroll.
    assert_eq!(vec![1, 2, 497, 498, 499, 500], rows);
    // Unscrolled, the cursor in the frozen rows shows no row twice.
    let viewport = Viewport::new(&book, Some(&app_state.range_select), &app_state.theme)
        .with_selected(at(1));
    let rows = viewport.get_visible_rows(8, &ViewportState::default());
    assert_eq!(vec![1, 2, 3, 4, 5, 6], rows);
}

#[test]
fn test_viewport_frozen_rows_render_under_the_header() {
    use ratatui::{backend::TestBackend, Terminal};
    use ratatui::widgets::StatefulWidget;
    let mut book = Book::from_model(
        Model::new_empty("test", "en", "America/New_York", "en").expect("Failed to make model"),
    );
    book.set_frozen(0, 1, 0).unwrap();
    book.update_cell(&Address { sheet: 0, row: 1, col: 1 }, "head").unwrap();
    book.update_cell(&Address { sheet: 0, row: 50, col: 1 }, "far").unwrap();
    book.move_to(&Address { sheet: 0, row: 50, col: 1 }).unwrap();
    book.evaluate();
    let app_state = AppState::default();
    let viewport = Viewport::new(&book, Some(&app_state.range_select), &app_state.theme)
        .with_selected(book.location.clone());
    let mut term = Terminal::new(TestBackend::new(30, 8)).expect("terminal");
    term.draw(|f| {
        let mut state = ViewportState::default();
        viewport.render(f.area(), f.buffer_mut(), &mut state);
    })
    .expect("draw");
    let buf = term.backend().buffer();
    let line = |y: u16| (0..30).map(|x| buf[(x, y)].symbol().to_string()).collect::<String>();
    let all: Vec<String> = (0..8).map(line).collect();
    assert!(line(1).starts_with("1") && line(1).contains("head"), "{all:#?}");
    let all: Vec<String> = (0..8).map(line).collect();
    assert!(line(5).starts_with("50") && line(5).contains("far"), "{all:#?}");
    assert!(!line(2).starts_with("2 "), "row 2 should be scrolled away: {all:#?}");
}

#[test]
fn test_viewport_frozen_columns_stay_while_scrolled() {
    let mut book = Book::from_model(
        Model::new_empty("test", "en", "America/New_York", "en").expect("Failed to make model"),
    );
    book.set_frozen(0, 0, 1).unwrap();
    let app_state = AppState::default();
    let at = |col| Address { sheet: 0, row: 1, col };
    let idxs = |col, width| {
        let viewport = Viewport::new(&book, Some(&app_state.range_select), &app_state.theme)
            .with_selected(at(col));
        viewport
            .get_visible_columns(width, &ViewportState::default())
            .unwrap()
            .iter()
            .map(|c| c.idx)
            .collect::<Vec<_>>()
    };
    let far = idxs(40, 80);
    assert_eq!(1, far[0], "{far:?}");
    assert!(far[1] > 20, "{far:?}");
    assert_eq!(Some(&40), far.last());
    // Unscrolled, no column shows twice.
    let near = idxs(1, 80);
    assert_eq!(&[1, 2, 3], &near[..3]);
    // A freeze wider than the screen leaves one column to scroll.
    book.set_frozen(0, 0, 30).unwrap();
    let viewport = Viewport::new(&book, Some(&app_state.range_select), &app_state.theme)
        .with_selected(at(35));
    let clipped = viewport.get_visible_columns(60, &ViewportState::default()).unwrap();
    assert!(clipped.len() >= 2 && clipped.last().unwrap().idx == 35, "{clipped:?}");
}

#[test]
fn test_viewport_frozen_columns_render_header_and_corner() {
    use ratatui::{backend::TestBackend, Terminal};
    use ratatui::widgets::StatefulWidget;
    let mut book = Book::from_model(
        Model::new_empty("test", "en", "America/New_York", "en").expect("Failed to make model"),
    );
    book.set_frozen(0, 2, 1).unwrap();
    book.update_cell(&Address { sheet: 0, row: 1, col: 1 }, "corner").unwrap();
    book.update_cell(&Address { sheet: 0, row: 3, col: 1 }, "key").unwrap();
    book.update_cell(&Address { sheet: 0, row: 1, col: 30 }, "head").unwrap();
    book.move_to(&Address { sheet: 0, row: 3, col: 30 }).unwrap();
    book.evaluate();
    let app_state = AppState::default();
    let viewport = Viewport::new(&book, Some(&app_state.range_select), &app_state.theme)
        .with_selected(book.location.clone());
    let mut term = Terminal::new(TestBackend::new(80, 8)).expect("terminal");
    term.draw(|f| {
        let mut state = ViewportState::default();
        viewport.render(f.area(), f.buffer_mut(), &mut state);
    })
    .expect("draw");
    let buf = term.backend().buffer();
    let line = |y: u16| (0..80).map(|x| buf[(x, y)].symbol().to_string()).collect::<String>();
    let all: Vec<String> = (0..8).map(line).collect();
    // Header: pinned A, then the scrolled-to columns.
    assert!(line(0).trim_start().starts_with("A"), "{all:#?}");
    // Corner and pinned column keep showing.
    assert!(line(1).contains("corner") && line(1).contains("head"), "{all:#?}");
    assert!(line(4).contains("key"), "{all:#?}");
}

fn draw_lines(book: &Book, w: u16, h: u16) -> Vec<String> {
    use ratatui::{backend::TestBackend, Terminal};
    use ratatui::widgets::StatefulWidget;
    let app_state = AppState::default();
    let viewport = Viewport::new(&book, Some(&app_state.range_select), &app_state.theme)
        .with_selected(book.location.clone());
    let mut term = Terminal::new(TestBackend::new(w, h)).expect("terminal");
    term.draw(|f| {
        let mut state = ViewportState::default();
        viewport.render(f.area(), f.buffer_mut(), &mut state);
    })
    .expect("draw");
    let buf = term.backend().buffer();
    (0..h)
        .map(|y| (0..w).map(|x| buf[(x, y)].symbol().to_string()).collect::<String>())
        .collect()
}

#[test]
fn test_freeze_line_position_and_style() {
    let mut book = Book::from_model(
        Model::new_empty("test", "en", "America/New_York", "en").expect("Failed to make model"),
    );
    book.set_frozen(0, 2, 1).unwrap();
    book.evaluate();
    let lines = draw_lines(&book, 60, 14);
    // Header, rows 1-2, then the rule, then row 3.
    assert!(lines[3].starts_with("─────"), "{lines:#?}");
    assert!(lines[3].contains('┼'), "{lines:#?}");
    assert!(lines[2].starts_with("2"), "{lines:#?}");
    assert!(lines[4].starts_with("3"), "{lines:#?}");
    // The vertical line sits right after the 5 wide row numbers and column A.
    let col = lines[1].chars().position(|c| c == '│').expect("vertical line");
    assert_eq!(5 + book.get_col_size(1).unwrap(), col, "{lines:#?}");
    assert_eq!(Some(col), lines[0].chars().position(|c| c == '│'));
    assert_eq!(Some(col), lines[3].chars().position(|c| c == '┼'));
    // Rows only: a rule and no vertical line.
    book.set_frozen(0, 1, 0).unwrap();
    let lines = draw_lines(&book, 60, 14);
    assert!(lines[2].starts_with("─────") && !lines[2].contains('┼'), "{lines:#?}");
    assert!(lines.iter().all(|l| !l.contains('│')), "{lines:#?}");
    // Columns only: a vertical line and no rule.
    book.set_frozen(0, 0, 1).unwrap();
    let lines = draw_lines(&book, 60, 14);
    assert!(lines.iter().all(|l| !l.contains('─')), "{lines:#?}");
    assert!(lines[1].contains('│'), "{lines:#?}");
}

#[test]
fn test_no_freeze_draws_no_line_and_takes_no_space() {
    let mut book = Book::from_model(
        Model::new_empty("test", "en", "America/New_York", "en").expect("Failed to make model"),
    );
    book.evaluate();
    let lines = draw_lines(&book, 60, 14);
    assert!(lines.iter().all(|l| !l.contains('─') && !l.contains('│')), "{lines:#?}");
    assert!(lines[1].starts_with("1") && lines[2].starts_with("2"), "{lines:#?}");
}

#[test]
fn test_freeze_line_uses_the_theme_slot() {
    use crate::ui::theme::Slot;
    use ratatui::style::{Color, Style};
    use ratatui::{backend::TestBackend, Terminal};
    use ratatui::widgets::StatefulWidget;
    let mut book = Book::from_model(
        Model::new_empty("test", "en", "America/New_York", "en").expect("Failed to make model"),
    );
    book.set_frozen(0, 1, 0).unwrap();
    book.evaluate();
    let mut theme = crate::ui::theme::Theme::default();
    let (t, w) = crate::ui::theme::parse("style freeze.line fg #123456", &theme);
    assert!(w.is_empty());
    theme = t;
    assert_ne!(Style::default(), theme.get(Slot::FreezeLine));
    let app_state = AppState::default();
    let viewport = Viewport::new(&book, Some(&app_state.range_select), &theme)
        .with_selected(book.location.clone());
    let mut term = Terminal::new(TestBackend::new(40, 10)).expect("terminal");
    term.draw(|f| {
        let mut state = ViewportState::default();
        viewport.render(f.area(), f.buffer_mut(), &mut state);
    })
    .expect("draw");
    assert_eq!(Some(Color::Rgb(0x12, 0x34, 0x56)), term.backend().buffer()[(8, 2)].style().fg);
}

#[test]
fn test_builtin_themes_define_the_freeze_line() {
    use crate::ui::theme::Slot;
    use ratatui::style::Style;
    assert_eq!(5, crate::ui::theme::lookup::BUILTIN.len());
    for (name, _) in crate::ui::theme::lookup::BUILTIN {
        let (theme, warnings) = crate::ui::theme::lookup::load(name, None).unwrap();
        assert!(warnings.is_empty(), "{name}");
        assert_ne!(Style::default(), theme.get(Slot::FreezeLine), "{name}");
    }
}

#[test]
fn test_frozen_cells_never_change_the_scroll() {
    let mut book = Book::from_model(
        Model::new_empty("test", "en", "America/New_York", "en").expect("Failed to make model"),
    );
    book.set_frozen(0, 2, 1).unwrap();
    let app_state = AppState::default();
    let mut state = ViewportState::default();
    let mut frame = |row, col| {
        let viewport = Viewport::new(&book, Some(&app_state.range_select), &app_state.theme)
            .with_selected(Address { sheet: 0, row, col });
        let rows = viewport.get_visible_rows(8, &state);
        let cols: Vec<usize> = viewport
            .get_visible_columns(80, &state)
            .unwrap()
            .iter()
            .map(|c| c.idx)
            .collect();
        // Settle the scroll the way a render does.
        viewport.to_table(80, 8, &mut state).unwrap();
        (rows, cols)
    };
    let (far_rows, far_cols) = frame(500, 40);
    assert_eq!(vec![1, 2, 497, 498, 499, 500], far_rows);
    // Jumping to A1 (`gg`) and walking the frozen cells does not scroll.
    for (row, col) in [(1, 1), (2, 1), (2, 40), (500, 1), (1, 40), (1, 1)] {
        let (rows, cols) = frame(row, col);
        assert_eq!(far_rows[2..], rows[2..], "rows moved at ({row},{col})");
        assert_eq!(far_cols[1..], cols[1..], "cols moved at ({row},{col})");
    }
    // A jump to a scrolling cell scrolls the least that reveals it.
    let (rows, _) = frame(3, 1);
    assert_eq!(vec![1, 2, 3, 4, 5, 6], rows);
    // No row or column ever shows twice.
    let (rows, cols) = frame(1, 1);
    let mut r = rows.clone();
    r.dedup();
    assert_eq!(rows, r);
    let mut c = cols.clone();
    c.dedup();
    assert_eq!(cols, c);
}

#[test]
fn test_range_select_highlights_both_sides_of_the_freeze() {
    use crate::ui::{theme::Slot, RangeSelection};
    use ratatui::{backend::TestBackend, widgets::StatefulWidget, Terminal};
    let mut book = Book::from_model(
        Model::new_empty("test", "en", "America/New_York", "en").expect("Failed to make model"),
    );
    book.set_frozen(0, 1, 0).unwrap();
    book.move_to(&Address { sheet: 0, row: 50, col: 2 }).unwrap();
    let app_state = AppState::default();
    let sel = RangeSelection {
        start: Some(Address { sheet: 0, row: 1, col: 1 }),
        end: Some(Address { sheet: 0, row: 50, col: 1 }),
        ..Default::default()
    };
    let viewport = Viewport::new(&book, Some(&sel), &app_state.theme)
        .with_selected(book.location.clone());
    let mut term = Terminal::new(TestBackend::new(60, 14)).expect("terminal");
    term.draw(|f| {
        viewport.render(f.area(), f.buffer_mut(), &mut ViewportState::default());
    })
    .expect("draw");
    let buf = term.backend().buffer();
    let want = app_state.theme.get(Slot::Selection).bg.expect("bg");
    // Row 1 sits under the header; the last scrolling row is row 50.
    assert_eq!(want, buf[(6, 1)].bg, "frozen row 1 not highlighted");
    assert_eq!(want, buf[(6, 11)].bg, "scrolling row 50 not highlighted");
    assert_ne!(want, buf[(26, 5)].bg, "a cell outside the range was highlighted");
}
