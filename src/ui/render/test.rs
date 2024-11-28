use ironcalc::base::Model;

use super::{Address, Book, Viewport, ViewportState};

#[test]
fn test_viewport_get_visible_columns() {
    let mut state = ViewportState::default();
    let book = Book::new(Model::new_empty("test", "en", "America/New_York").expect("Failed to make model"));
    let default_size = book.get_col_size(1).expect("Failed to get column size");
    let width = dbg!(dbg!(default_size) * 12 / 2);
    let viewport = Viewport::new(&book).with_selected(Address { row: 1, col: 17 });
    let cols = viewport.get_visible_columns((width + 5) as u16, &mut state).expect("Failed to get visible columns");
    assert_eq!(5, cols.len());
    assert_eq!(17, cols.last().expect("Failed to get last column").idx);
}

#[test]
fn test_viewport_get_visible_rows() {
    let mut state = dbg!(ViewportState::default());
    let book = Book::new(Model::new_empty("test", "en", "America/New_York").expect("Failed to make model"));
    let height = 6;
    let viewport = Viewport::new(&book).with_selected(Address { row: 17, col: 1 });
    let rows = dbg!(viewport.get_visible_rows(height as u16, &mut state));
    assert_eq!(height - 1, rows.len());
    assert_eq!(17 - (height - 2), *rows.first().expect("Failed to get first row"));
    assert_eq!(17, *rows.last().expect("Failed to get last row"));
}

#[test]
fn test_viewport_visible_columns_after_length_change() {
    let mut state = ViewportState::default();
    let mut book = Book::new(Model::new_empty("test", "en", "America/New_York").expect("Failed to make model"));
    let default_size = book.get_col_size(1).expect("Failed to get column size");
    let width = dbg!(dbg!(default_size) * 12 / 2);
    {
        let viewport = Viewport::new(&book).with_selected(Address { row: 1, col: 17 });
        let cols = viewport.get_visible_columns((width + 5) as u16, &mut state).expect("Failed to get visible columns");
        assert_eq!(5, cols.len());
        assert_eq!(17, cols.last().expect("Failed to get last column").idx);
    }

    book.set_col_size(1, default_size * 6).expect("Failed to set column size");
    {
        let viewport = Viewport::new(&book).with_selected(Address { row: 1, col: 1 });
        let cols = viewport.get_visible_columns((width + 5) as u16, &mut state).expect("Failed to get visible columns");
        assert_eq!(1, cols.len());
        assert_eq!(1, cols.last().expect("Failed to get last column").idx);
    }
}
