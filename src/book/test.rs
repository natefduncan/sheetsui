use ironcalc::base::{expressions::types::Area, worksheet::WorksheetDimension, Model};

use crate::ui::Address;

use super::{
    shrink_freeze,
    search::{Direction, SearchIn},
    change_decimals, check_number_format, StyleAttr, StyleFlag, number_format_preset, DecimalsChange, AutofitReport, Book, FileFormat, DEFAULT_COL_WIDTH, LAST_COLUMN, LAST_ROW,
};
use ironcalc::base::UserModel;

#[test]
fn test_book_default() {
    let mut book = Book::default();
    let WorksheetDimension {
        min_row,
        max_row,
        min_column,
        max_column,
    } = book.get_dimensions().expect("couldn't get dimensions");
    assert_eq!((1, 1, 1, 1), (min_row, max_row, min_column, max_column));
    assert_eq!((1, 1), book.get_size().expect("Failed to get size"));
    let cell = book
        .get_current_cell_contents()
        .expect("couldn't get contents");
    assert_eq!("", cell);
    book.edit_current_cell("1").expect("failed to edit cell");
    book.evaluate();
    let cell = book
        .get_current_cell_contents()
        .expect("couldn't get contents");
    assert_eq!("1", cell);
    let cell = book
        .get_current_cell_rendered()
        .expect("couldn't get contents");
    assert_eq!("1", cell);
    let sheets = book.get_all_sheets_identifiers();
    assert_eq!(1, sheets.len());
}

#[test]
fn test_book_insert_cell_new_row() {
    let mut book = Book::default();
    book.update_cell(
        &Address {
            sheet: 0,
            row: 2,
            col: 1,
        },
        "1",
    )
    .expect("failed to edit cell");
    book.evaluate();
    let WorksheetDimension {
        min_row,
        max_row,
        min_column,
        max_column,
    } = book.get_dimensions().expect("couldn't get dimensions");
    assert_eq!((1, 2, 1, 1), (min_row, max_row, min_column, max_column));
    assert_eq!((2, 1), book.get_size().expect("Failed to get size"));
}

#[test]
fn test_book_insert_cell_new_column() {
    let mut book = Book::default();
    book.update_cell(
        &Address {
            sheet: 0,
            row: 1,
            col: 2,
        },
        "1",
    )
    .expect("failed to edit cell");
    let WorksheetDimension {
        min_row,
        max_row,
        min_column,
        max_column,
    } = book.get_dimensions().expect("couldn't get dimensions");
    assert_eq!((1, 1, 1, 2), (min_row, max_row, min_column, max_column));
    assert_eq!((1, 2), book.get_size().expect("Failed to get size"));
}

#[test]
fn test_book_insert_rows() {
    let mut book = Book::default();
    book.update_cell(
        &Address {
            sheet: 0,
            row: 2,
            col: 2,
        },
        "1",
    )
    .expect("failed to edit cell");
    book.move_to(&Address {
        sheet: 0,
        row: 2,
        col: 2,
    })
    .expect("Failed to move to location");
    assert_eq!((2, 2), book.get_size().expect("Failed to get size"));
    book.insert_rows(1, 5).expect("Failed to insert rows");
    assert_eq!((7, 2), book.get_size().expect("Failed to get size"));
    assert_eq!(
        Address {
            sheet: 0,
            row: 7,
            col: 2
        },
        book.location
    );
    assert_eq!(
        "1",
        book.get_current_cell_rendered()
            .expect("Failed to get rendered content")
    );
}

#[test]
fn test_book_insert_columns() {
    let mut book = Book::default();
    book.update_cell(
        &Address {
            sheet: 0,
            row: 2,
            col: 2,
        },
        "1",
    )
    .expect("failed to edit cell");
    book.move_to(&Address {
        sheet: 0,
        row: 2,
        col: 2,
    })
    .expect("Failed to move to location");
    assert_eq!((2, 2), book.get_size().expect("Failed to get size"));
    book.insert_columns(1, 5).expect("Failed to insert rows");
    assert_eq!((2, 7), book.get_size().expect("Failed to get size"));
    assert_eq!(
        Address {
            sheet: 0,
            row: 2,
            col: 7
        },
        book.location
    );
    assert_eq!(
        "1",
        book.get_current_cell_rendered()
            .expect("Failed to get rendered content")
    );
}

#[test]
fn test_book_col_size() {
    let mut book = Book::default();
    book.update_cell(
        &Address {
            sheet: 0,
            row: 2,
            col: 2,
        },
        "1",
    )
    .expect("failed to edit cell");
    book.set_col_size(1, 20).expect("Failed to set column size");
    assert_eq!(20, book.get_col_size(1).expect("Failed to get column size"));
}

#[test]
fn test_book_get_exportable_rows() {
    let mut book = Book::default();
    book.update_cell(
        &Address {
            sheet: 0,
            row: 1,
            col: 3,
        },
        "1-3",
    )
    .expect("failed to edit cell");
    book.update_cell(
        &Address {
            sheet: 0,
            row: 3,
            col: 6,
        },
        "3-6",
    )
    .expect("failed to edit cell");

    let rows = book.get_export_rows().expect("Failed to get export rows");
    assert_eq!(3, rows.len());
    assert_eq!(
        rows,
        vec![
            vec!["", "", "1-3", "", "", ""],
            vec!["", "", "", "", "", ""],
            vec!["", "", "", "", "", "3-6"],
        ]
    );
}

#[test]
fn test_sheet_to_clipboard_content() {
    let mut book = Book::default();
    book.update_cell(
        &Address {
            sheet: 0,
            row: 1,
            col: 1,
        },
        "A1",
    )
    .expect("failed to edit cell");
    book.update_cell(
        &Address {
            sheet: 0,
            row: 1,
            col: 2,
        },
        "B1",
    )
    .expect("failed to edit cell");
    book.update_cell(
        &Address {
            sheet: 0,
            row: 2,
            col: 1,
        },
        "A2",
    )
    .expect("failed to edit cell");
    book.update_cell(
        &Address {
            sheet: 0,
            row: 2,
            col: 2,
        },
        "B2",
    )
    .expect("failed to edit cell");
    
    let (html, csv) = dbg!(book.sheeet_to_clipboard_content(0).expect("Failed to get clipboard content"));
    
    // Check that HTML contains table elements and our data
    assert!(html.contains("<table>"));
    assert!(html.contains("<tr>"));
    assert!(html.contains("<td>"));
    assert!(html.contains("A1"));
    assert!(html.contains("B1"));
    assert!(html.contains("A2"));
    assert!(html.contains("B2"));
    
    // Check CSV content
    let expected_csv = "A1,B1\nA2,B2\n";
    assert_eq!(csv, expected_csv);
}

#[test]
fn test_range_to_clipboard_content() {
    let mut book = Book::default();
    book.update_cell(
        &Address {
            sheet: 0,
            row: 1,
            col: 1,
        },
        "A1",
    )
    .expect("failed to edit cell");
    book.update_cell(
        &Address {
            sheet: 0,
            row: 1,
            col: 2,
        },
        "B1",
    )
    .expect("failed to edit cell");
    book.update_cell(
        &Address {
            sheet: 0,
            row: 2,
            col: 1,
        },
        "A2",
    )
    .expect("failed to edit cell");
    book.update_cell(
        &Address {
            sheet: 0,
            row: 2,
            col: 2,
        },
        "B2",
    )
    .expect("failed to edit cell");
    
    let start = Address { sheet: 0, row: 1, col: 1 };
    let end = Address { sheet: 0, row: 2, col: 2 };
    let range = super::AddressRange { start: &start, end: &end };
    
    let (html, csv) = book.range_to_clipboard_content(range).expect("Failed to get clipboard content");
    
    // Check that HTML contains table elements and our data
    assert!(html.contains("<table>"));
    assert!(html.contains("<tr>"));
    assert!(html.contains("<td>"));
    assert!(html.contains("A1"));
    assert!(html.contains("B1"));
    assert!(html.contains("A2"));
    assert!(html.contains("B2"));
    
    // Check CSV content
    let expected_csv = "A1,B1\nA2,B2\n";
    assert_eq!(csv, expected_csv);
}

// -------------------------------------------------------------------------
// Phase 2: Book I/O Integration and Format Detection (REQ-005..REQ-008)
// -------------------------------------------------------------------------

fn phase2_addr(row: usize, col: usize) -> Address {
    Address { sheet: 0, row, col }
}

fn tmp_path(name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("sheetui_test_{name}"))
}

#[test]
fn test_new_book_default_format_is_sui() {
    // REQ-008: new/empty books default to SUI format.
    let book = Book::default();
    assert_eq!(
        book.get_format(),
        &FileFormat::Sui,
        "a new Book should default to FileFormat::Sui"
    );
    assert!(
        book.file_path.is_none(),
        "a new Book should have no file_path"
    );
}

#[test]
fn test_load_sui_file_by_extension() {
    // REQ-005: .sui extension → SUI parser.
    let path = tmp_path("load_sui.sui");
    let content = "[sheet \"TestSheet\"]\nA1 = \"loaded-from-sui\"\n[/sheet]\n";
    std::fs::write(&path, content).expect("write temp .sui");
    let book = Book::load(&path, "en", "America/New_York").expect("load .sui file");
    assert_eq!(
        book.get_format(),
        &FileFormat::Sui,
        "loading a .sui file should set format=Sui"
    );
    let val = book
        .get_cell_addr_contents(&phase2_addr(1, 1))
        .expect("get cell A1");
    assert_eq!(val, "loaded-from-sui");
    std::fs::remove_file(&path).ok();
}

#[test]
fn test_load_xlsx_file_by_extension() {
    // REQ-005: .xlsx extension → xlsx parser, format=Xlsx.
    let path = tmp_path("load_xlsx.xlsx");
    // Use the existing new_from_xlsx-style save path to create a real .xlsx
    let mut book = Book::default();
    book.update_cell(&phase2_addr(1, 1), "xlsx-load-test")
        .expect("set A1");
    book.evaluate();
    book.save_to_xlsx(&path.to_string_lossy()).expect("save_to_xlsx");
    // Now load via the new format-detecting Book::load
    let loaded = Book::load(&path, "en", "America/New_York").expect("load .xlsx");
    assert_eq!(
        loaded.get_format(),
        &FileFormat::Xlsx,
        "loading a .xlsx file should set format=Xlsx"
    );
    let val = loaded
        .get_cell_addr_contents(&phase2_addr(1, 1))
        .expect("get A1 after load");
    assert_eq!(val, "xlsx-load-test");
    std::fs::remove_file(&path).ok();
}

#[test]
fn test_save_as_sui_round_trip() {
    // REQ-006 / REQ-007: save_as writes the file; reloading it returns the same content.
    let path = tmp_path("save_as_sui.sui");
    let mut book = Book::default();
    book.update_cell(&phase2_addr(1, 1), "save-round-trip")
        .expect("set cell");
    book.evaluate();
    book.save_as(&path).expect("save_as .sui");
    let book2 = Book::load(&path, "en", "America/New_York").expect("reload .sui");
    let val = book2
        .get_cell_addr_contents(&phase2_addr(1, 1))
        .expect("get cell after reload");
    assert_eq!(val, "save-round-trip", "cell must survive save_as/load round-trip");
    std::fs::remove_file(&path).ok();
}

#[test]
fn test_save_updates_file_in_place() {
    // REQ-006: Book::save() overwrites the current file_path.
    let path = tmp_path("save_in_place.sui");
    let mut book = Book::default();
    book.update_cell(&phase2_addr(1, 1), "initial")
        .expect("set cell");
    book.evaluate();
    book.save_as(&path).expect("initial save_as");
    // Update and save in-place
    book.update_cell(&phase2_addr(1, 1), "updated")
        .expect("update cell");
    book.evaluate();
    book.save().expect("save in-place");
    let book2 = Book::load(&path, "en", "America/New_York").expect("reload");
    let val = book2
        .get_cell_addr_contents(&phase2_addr(1, 1))
        .expect("get cell");
    assert_eq!(val, "updated", "save() must overwrite the file in place");
    std::fs::remove_file(&path).ok();
}

#[test]
fn test_save_as_updates_stored_format_to_xlsx() {
    // REQ-007: save_as with a .xlsx path updates format to Xlsx.
    let path = tmp_path("format_change.xlsx");
    let mut book = Book::default();
    book.update_cell(&phase2_addr(1, 1), "format-change")
        .expect("set cell");
    book.evaluate();
    assert_eq!(book.get_format(), &FileFormat::Sui);
    book.save_as(&path).expect("save_as .xlsx");
    assert_eq!(
        book.get_format(),
        &FileFormat::Xlsx,
        "save_as .xlsx must update stored format to Xlsx"
    );
    std::fs::remove_file(&path).ok();
}

#[test]
fn test_save_as_subsequent_save_uses_new_format() {
    // REQ-007: after save_as changes the format, a bare save() uses the new format.
    let path_sui = tmp_path("subsequent_sui.sui");
    let path_xlsx = tmp_path("subsequent_xlsx.xlsx");
    let mut book = Book::default();
    book.update_cell(&phase2_addr(1, 1), "step1").expect("set");
    book.evaluate();
    book.save_as(&path_sui).expect("save as .sui");
    book.save_as(&path_xlsx).expect("save as .xlsx — changes format");
    book.update_cell(&phase2_addr(1, 1), "step2").expect("update");
    book.evaluate();
    book.save().expect("save with updated format");
    // The save should have written the xlsx file
    let book2 =
        Book::load(&path_xlsx, "en", "America/New_York").expect("reload .xlsx");
    let val = book2
        .get_cell_addr_contents(&phase2_addr(1, 1))
        .expect("get cell");
    assert_eq!(
        val, "step2",
        "save() must use the format last set by save_as"
    );
    std::fs::remove_file(&path_sui).ok();
    std::fs::remove_file(&path_xlsx).ok();
}

#[test]
fn test_save_without_path_returns_err() {
    // REQ-006: save() with no file_path must return Err, not panic.
    let mut book = Book::default();
    book.update_cell(&phase2_addr(1, 1), "no-path")
        .expect("set cell");
    let result = book.save();
    assert!(
        result.is_err(),
        "save() with no file_path must return Err, got: Ok"
    );
}

#[test]
fn test_cross_format_conversion_xlsx_to_sui() {
    // REQ-010 Phase 2: open xlsx (via save_to_xlsx), convert to .sui, reload, compare cells.
    let path_xlsx = tmp_path("cross_format.xlsx");
    let path_sui = tmp_path("cross_format.sui");
    // Build a book and save as xlsx using the existing save_to_xlsx method
    let mut book = Book::default();
    book.update_cell(&phase2_addr(1, 1), "cell-a1").expect("A1");
    book.update_cell(&phase2_addr(1, 2), "42").expect("B1");
    book.update_cell(&phase2_addr(2, 1), "=SUM(A1:B1)").expect("A2 formula");
    book.evaluate();
    book.save_to_xlsx(&path_xlsx.to_string_lossy()).expect("save xlsx");
    // Load the xlsx via format-detecting Book::load, then convert to sui
    let mut book2 =
        Book::load(&path_xlsx, "en", "America/New_York").expect("load .xlsx");
    book2.save_as(&path_sui).expect("save_as .sui");
    // Reload the .sui and verify cells survived
    let book3 =
        Book::load(&path_sui, "en", "America/New_York").expect("load .sui");
    let a1 = book3.get_cell_addr_contents(&phase2_addr(1, 1)).expect("A1");
    let b1 = book3.get_cell_addr_contents(&phase2_addr(1, 2)).expect("B1");
    let a2 = book3.get_cell_addr_contents(&phase2_addr(2, 1)).expect("A2");
    assert_eq!(a1, "cell-a1", "A1 must survive xlsx→sui conversion");
    assert_eq!(b1, "42", "B1 must survive xlsx→sui conversion");
    assert!(
        a2.starts_with('=') && a2.contains("SUM"),
        "formula in A2 must survive xlsx→sui conversion, got: {a2}"
    );
    std::fs::remove_file(&path_xlsx).ok();
    std::fs::remove_file(&path_sui).ok();
}

// -------------------------------------------------------------------------
// CSV load and save
// -------------------------------------------------------------------------

fn csv_book(text: &str) -> Book {
    Book::from_csv_reader(text.as_bytes(), None).expect("parse csv")
}

fn rendered(book: &Book, row: usize, col: usize) -> String {
    book.get_cell_addr_rendered(&phase2_addr(row, col))
        .expect("render cell")
}

#[test]
fn test_csv_loads_rows_and_columns_from_a1() {
    let book = csv_book("a,b\nc,d\n");
    assert_eq!(rendered(&book, 1, 1), "a");
    assert_eq!(rendered(&book, 1, 2), "b");
    assert_eq!(rendered(&book, 2, 1), "c");
    assert_eq!(rendered(&book, 2, 2), "d");
    assert_eq!(book.get_format(), &FileFormat::Csv);
    assert!(!book.dirty, "a freshly loaded book is not dirty");
}

#[test]
fn test_csv_keeps_first_row_as_data() {
    let book = csv_book("name,qty\nfoo,1\n");
    assert_eq!(rendered(&book, 1, 1), "name");
    assert_eq!(rendered(&book, 2, 1), "foo");
}

#[test]
fn test_csv_accepts_ragged_rows() {
    let book = csv_book("a\nb,c,d\ne,f\n");
    assert_eq!(rendered(&book, 1, 1), "a");
    assert_eq!(rendered(&book, 2, 3), "d");
    assert_eq!(rendered(&book, 3, 2), "f");
}

#[test]
fn test_csv_numbers_are_numbers() {
    let mut book = csv_book("1,-2.5,+3\n");
    book.update_cell(&phase2_addr(2, 1), "=A1+B1+C1").expect("set formula");
    book.evaluate();
    assert_eq!(rendered(&book, 2, 1), "1.5");
}

#[test]
fn test_csv_formula_text_stays_literal() {
    let book = csv_book("=1+1,-A1,+B2,'quoted\n");
    assert_eq!(rendered(&book, 1, 1), "=1+1");
    assert_eq!(rendered(&book, 1, 2), "-A1");
    assert_eq!(rendered(&book, 1, 3), "+B2");
    assert_eq!(rendered(&book, 1, 4), "'quoted");
}

#[test]
fn test_csv_quoted_fields() {
    let book = csv_book("\"a,b\",\"line1\nline2\",\"say \"\"hi\"\"\"\n");
    assert_eq!(rendered(&book, 1, 1), "a,b");
    assert_eq!(rendered(&book, 1, 2), "line1\nline2");
    assert_eq!(rendered(&book, 1, 3), "say \"hi\"");
}

#[test]
fn test_csv_empty_input() {
    let book = csv_book("");
    assert_eq!(book.get_export_rows_for_sheet(0).expect("rows").len(), 0);
}

#[test]
fn test_load_csv_file_by_extension_names_sheet_from_stem() {
    let path = tmp_path("load_csv.CSV");
    std::fs::write(&path, "x,1\n").expect("write temp .csv");
    let book = Book::load(&path, "en", "America/New_York").expect("load .csv");
    assert_eq!(book.get_format(), &FileFormat::Csv);
    assert_eq!(book.get_sheet_names(), vec!["sheetui_test_load_csv".to_string()]);
    assert_eq!(rendered(&book, 1, 1), "x");
    std::fs::remove_file(&path).ok();
}

#[test]
fn test_csv_save_round_trip_is_stable() {
    let path = tmp_path("round_trip.csv");
    let content = "a,1\n=x,\"q,r\"\n";
    std::fs::write(&path, content).expect("write temp .csv");
    let mut book = Book::load(&path, "en", "America/New_York").expect("load .csv");
    book.save().expect("save .csv");
    assert_eq!(std::fs::read_to_string(&path).expect("read back"), content);
    std::fs::remove_file(&path).ok();
}

#[test]
fn test_csv_save_writes_first_sheet_values() {
    let path = tmp_path("save_values.csv");
    let mut book = Book::default();
    book.update_cell(&phase2_addr(1, 1), "2").expect("set A1");
    book.update_cell(&phase2_addr(1, 2), "=A1*3").expect("set B1");
    book.new_sheet(Some("Other")).expect("add sheet");
    book.update_cell(&Address { sheet: 1, row: 1, col: 1 }, "ignored")
        .expect("set Other!A1");
    book.evaluate();
    book.save_as(&path).expect("save_as .csv");
    assert_eq!(book.get_format(), &FileFormat::Csv);
    assert_eq!(std::fs::read_to_string(&path).expect("read back"), "2,6\n");
    std::fs::remove_file(&path).ok();
}

// -------------------------------------------------------------------------
// Sort rows
// -------------------------------------------------------------------------

fn key(col: usize, ascending: bool) -> super::sort::SortKey {
    super::sort::SortKey { col, ascending }
}

fn column(book: &Book, col: usize, rows: std::ops::RangeInclusive<usize>) -> Vec<String> {
    rows.map(|r| rendered(book, r, col)).collect()
}

#[test]
fn test_sort_rows_ascending_moves_whole_rows() {
    let mut book = csv_book("3,c\n1,a\n2,b\n");
    book.sort_rows(0, 1, 3, &[key(1, true)]).expect("sort");
    assert_eq!(column(&book, 1, 1..=3), vec!["1", "2", "3"]);
    assert_eq!(column(&book, 2, 1..=3), vec!["a", "b", "c"]);
    assert!(book.dirty);
}

#[test]
fn test_sort_rows_descending() {
    let mut book = csv_book("1\n3\n2\n");
    book.sort_rows(0, 1, 3, &[key(1, false)]).expect("sort");
    assert_eq!(column(&book, 1, 1..=3), vec!["3", "2", "1"]);
}

#[test]
fn test_sort_rows_only_touches_given_rows() {
    let mut book = csv_book("h\n3\n1\n2\nf\n");
    book.sort_rows(0, 2, 4, &[key(1, true)]).expect("sort");
    assert_eq!(column(&book, 1, 1..=5), vec!["h", "1", "2", "3", "f"]);
}

#[test]
fn test_sort_rows_multiple_keys() {
    let mut book = csv_book("1,a\n2,a\n1,b\n");
    book.sort_rows(0, 1, 3, &[key(1, true), key(2, false)])
        .expect("sort");
    assert_eq!(column(&book, 1, 1..=3), vec!["1", "1", "2"]);
    assert_eq!(column(&book, 2, 1..=3), vec!["b", "a", "a"]);
}

#[test]
fn test_sort_rows_is_stable() {
    let mut book = csv_book("1,x\n0,y\n1,z\n");
    book.sort_rows(0, 1, 3, &[key(1, true)]).expect("sort");
    assert_eq!(column(&book, 2, 1..=3), vec!["y", "x", "z"]);
}

#[test]
fn test_sort_rows_blanks_last_in_both_directions() {
    let mut book = csv_book("2,a\n,b\n1,c\n");
    book.sort_rows(0, 1, 3, &[key(1, true)]).expect("sort asc");
    assert_eq!(column(&book, 2, 1..=3), vec!["c", "a", "b"]);
    book.sort_rows(0, 1, 3, &[key(1, false)]).expect("sort desc");
    assert_eq!(column(&book, 2, 1..=3), vec!["a", "c", "b"]);
}

#[test]
fn test_sort_rows_mixed_types_use_excel_order() {
    let mut book = Book::default();
    for (row, value) in ["=1/0", "TRUE", "text", "5"].iter().enumerate() {
        book.update_cell(&phase2_addr(row + 1, 1), *value).expect("set");
    }
    book.evaluate();
    book.sort_rows(0, 1, 4, &[key(1, true)]).expect("sort asc");
    assert_eq!(column(&book, 1, 1..=4), vec!["5", "text", "TRUE", "#DIV/0!"]);
    book.sort_rows(0, 1, 4, &[key(1, false)]).expect("sort desc");
    assert_eq!(column(&book, 1, 1..=4), vec!["#DIV/0!", "TRUE", "text", "5"]);
}

#[test]
fn test_sort_rows_text_ignores_case() {
    let mut book = csv_book("b\nA\nc\n");
    book.sort_rows(0, 1, 3, &[key(1, true)]).expect("sort");
    assert_eq!(column(&book, 1, 1..=3), vec!["A", "b", "c"]);
}

#[test]
fn test_sort_rows_formulas_sort_by_value_and_translate() {
    let mut book = csv_book("3\n1\n2\n");
    for row in 1..=3 {
        book.update_cell(&phase2_addr(row, 2), format!("=A{row}*-1"))
            .expect("set formula");
    }
    book.evaluate();
    // Column B is -A, so ascending B is descending A.
    book.sort_rows(0, 1, 3, &[key(2, true)]).expect("sort");
    assert_eq!(column(&book, 1, 1..=3), vec!["3", "2", "1"]);
    assert_eq!(column(&book, 2, 1..=3), vec!["-3", "-2", "-1"]);
    for row in 1..=3 {
        assert_eq!(
            book.get_cell_addr_contents(&phase2_addr(row, 2)).expect("contents"),
            format!("=A{row}*-1")
        );
    }
}

#[test]
fn test_sort_rows_keeps_literal_text_as_text() {
    let mut book = csv_book("2,=1+1\n1,-A1\n");
    book.sort_rows(0, 1, 2, &[key(1, true)]).expect("sort");
    assert_eq!(column(&book, 2, 1..=2), vec!["-A1", "=1+1"]);
}

#[test]
fn test_sort_rows_moves_cell_styles() {
    let mut book = csv_book("2\n1\n");
    let area = Area { sheet: 0, row: 1, column: 1, width: 1, height: 1 };
    book.set_cell_style(&[("font.b", "true")], &area).expect("bold");
    book.sort_rows(0, 1, 2, &[key(1, true)]).expect("sort");
    let bold = |row| book.get_cell_style(&phase2_addr(row, 1)).map(|s| s.font.b);
    assert_eq!(bold(1), Some(false));
    assert_eq!(bold(2), Some(true));
}

#[test]
fn test_sort_rows_does_not_move_row_height() {
    let mut book = csv_book("2\n1\n");
    book.model.set_rows_height(0, 1, 1, 40.0).expect("set height");
    let default_height = book.model.get_row_height(0, 2).expect("height");
    book.sort_rows(0, 1, 2, &[key(1, true)]).expect("sort");
    assert_eq!(book.model.get_row_height(0, 1).expect("height"), 40.0);
    assert_eq!(book.model.get_row_height(0, 2).expect("height"), default_height);
}

#[test]
fn test_sort_rows_already_sorted_is_not_a_change() {
    let mut book = csv_book("1\n2\n");
    book.sort_rows(0, 1, 2, &[key(1, true)]).expect("sort");
    assert!(!book.dirty);
}

#[test]
fn test_sort_rows_duplicate_key_is_an_error() {
    let mut book = csv_book("2,a\n1,b\n");
    let err = book
        .sort_rows(0, 1, 2, &[key(2, true), key(1, true), key(2, false)])
        .expect_err("duplicate key");
    assert_eq!(err.to_string(), "Column B is given more than one time.");
    assert_eq!(column(&book, 1, 1..=2), vec!["2", "1"]);
}

#[test]
fn test_sort_rows_merged_cells_are_an_error() {
    let mut model =
        Model::new_empty("test", "en", "America/New_York", "en").expect("model");
    model.workbook.worksheets[0].merge_cells.push("A2:B3".to_string());
    let mut book = Book::from_model(model);
    for (row, value) in ["3", "1", "2"].iter().enumerate() {
        book.update_cell(&phase2_addr(row + 1, 1), *value).expect("set");
    }
    book.evaluate();
    let err = book.sort_rows(0, 1, 3, &[key(1, true)]).expect_err("merged");
    assert_eq!(err.to_string(), "Cannot sort a range that contains merged cells.");
    assert_eq!(column(&book, 1, 1..=3), vec!["3", "1", "2"]);
}

#[test]
fn test_sort_rows_array_formulas_are_an_error() {
    let mut book = csv_book("3,\n1,\n2,\n");
    book.update_cell(&phase2_addr(1, 2), "=SORT(A1:A3)").expect("set");
    book.evaluate();
    let err = book.sort_rows(0, 1, 3, &[key(1, true)]).expect_err("array");
    assert_eq!(err.to_string(), "Cannot sort a range that contains array formulas.");
    assert_eq!(column(&book, 1, 1..=3), vec!["3", "1", "2"]);
}

// -------------------------------------------------------------------------
// Undo and redo
// -------------------------------------------------------------------------

/// Everything a change can touch in the first `rows` × `cols` cells: sheet
/// names, column widths, and each cell's contents and style.
fn snapshot(book: &Book, rows: usize, cols: usize) -> String {
    let mut out = format!("{:?}\n", book.get_sheet_names());
    for sheet in 0..book.get_sheet_names().len() as u32 {
        for col in 1..=cols {
            let width = book.get_column_size_for_sheet(sheet, col).expect("width");
            out.push_str(&format!("{sheet} width {col}: {width}\n"));
        }
        for row in 1..=rows {
            for col in 1..=cols {
                let addr = Address { sheet, row, col };
                let contents = book.get_cell_addr_contents(&addr).expect("contents");
                let style = book.get_cell_style(&addr);
                out.push_str(&format!("{sheet} {row},{col}: {contents:?} {style:?}\n"));
            }
        }
    }
    out
}

type Change = fn(&mut Book) -> anyhow::Result<()>;

#[test]
fn test_undo_and_redo_each_change() {
    let changes: Vec<(&str, Change)> = vec![
        ("update_cell", |b| b.update_cell(&phase2_addr(1, 1), "x")),
        ("formula", |b| b.update_cell(&phase2_addr(1, 3), "=A2*2")),
        ("clear_cell_contents", |b| b.clear_cell_contents(phase2_addr(1, 1))),
        ("clear_cell_all", |b| b.clear_cell_all(phase2_addr(2, 2))),
        ("clear_cell_range", |b| {
            b.clear_cell_range(phase2_addr(1, 1), phase2_addr(2, 2))
        }),
        ("clear_cell_range_all", |b| {
            b.clear_cell_range_all(phase2_addr(1, 1), phase2_addr(2, 2))
        }),
        ("set_cell_style", |b| {
            let area = Area { sheet: 0, row: 1, column: 1, width: 2, height: 1 };
            b.set_cell_style(&[("font.b", "true"), ("fill.color", "#ff0000")], &area)
        }),
        ("extend_to", |b| b.extend_to(&phase2_addr(1, 1), &phase2_addr(3, 1))),
        ("insert_rows", |b| b.insert_rows(1, 2)),
        ("insert_columns", |b| b.insert_columns(1, 2)),
        ("sort_rows", |b| b.sort_rows(0, 1, 3, &[key(1, false)])),
        ("set_col_size", |b| b.set_col_size(1, 30)),
        ("set_sheet_name", |b| b.set_sheet_name(0, "Renamed")),
        ("new_sheet", |b| b.new_sheet(Some("Other"))),
    ];
    for (name, change) in changes {
        let mut book = csv_book("1,2\n3,4\n5,6\n");
        book.set_cell_style(
            &[("font.i", "true")],
            &Area { sheet: 0, row: 2, column: 2, width: 1, height: 1 },
        )
        .expect("style");
        book.evaluate();
        let before = snapshot(&book, 5, 4);
        change(&mut book).expect(name);
        book.evaluate();
        let after = snapshot(&book, 5, 4);
        assert_ne!(before, after, "{name} changes the book");

        assert!(book.undo().expect("undo"), "{name} can be undone");
        assert_eq!(snapshot(&book, 5, 4), before, "undo of {name}");
        assert!(book.redo().expect("redo"), "{name} can be redone");
        assert_eq!(snapshot(&book, 5, 4), after, "redo of {name}");
        assert!(book.undo().expect("undo"));
        assert!(book.undo().expect("undo"), "the style is the action before {name}");
        assert!(!book.undo().expect("undo"), "{name} is one action");
    }
}

#[test]
fn test_undo_of_a_sort_restores_formulas_and_styles() {
    let mut book = csv_book("3,c\n1,a\n2,b\n");
    book.update_cell(&phase2_addr(1, 3), "=A1*2").expect("formula");
    book.set_cell_style(
        &[("font.b", "true")],
        &Area { sheet: 0, row: 1, column: 2, width: 1, height: 1 },
    )
    .expect("style");
    book.evaluate();
    let before = snapshot(&book, 3, 3);
    book.sort_rows(0, 1, 3, &[key(1, true)]).expect("sort");
    assert_eq!(column(&book, 1, 1..=3), vec!["1", "2", "3"]);
    assert!(book.undo().expect("undo"));
    assert_eq!(snapshot(&book, 3, 3), before);
    assert_eq!(column(&book, 3, 1..=3), vec!["6", "", ""]);
}

#[test]
fn test_a_loaded_book_has_nothing_to_undo() {
    let mut book = csv_book("1,2\n3,4\n");
    assert!(!book.undo().expect("undo"));
    assert!(!book.redo().expect("redo"));
    let mut book = Book::default();
    assert!(!book.undo().expect("undo"));
}

#[test]
fn test_a_sort_that_changes_nothing_is_not_an_action() {
    let mut book = csv_book("1\n2\n3\n");
    book.sort_rows(0, 1, 3, &[key(1, true)]).expect("sort");
    assert!(!book.undo().expect("undo"));
}

#[test]
fn test_changes_inside_an_action_undo_together() {
    let mut book = csv_book("1,2\n");
    book.begin_action();
    book.update_cell(&phase2_addr(1, 1), "a").expect("set");
    book.begin_action();
    book.update_cell(&phase2_addr(1, 2), "b").expect("set");
    book.end_action();
    book.update_cell(&phase2_addr(2, 1), "c").expect("set");
    book.end_action();
    book.evaluate();
    assert!(book.undo().expect("undo"));
    assert_eq!(column(&book, 1, 1..=2), vec!["1", ""]);
    assert_eq!(rendered(&book, 1, 2), "2");
    assert!(!book.undo().expect("undo"));
}

#[test]
fn test_an_action_with_no_change_is_not_kept() {
    let mut book = csv_book("1\n");
    book.update_cell(&phase2_addr(1, 1), "2").expect("set");
    book.begin_action();
    book.end_action();
    assert!(book.undo().expect("undo"));
    assert_eq!(rendered(&book, 1, 1), "1");
}

#[test]
fn test_undo_and_redo_go_to_the_change() {
    let mut book = csv_book("1,2\n");
    book.new_sheet(Some("Other")).expect("new sheet");
    assert!(book.select_sheet_by_name("Other"));
    book.move_to(&Address { sheet: 1, row: 4, col: 3 }).expect("move");
    book.edit_current_cell("x").expect("edit");
    assert!(book.select_sheet_by_name("Sheet1"));
    book.move_to(&Address { sheet: 0, row: 1, col: 1 }).expect("move");

    assert!(book.undo().expect("undo"));
    assert_eq!(book.location, Address { sheet: 1, row: 4, col: 3 });
    assert_eq!(book.model.get_selected_view().sheet, 1);
    book.move_to(&Address { sheet: 1, row: 1, col: 1 }).expect("move");
    assert!(book.redo().expect("redo"));
    assert_eq!(book.location, Address { sheet: 1, row: 4, col: 3 });
}

#[test]
fn test_undo_of_new_sheet_removes_it() {
    let mut book = csv_book("1\n");
    book.new_sheet(Some("Other")).expect("new sheet");
    assert_eq!(book.get_sheet_names().len(), 2);
    assert!(book.undo().expect("undo"));
    assert_eq!(book.get_sheet_names().len(), 1);
    assert_eq!(book.location.sheet, 0);
}

#[test]
fn test_a_new_change_clears_redo() {
    let mut book = csv_book("1,2\n");
    book.update_cell(&phase2_addr(1, 1), "a").expect("set");
    assert!(book.undo().expect("undo"));
    book.update_cell(&phase2_addr(1, 2), "b").expect("set");
    assert!(!book.redo().expect("redo"));
}

#[test]
fn test_undo_back_to_the_save_point_is_not_a_change() {
    let path = tmp_path("undo_save_point.sui");
    let mut book = csv_book("1,2\n");
    book.update_cell(&phase2_addr(1, 1), "a").expect("set");
    assert!(book.undo().expect("undo"));
    assert!(!book.dirty, "back at the loaded state");

    book.update_cell(&phase2_addr(1, 1), "a").expect("set");
    book.save_as(&path).expect("save");
    assert!(!book.dirty);
    book.update_cell(&phase2_addr(1, 2), "b").expect("set");
    assert!(book.dirty);
    assert!(book.undo().expect("undo"));
    assert!(!book.dirty, "back at the saved state");
    assert!(book.redo().expect("redo"));
    assert!(book.dirty);

    // A new change below the save point: the save point is gone.
    assert!(book.undo().expect("undo"));
    assert!(book.undo().expect("undo"));
    assert!(book.dirty);
    book.update_cell(&phase2_addr(2, 1), "c").expect("set");
    assert!(book.undo().expect("undo"));
    assert!(book.dirty, "the saved state can no longer be reached");
    std::fs::remove_file(&path).expect("clean up");
}

// -------------------------------------------------------------------------
// Search
// -------------------------------------------------------------------------

fn re(pattern: &str) -> regex::Regex {
    regex::Regex::new(pattern).expect("pattern")
}

/// The address and wrap flag of the next match, or `None`.
fn find(
    book: &Book,
    pattern: &str,
    from: (usize, usize),
    direction: Direction,
    wrapscan: bool,
) -> Option<((usize, usize), bool)> {
    find_in(book, SearchIn::Value, pattern, from, direction, wrapscan)
}

/// Like [`find`], matching the text `within` says.
fn find_in(
    book: &Book,
    within: SearchIn,
    pattern: &str,
    from: (usize, usize),
    direction: Direction,
    wrapscan: bool,
) -> Option<((usize, usize), bool)> {
    let from = phase2_addr(from.0, from.1);
    book.search(0, &re(pattern), within, &from, direction, wrapscan)
        .expect("search")
        .map(|hit| ((hit.address.row, hit.address.col), hit.wrapped))
}

#[test]
fn test_search_forward_is_row_major() {
    // x at A1, C1, B2, A3.
    let book = csv_book("x,y,x\nz,x,z\nx,y,y\n");
    let fwd = Direction::Forward;
    assert_eq!(Some(((1, 3), false)), find(&book, "x", (1, 1), fwd, true));
    assert_eq!(Some(((2, 2), false)), find(&book, "x", (1, 3), fwd, true));
    assert_eq!(Some(((3, 1), false)), find(&book, "x", (2, 2), fwd, true));
}

#[test]
fn test_search_starts_after_an_empty_cursor_cell() {
    let book = csv_book("a,b\nc,hit\n");
    // The cursor is beyond the used range.
    assert_eq!(None, find(&book, "hit", (9, 9), Direction::Forward, false));
    assert_eq!(
        Some(((2, 2), true)),
        find(&book, "hit", (9, 9), Direction::Forward, true)
    );
    assert_eq!(
        Some(((2, 2), false)),
        find(&book, "hit", (1, 5), Direction::Forward, true)
    );
}

#[test]
fn test_search_wraps_to_the_top() {
    let book = csv_book("x,a\nb,c\n");
    assert_eq!(Some(((1, 1), true)), find(&book, "x", (2, 1), Direction::Forward, true));
}

#[test]
fn test_search_only_match_is_the_cursor_cell() {
    let book = csv_book("a,x\nb,c\n");
    assert_eq!(Some(((1, 2), true)), find(&book, "x", (1, 2), Direction::Forward, true));
}

#[test]
fn test_search_without_wrapscan_stops_at_the_end() {
    let book = csv_book("x,a\nb,c\n");
    assert_eq!(None, find(&book, "x", (1, 1), Direction::Forward, false));
}

#[test]
fn test_search_backward_is_reverse_row_major() {
    let book = csv_book("x,y,x\nz,x,z\nx,y,y\n");
    let back = Direction::Backward;
    assert_eq!(Some(((2, 2), false)), find(&book, "x", (3, 1), back, true));
    assert_eq!(Some(((1, 3), false)), find(&book, "x", (2, 2), back, true));
    assert_eq!(Some(((3, 1), true)), find(&book, "x", (1, 1), back, true));
    assert_eq!(None, find(&book, "x", (1, 1), back, false));
}

#[test]
fn test_search_no_match() {
    let book = csv_book("a,b\nc,d\n");
    assert_eq!(None, find(&book, "zzz", (1, 1), Direction::Forward, true));
}

#[test]
fn test_search_empty_sheet() {
    let book = Book::default();
    assert_eq!(None, find(&book, "", (1, 1), Direction::Forward, true));
    assert_eq!(None, find(&book, ".*", (1, 1), Direction::Forward, true));
}

#[test]
fn test_search_skips_cells_with_no_content() {
    let mut book = csv_book("a,b\n");
    // A styled cell with no content is in the sheet data but is not visited.
    book.set_cell_style(
        &[("font.b", "true")],
        &Area { sheet: 0, row: 1, column: 3, width: 1, height: 1 },
    )
    .expect("style");
    assert_eq!(Some(((1, 2), false)), find(&book, "^b?$", (1, 1), Direction::Forward, true));
    assert_eq!(Some(((1, 1), true)), find(&book, "^a?$", (1, 2), Direction::Forward, true));
}

#[test]
fn test_search_matches_the_displayed_value() {
    let mut book = Book::default();
    book.update_cell(&phase2_addr(1, 1), "=1+1").expect("set");
    book.update_cell(&phase2_addr(1, 2), "$12.00").expect("set");
    book.evaluate();
    assert_eq!(Some(((1, 1), true)), find(&book, "^2$", (1, 1), Direction::Forward, true));
    assert_eq!(None, find(&book, "1\\+1", (1, 1), Direction::Forward, true));
    assert_eq!(
        Some(((1, 2), false)),
        find(&book, "^\\$12\\.00$", (1, 1), Direction::Forward, true)
    );
}

/// Formula `=1+1` at A1, `$12.00` at B1, `50%` at C1, a date at D1 and text
/// at E1.
fn formats_book() -> Book {
    let mut book = Book::default();
    let texts = ["=1+1", "$12.00", "50%", "2026-11-04", "SUM it"];
    for (col, text) in texts.iter().enumerate() {
        book.update_cell(&phase2_addr(1, col + 1), text).expect("set");
    }
    book.evaluate();
    book
}

/// The column of the first cell after A1 whose `within` text matches.
fn first_col(book: &Book, within: SearchIn, pattern: &str) -> Option<usize> {
    find_in(book, within, pattern, (1, 1), Direction::Forward, true).map(|((_, col), _)| col)
}

#[test]
fn test_search_value_matches_what_is_shown() {
    let book = formats_book();
    let value = SearchIn::Value;
    assert_eq!(Some(1), first_col(&book, value, "^2$"));
    assert_eq!(None, first_col(&book, value, "1\\+1"));
    assert_eq!(Some(2), first_col(&book, value, "^\\$12\\.00$"));
    assert_eq!(Some(3), first_col(&book, value, "^50%$"));
    assert_eq!(None, first_col(&book, value, "^0\\.5$"));
    assert_eq!(Some(4), first_col(&book, value, "^2026-11-04$"));
    assert_eq!(None, first_col(&book, value, "SUM\\("));
}

#[test]
fn test_search_raw_matches_what_cell_edit_shows() {
    let book = formats_book();
    let raw = SearchIn::Raw;
    // Each cell matches its stored text, as `e` would show it.
    for col in 1..=5 {
        let stored = book.get_cell_addr_contents(&phase2_addr(1, col)).expect("contents");
        let pattern = format!("^{}$", regex::escape(&stored));
        let hit = find_in(&book, raw, &pattern, (2, 1), Direction::Forward, true);
        assert_eq!(Some(((1, col), true)), hit, "{stored}");
    }
    assert_eq!(Some(1), first_col(&book, raw, "^=1\\+1$"));
    assert_eq!(None, first_col(&book, raw, "^2$"));
    assert_eq!(Some(2), first_col(&book, raw, "^12$"));
    assert_eq!(None, first_col(&book, raw, "\\$"));
    assert_eq!(Some(3), first_col(&book, raw, "^0\\.5$"));
    assert_eq!(None, first_col(&book, raw, "%"));
    // A date is stored as a number, but CellEdit shows it as a date, so raw
    // search matches the date too.
    assert_eq!(Some(4), first_col(&book, raw, "^2026-11-04$"));
}

#[test]
fn test_search_raw_finds_formulas() {
    let mut book = csv_book("1,2\n");
    book.update_cell(&phase2_addr(2, 1), "=SUM(A1:B1)").expect("set");
    book.evaluate();
    let raw = SearchIn::Raw;
    let fwd = Direction::Forward;
    assert_eq!(Some(((2, 1), false)), find_in(&book, raw, "SUM\\(", (1, 1), fwd, true));
    assert_eq!(None, find(&book, "SUM\\(", (1, 1), Direction::Forward, true));
}

#[test]
fn test_search_merged_cell_is_its_top_left_cell() {
    let mut model =
        Model::new_empty("test", "en", "America/New_York", "en").expect("model");
    model.workbook.worksheets[0].merge_cells.push("A1:B2".to_string());
    let mut book = Book::from_model(model);
    for (row, col) in [(1, 1), (1, 2), (2, 1), (2, 2), (3, 1)] {
        book.update_cell(&phase2_addr(row, col), "m").expect("set");
    }
    book.evaluate();
    let fwd = Direction::Forward;
    assert_eq!(Some(((3, 1), false)), find(&book, "m", (1, 1), fwd, true));
    assert_eq!(Some(((1, 1), true)), find(&book, "m", (3, 1), fwd, true));
}

#[test]
fn test_search_array_formula_is_one_cell() {
    let mut book = csv_book("3\n1\n2\n");
    book.update_cell(&phase2_addr(1, 2), "=SORT(A1:A3)").expect("set");
    book.evaluate();
    // The spilled values in B2 and B3 belong to the formula in B1.
    assert_eq!(Some(((1, 2), false)), find(&book, "^1$", (1, 1), Direction::Forward, true));
    assert_eq!(Some(((2, 1), false)), find(&book, "^1$", (1, 2), Direction::Forward, true));
}

#[test]
fn test_search_other_sheets_are_not_searched() {
    let mut book = csv_book("a\n");
    book.new_sheet(Some("two")).expect("sheet");
    book.update_cell(&Address { sheet: 1, row: 1, col: 1 }, "hit").expect("set");
    book.evaluate();
    assert_eq!(None, find(&book, "hit", (1, 1), Direction::Forward, true));
}

#[test]
fn test_search_backward_from_beyond_the_used_range() {
    let book = csv_book("a,hit\nc,d\n");
    // Every cell is before the cursor, so nothing wraps.
    assert_eq!(
        Some(((1, 2), false)),
        find(&book, "hit", (9, 9), Direction::Backward, false)
    );
    // From before the first cell, backward wraps to the last.
    assert_eq!(Some(((2, 2), true)), find(&book, "d", (1, 1), Direction::Backward, true));
}

/// The cells of rows `rows` and columns `cols` that match `pattern`, sorted.
fn matches_in(book: &Book, within: SearchIn, pattern: &str, rows: (usize, usize), cols: (usize, usize)) -> Vec<(usize, usize)> {
    let mut found: Vec<_> = book
        .matches_in(0, &re(pattern), within, rows.0..=rows.1, cols.0..=cols.1)
        .expect("matches")
        .into_iter()
        .collect();
    found.sort_unstable();
    found
}

#[test]
fn test_matches_in_a_rectangle() {
    let book = csv_book("x,y,x\nz,x,z\nx,y,y\n");
    let value = SearchIn::Value;
    assert_eq!(
        vec![(1, 1), (1, 3), (2, 2), (3, 1)],
        matches_in(&book, value, "x", (1, 3), (1, 3))
    );
    // Only cells inside the rectangle count.
    assert_eq!(vec![(1, 3), (2, 2)], matches_in(&book, value, "x", (1, 2), (2, 3)));
    // Empty cells and cells beyond the used range never match.
    assert_eq!(Vec::<(usize, usize)>::new(), matches_in(&book, value, "^$", (1, 9), (1, 9)));
    assert_eq!(Vec::<(usize, usize)>::new(), matches_in(&book, value, "q", (1, 3), (1, 3)));
}

#[test]
fn test_matches_in_follows_searchin() {
    let mut book = csv_book("1,2\n");
    book.update_cell(&phase2_addr(1, 3), "=A1+B1").expect("set");
    book.evaluate();
    assert_eq!(vec![(1, 3)], matches_in(&book, SearchIn::Value, "^3$", (1, 1), (1, 3)));
    assert_eq!(vec![(1, 3)], matches_in(&book, SearchIn::Raw, "A1", (1, 1), (1, 3)));
    assert_eq!(Vec::<(usize, usize)>::new(), matches_in(&book, SearchIn::Value, "A1", (1, 1), (1, 3)));
}

#[test]
fn test_matches_in_skips_hidden_merged_and_spilled_cells() {
    let mut model =
        Model::new_empty("test", "en", "America/New_York", "en").expect("model");
    model.workbook.worksheets[0].merge_cells.push("A1:B2".to_string());
    let mut book = Book::from_model(model);
    for (row, col) in [(1, 1), (1, 2), (2, 1), (2, 2), (3, 1)] {
        book.update_cell(&phase2_addr(row, col), "1").expect("set");
    }
    book.update_cell(&phase2_addr(1, 3), "=SORT(A1:A3)").expect("set");
    book.evaluate();
    // C1 shows 1 too, but C2 and C3 are spilled from it.
    assert_eq!(
        vec![(1, 1), (3, 1)],
        matches_in(&book, SearchIn::Value, "^1$", (1, 3), (1, 2))
    );
    assert_eq!(
        vec![(1, 3)],
        matches_in(&book, SearchIn::Raw, "SORT", (1, 3), (3, 3))
    );
}

// -------------------------------------------------------------------------
// Substitute
// -------------------------------------------------------------------------

/// Runs `:s` with `pattern` and `replacement` (vim form) on one cell.
fn sub_cell(
    book: &mut Book,
    cell: (usize, usize),
    pattern: &str,
    replacement: &str,
    global: bool,
) -> anyhow::Result<super::substitute::Counts> {
    let replacement = super::substitute::replacement_form(replacement).expect("replacement");
    book.substitute(&[phase2_addr(cell.0, cell.1)], &re(pattern), &replacement, global)
}

fn contents(book: &Book, row: usize, col: usize) -> String {
    book.get_cell_addr_contents(&phase2_addr(row, col)).expect("contents")
}

#[test]
fn test_substitute_edits_the_formula_and_it_stays_a_formula() {
    let mut book = csv_book("2,3\n");
    book.update_cell(&phase2_addr(2, 1), "=SUM(A1:B1)").expect("set");
    book.evaluate();
    let counts = sub_cell(&mut book, (2, 1), "SUM", "PRODUCT", false).expect("sub");
    assert_eq!((counts.substitutions, counts.cells), (1, 1));
    assert_eq!(contents(&book, 2, 1), "=PRODUCT(A1:B1)");
    assert_eq!(rendered(&book, 2, 1), "6");
}

#[test]
fn test_substitute_number_stays_a_number() {
    let mut book = csv_book("12\n");
    book.update_cell(&phase2_addr(2, 1), "=A1*2").expect("set");
    book.evaluate();
    sub_cell(&mut book, (1, 1), "1", "3", false).expect("sub");
    assert_eq!(rendered(&book, 2, 1), "64");
}

#[test]
fn test_substitute_matches_the_stored_number_not_the_shown_one() {
    let mut book = csv_book("12\n");
    let area = Area { sheet: 0, row: 1, column: 1, width: 1, height: 1 };
    book.set_cell_style(&[("num_fmt", "$#,##0.00")], &area).expect("format");
    assert_eq!(rendered(&book, 1, 1), "$12.00");
    let counts = sub_cell(&mut book, (1, 1), "12\\.00", "x", false).expect("sub");
    assert_eq!(counts.substitutions, 0);
    sub_cell(&mut book, (1, 1), "12", "15", false).expect("sub");
    assert_eq!(rendered(&book, 1, 1), "$15.00");
}

#[test]
fn test_substitute_keeps_the_cell_style() {
    let mut book = csv_book("apple\n");
    let area = Area { sheet: 0, row: 1, column: 1, width: 1, height: 1 };
    book.set_cell_style(&[("font.b", "true"), ("fill.color", "#ff0000")], &area)
        .expect("style");
    let before = book.get_cell_style(&phase2_addr(1, 1));
    sub_cell(&mut book, (1, 1), "apple", "pear", false).expect("sub");
    assert_eq!(rendered(&book, 1, 1), "pear");
    assert_eq!(book.get_cell_style(&phase2_addr(1, 1)), before);
}

#[test]
fn test_substitute_invalid_formula_behaves_as_typed() {
    let mut book = csv_book("1\n");
    book.update_cell(&phase2_addr(2, 1), "=A1+1").expect("set");
    book.evaluate();
    let mut typed = csv_book("1\n");
    typed.update_cell(&phase2_addr(2, 1), "=A1+").expect("set");
    typed.evaluate();
    sub_cell(&mut book, (2, 1), "1$", "", false).expect("sub");
    assert_eq!(contents(&book, 2, 1), contents(&typed, 2, 1));
    assert_eq!(rendered(&book, 2, 1), rendered(&typed, 2, 1));
    assert!(book.undo().expect("undo"));
    assert_eq!(contents(&book, 2, 1), "=A1+1");
}

#[test]
fn test_substitute_is_one_undo_step() {
    let mut book = csv_book("banana\n");
    sub_cell(&mut book, (1, 1), "a", "o", true).expect("sub");
    assert_eq!(rendered(&book, 1, 1), "bonono");
    assert!(book.undo().expect("undo"));
    assert_eq!(rendered(&book, 1, 1), "banana");
    assert!(!book.undo().expect("undo"));
    assert!(book.redo().expect("redo"));
    assert_eq!(rendered(&book, 1, 1), "bonono");
}

#[test]
fn test_substitute_with_no_match_is_not_an_undo_step() {
    let mut book = csv_book("banana\n");
    let counts = sub_cell(&mut book, (1, 1), "z", "o", true).expect("sub");
    assert_eq!(counts.substitutions, 0);
    assert!(!book.undo().expect("undo"));
    assert!(!book.dirty);
}

#[test]
fn test_substitute_with_the_same_text_counts_but_changes_nothing() {
    let mut book = csv_book("banana\n");
    let counts = sub_cell(&mut book, (1, 1), "a", "a", true).expect("sub");
    assert_eq!((counts.substitutions, counts.cells), (3, 1));
    assert!(!book.undo().expect("undo"));
    assert!(!book.dirty);
}

#[test]
fn test_substitute_refuses_an_array_formula() {
    let mut book = csv_book("3,\n1,\n2,\n");
    book.update_cell(&phase2_addr(1, 2), "=SORT(A1:A3)").expect("set");
    book.evaluate();
    let err = sub_cell(&mut book, (1, 2), "SORT", "SUM", false).expect_err("array");
    assert_eq!(err.to_string(), "Cannot substitute in an array formula.");
    assert_eq!(contents(&book, 1, 2), "=SORT(A1:A3)");
    let err = sub_cell(&mut book, (2, 2), ".", "9", false).expect_err("spill");
    assert_eq!(err.to_string(), "Cannot substitute in an array formula.");
    assert_eq!(column(&book, 2, 1..=3), vec!["1", "2", "3"]);
}

#[test]
fn test_substitute_groups_and_ampersand_in_a_cell() {
    let mut book = csv_book("Smith John\n");
    sub_cell(&mut book, (1, 1), "(\\w+) (\\w+)", "\\2 \\1 (&)", false).expect("sub");
    assert_eq!(rendered(&book, 1, 1), "John Smith (Smith John)");
}

// -------------------------------------------------------------------------
// Substitute scopes: `:%s` and `:'<,'>s`
// -------------------------------------------------------------------------

/// The cells that `content_cells` gives, as (row, col).
fn content_cells(book: &Book, within: Option<((usize, usize), (usize, usize))>) -> Vec<(usize, usize)> {
    let within = within.map(|(start, end)| (phase2_addr(start.0, start.1), phase2_addr(end.0, end.1)));
    book.content_cells(0, within.as_ref().map(|(s, e)| (s, e)))
        .expect("cells")
        .into_iter()
        .map(|a| (a.row, a.col))
        .collect()
}

#[test]
fn test_content_cells_of_the_sheet_in_row_major_order() {
    let book = csv_book("a,,c\n,e,\ng,h,i\n");
    assert_eq!(
        vec![(1, 1), (1, 3), (2, 2), (3, 1), (3, 2), (3, 3)],
        content_cells(&book, None)
    );
}

#[test]
fn test_content_cells_inside_a_range() {
    let book = csv_book("a,,c\n,e,\ng,h,i\n");
    assert_eq!(vec![(2, 2), (3, 2), (3, 3)], content_cells(&book, Some(((2, 2), (3, 3)))));
    // The corners can come in any order.
    assert_eq!(vec![(2, 2), (3, 2), (3, 3)], content_cells(&book, Some(((3, 3), (2, 2)))));
    // A range past the used range is fine.
    assert_eq!(vec![(3, 3)], content_cells(&book, Some(((3, 3), (99, 99)))));
}

#[test]
fn test_content_cells_skip_hidden_merged_and_spilled_cells() {
    let mut model =
        Model::new_empty("test", "en", "America/New_York", "en").expect("model");
    model.workbook.worksheets[0].merge_cells.push("A1:B2".to_string());
    let mut book = Book::from_model(model);
    for (row, col) in [(1, 1), (1, 2), (2, 1), (2, 2), (3, 1)] {
        book.update_cell(&phase2_addr(row, col), "1").expect("set");
    }
    book.update_cell(&phase2_addr(1, 3), "=SORT(A1:A3)").expect("set");
    book.evaluate();
    assert_eq!(vec![(1, 1), (1, 3), (3, 1)], content_cells(&book, None));
    // A range over the hidden part of the merge and the spill only.
    assert_eq!(Vec::<(usize, usize)>::new(), content_cells(&book, Some(((2, 2), (3, 3)))));
}

#[test]
fn test_substitute_on_the_whole_sheet_counts_and_is_one_step() {
    let mut book = csv_book("banana,apple\ncherry,anna\n");
    let cells = book.content_cells(0, None).expect("cells");
    let counts = book.substitute(&cells, &re("a"), "o", true).expect("sub");
    assert_eq!((counts.substitutions, counts.cells), (6, 3));
    assert_eq!(rendered(&book, 1, 1), "bonono");
    assert_eq!(rendered(&book, 1, 2), "opple");
    assert_eq!(rendered(&book, 2, 1), "cherry");
    assert_eq!(rendered(&book, 2, 2), "onno");
    assert!(book.undo().expect("undo"));
    assert_eq!(rendered(&book, 1, 1), "banana");
    assert_eq!(rendered(&book, 2, 2), "anna");
    assert!(!book.undo().expect("undo"));
}

#[test]
fn test_substitute_in_a_range_leaves_cells_outside_it() {
    let mut book = csv_book("a,a,a\na,a,a\n");
    let (start, end) = (phase2_addr(1, 2), phase2_addr(2, 3));
    let cells = book.content_cells(0, Some((&start, &end))).expect("cells");
    let counts = book.substitute(&cells, &re("a"), "b", false).expect("sub");
    assert_eq!((counts.substitutions, counts.cells), (4, 4));
    assert_eq!(rendered(&book, 1, 1), "a");
    assert_eq!(rendered(&book, 2, 1), "a");
    assert_eq!(rendered(&book, 1, 2), "b");
    assert_eq!(rendered(&book, 2, 3), "b");
}

#[test]
fn test_substitute_array_formula_in_a_range_refuses_before_any_write() {
    let mut book = csv_book("3,\n1,\n2,\n");
    book.update_cell(&phase2_addr(1, 2), "=SORT(A1:A3)").expect("set");
    book.evaluate();
    let (start, end) = (phase2_addr(1, 1), phase2_addr(3, 2));
    let cells = book.content_cells(0, Some((&start, &end))).expect("cells");
    // A1 matches first, but nothing is written because B1 is an array.
    let err = book.substitute(&cells, &re("[0-9A-Z]"), "x", true).expect_err("array");
    assert!(err.to_string().contains("array formula"), "{}", err);
    assert_eq!(rendered(&book, 1, 1), "3");
    assert_eq!(contents(&book, 1, 2), "=SORT(A1:A3)");
    // A pattern that skips the array formula edits the other cells; the
    // spilled values are not cells of their own.
    let counts = book.substitute(&cells, &re("^[0-9]$"), "9", false).expect("sub");
    assert_eq!((counts.substitutions, counts.cells), (3, 3));
    assert_eq!(contents(&book, 1, 2), "=SORT(A1:A3)");
}

#[test]
fn test_substitute_undo_puts_the_cursor_on_the_first_changed_cell() {
    let mut book = csv_book("x,y\nz,y\n");
    book.location = phase2_addr(1, 1);
    let cells = book.content_cells(0, None).expect("cells");
    book.substitute(&cells, &re("y"), "w", false).expect("sub");
    // The cursor stays where it was.
    assert_eq!(book.location, phase2_addr(1, 1));
    book.location = phase2_addr(5, 5);
    assert!(book.undo().expect("undo"));
    assert_eq!(book.location, phase2_addr(1, 2));
    assert_eq!(rendered(&book, 2, 2), "y");
}

#[test]
fn test_substitute_on_a_large_sheet_is_one_undo_step() {
    let row = vec!["a"; 20].join(",");
    let text = vec![row; 250].join("\n");
    let mut book = csv_book(&text);
    let cells = book.content_cells(0, None).expect("cells");
    assert_eq!(cells.len(), 5000);
    let counts = book.substitute(&cells, &re("a"), "b", false).expect("sub");
    assert_eq!((counts.substitutions, counts.cells), (5000, 5000));
    assert_eq!(rendered(&book, 250, 20), "b");
    assert!(book.undo().expect("undo"));
    assert_eq!(rendered(&book, 1, 1), "a");
    assert_eq!(rendered(&book, 250, 20), "a");
    assert!(!book.undo().expect("undo"));
}

/// Stage 0 for the freeze design (F3): the freeze is applied to a copy of the
/// model at save time, which needs `Model::to_bytes`/`from_bytes` to lose
/// nothing a save would write.
#[test]
fn test_model_byte_round_trip_loses_nothing() {
    let mut book = Book::default();
    book.new_sheet(Some("Second")).expect("new sheet");
    let first = Address { sheet: 0, row: 1, col: 1 };
    book.update_cell(&first, "Header").expect("a1");
    book.update_cell(&Address { sheet: 0, row: 2, col: 1 }, "41").expect("a2");
    book.update_cell(&Address { sheet: 0, row: 2, col: 2 }, "=A2+1").expect("b2");
    book.update_cell(&Address { sheet: 0, row: 3, col: 2 }, "0.5").expect("b3");
    book.update_cell(&Address { sheet: 1, row: 1, col: 1 }, "=Sheet1!B2*2").expect("s2 a1");
    book.update_cell(&Address { sheet: 1, row: 4, col: 3 }, "text").expect("s2 c4");
    book.set_cell_style(
        &[("font.b", "true"), ("font.color", "#00ff00"), ("fill.color", "#ff0000")],
        &Area { sheet: 0, row: 1, column: 1, width: 2, height: 1 },
    )
    .expect("style header");
    book.set_cell_style(
        &[("num_fmt", "0.00%"), ("font.i", "true")],
        &Area { sheet: 0, row: 3, column: 2, width: 1, height: 1 },
    )
    .expect("style pct");
    book.set_cell_style(
        &[("fill.color", "#0000ff")],
        &Area { sheet: 1, row: 4, column: 3, width: 1, height: 1 },
    )
    .expect("style sheet 2");
    book.set_column_size_for_sheet(0, 1, 30).expect("width a");
    book.set_column_size_for_sheet(0, 2, 7).expect("width b");
    book.set_column_size_for_sheet(1, 3, 22).expect("width s2 c");
    book.evaluate();

    let original = book.model.get_model();
    let copy = Model::from_bytes(&original.to_bytes(), "en").expect("from_bytes");

    let sheets = original.workbook.worksheets.len();
    assert_eq!(2, sheets);
    assert_eq!(sheets, copy.workbook.worksheets.len());
    let mut checked_styles = 0;
    for sheet in 0..sheets as u32 {
        assert_eq!(
            original.workbook.worksheets[sheet as usize].name,
            copy.workbook.worksheets[sheet as usize].name
        );
        for col in 1..=6 {
            assert_eq!(
                original.get_column_width(sheet, col).expect("orig width"),
                copy.get_column_width(sheet, col).expect("copy width"),
                "width sheet {sheet} col {col}"
            );
        }
        for row in 1..=6 {
            for col in 1..=6 {
                let at = format!("sheet {sheet} r{row} c{col}");
                assert_eq!(
                    original.get_cell_value_by_index(sheet, row, col),
                    copy.get_cell_value_by_index(sheet, row, col),
                    "value {at}"
                );
                assert_eq!(
                    original.get_cell_formula(sheet, row, col),
                    copy.get_cell_formula(sheet, row, col),
                    "formula {at}"
                );
                assert_eq!(
                    original.get_formatted_cell_value(sheet, row, col),
                    copy.get_formatted_cell_value(sheet, row, col),
                    "formatted {at}"
                );
                let style = original.get_style_for_cell(sheet, row, col).expect("orig style");
                assert_eq!(
                    style,
                    copy.get_style_for_cell(sheet, row, col).expect("copy style"),
                    "style {at}"
                );
                if style != original.get_style_for_cell(sheet, 6, 6).expect("default") {
                    checked_styles += 1;
                }
            }
        }
    }
    // The fixture really did carry styles, so the equality above means something.
    assert!(checked_styles >= 4, "only {checked_styles} styled cells");
    assert_eq!(
        Ok(84.0),
        copy.get_cell_value_by_index(1, 1, 1).map(|v| match v {
            ironcalc::base::cell::CellValue::Number(n) => n,
            _ => f64::NAN,
        })
    );
}

// ---------------------------------------------------------------------------
// Freeze
// ---------------------------------------------------------------------------

#[test]
fn test_freeze_xlsx_round_trip_and_live_model_untouched() {
    let path = std::env::temp_dir().join("sheetui_freeze_round_trip.xlsx");
    let mut book = Book::new(UserModel::new_empty("one", "en", "America/New_York", "en").unwrap());
    book.new_sheet(Some("two")).unwrap();
    book.update_cell(&Address { sheet: 0, row: 2, col: 2 }, "x").unwrap();
    book.set_frozen(0, 2, 1).unwrap();
    book.set_frozen(1, 3, 0).unwrap();
    assert!(book.dirty);
    book.save_to_xlsx(path.to_str().unwrap()).unwrap();
    assert!(!book.dirty);
    // The live model was never told.
    assert_eq!(0, book.model.get_model().get_frozen_rows_count(0).unwrap());
    let loaded = Book::load(&path, "en", "America/New_York").unwrap();
    assert_eq!((2, 1), (loaded.frozen_rows(0), loaded.frozen_columns(0)));
    assert_eq!((3, 0), (loaded.frozen_rows(1), loaded.frozen_columns(1)));
    assert_eq!("x", loaded.get_cell_addr_contents(&Address { sheet: 0, row: 2, col: 2 }).unwrap());
    // Unfreezing is saved too.
    let mut loaded = loaded;
    loaded.set_frozen(0, 0, 0).unwrap();
    loaded.save().unwrap();
    let again = Book::load(&path, "en", "America/New_York").unwrap();
    assert_eq!(0, again.frozen_rows(0));
    assert_eq!(3, again.frozen_rows(1));
    std::fs::remove_file(&path).ok();
}

#[test]
fn test_set_frozen_rejects_out_of_range() {
    let mut book = Book::new(UserModel::new_empty("one", "en", "America/New_York", "en").unwrap());
    assert!(book.set_frozen(0, LAST_ROW as usize + 1, 0).is_err());
    assert!(book.set_frozen(0, 0, LAST_COLUMN as usize + 1).is_err());
    assert!(book.set_frozen(5, 1, 0).is_err());
    assert!(!book.dirty);
    assert_eq!(0, book.frozen_rows(0));
}

#[test]
fn test_freeze_in_sui_and_csv_is_not_saved() {
    let path = std::env::temp_dir().join("sheetui_freeze_not_saved.sui");
    let mut book = Book::new(UserModel::new_empty("one", "en", "America/New_York", "en").unwrap());
    book.set_frozen(0, 2, 0).unwrap();
    book.save_as(&path).unwrap();
    assert!(!book.dirty);
    let loaded = Book::load(&path, "en", "America/New_York").unwrap();
    assert_eq!(0, loaded.frozen_rows(0));
    std::fs::remove_file(&path).ok();
}

fn frozen_book(rows: usize, cols: usize) -> Book {
    let mut book = Book::new(UserModel::new_empty("one", "en", "America/New_York", "en").unwrap());
    book.set_frozen(0, rows, cols).unwrap();
    book.dirty = false;
    book
}

#[test]
fn test_insert_rows_inside_freeze_grows_it() {
    let mut book = frozen_book(3, 0);
    book.insert_rows(2, 2).unwrap();
    assert_eq!(5, book.frozen_rows(0));
    assert!(book.dirty);
}

#[test]
fn test_insert_rows_at_freeze_boundary_and_below() {
    // Row n is the last frozen row, so it is still inside.
    let mut book = frozen_book(3, 0);
    book.insert_rows(3, 1).unwrap();
    assert_eq!(4, book.frozen_rows(0));
    // Row n + 1 is the first scrolling row, so it lands below the freeze.
    let mut book = frozen_book(3, 0);
    book.insert_rows(4, 1).unwrap();
    assert_eq!(3, book.frozen_rows(0));
    let mut book = frozen_book(3, 0);
    book.insert_rows(10, 2).unwrap();
    assert_eq!(3, book.frozen_rows(0));
}

#[test]
fn test_insert_columns_follow_the_same_rule() {
    let mut book = frozen_book(0, 3);
    book.insert_columns(2, 2).unwrap();
    assert_eq!(5, book.frozen_columns(0));
    let mut book = frozen_book(0, 3);
    book.insert_columns(4, 2).unwrap();
    assert_eq!(3, book.frozen_columns(0));
}

#[test]
fn test_insert_with_no_freeze_stays_unfrozen() {
    let mut book = frozen_book(0, 0);
    book.insert_rows(1, 2).unwrap();
    book.insert_columns(1, 2).unwrap();
    assert_eq!((0, 0), (book.frozen_rows(0), book.frozen_columns(0)));
}

#[test]
fn test_undo_insert_does_not_change_freeze() {
    let mut book = frozen_book(3, 2);
    book.insert_rows(2, 2).unwrap();
    book.insert_columns(1, 1).unwrap();
    assert_eq!((5, 3), (book.frozen_rows(0), book.frozen_columns(0)));
    book.undo().unwrap();
    book.undo().unwrap();
    assert_eq!((5, 3), (book.frozen_rows(0), book.frozen_columns(0)));
}

#[test]
fn test_insert_rows_clamps_freeze_to_last_row() {
    let mut book = frozen_book(LAST_ROW as usize - 1, 0);
    book.insert_rows(1, 5).unwrap();
    assert_eq!(LAST_ROW as usize, book.frozen_rows(0));
}

#[test]
fn test_shrink_freeze_table() {
    // (first deleted, count, frozen, expected)
    let table = [
        (2, 4, 3, 1),  // spanning the boundary
        (6, 1, 3, 3),  // wholly below
        (1, 3, 3, 0),  // every frozen row
        (1, 10, 3, 0), // past the freeze
        (3, 1, 3, 2),  // last frozen row
        (4, 1, 3, 3),  // first scrolling row
        (2, 1, 3, 2),  // inside
        (1, 1, 0, 0),  // no freeze
    ];
    for (first, count, frozen, want) in table {
        assert_eq!(want, shrink_freeze(first, count, frozen), "{first} {count} {frozen}");
    }
}

#[test]
fn test_delete_rows_shrinks_freeze() {
    let mut book = frozen_book(3, 0);
    book.delete_rows(2, 4).unwrap();
    assert_eq!(1, book.frozen_rows(0));
    assert!(book.dirty);
    let mut book = frozen_book(3, 0);
    book.delete_rows(6, 1).unwrap();
    assert_eq!(3, book.frozen_rows(0));
    let mut book = frozen_book(3, 0);
    book.delete_rows(1, 3).unwrap();
    assert_eq!(0, book.frozen_rows(0));
}

#[test]
fn test_delete_columns_shrinks_freeze() {
    let mut book = frozen_book(0, 3);
    book.delete_columns(2, 4).unwrap();
    assert_eq!(1, book.frozen_columns(0));
    let mut book = frozen_book(0, 3);
    book.delete_columns(6, 1).unwrap();
    assert_eq!(3, book.frozen_columns(0));
    let mut book = frozen_book(0, 3);
    book.delete_columns(1, 3).unwrap();
    assert_eq!(0, book.frozen_columns(0));
}

#[test]
fn test_undo_delete_does_not_restore_freeze() {
    let mut book = frozen_book(3, 3);
    book.delete_rows(1, 2).unwrap();
    book.delete_columns(1, 2).unwrap();
    book.undo().unwrap();
    book.undo().unwrap();
    assert_eq!((1, 1), (book.frozen_rows(0), book.frozen_columns(0)));
}

// -------------------------------------------------------------------------
// Delete rows

fn rows_book(n: usize) -> Book {
    let mut book = Book::default();
    for r in 1..=n {
        book.update_cell(&phase2_addr(r, 1), r.to_string()).expect("set");
    }
    book.evaluate();
    book.clear_history();
    book.dirty = false;
    book
}

fn at(book: &mut Book, row: usize, col: usize) {
    book.move_to(&phase2_addr(row, col)).expect("move");
}

#[test]
fn test_delete_rows_shifts_rows_up() {
    let mut book = rows_book(6);
    book.delete_rows(2, 1).expect("delete");
    assert_eq!(column(&book, 1, 1..=5), vec!["1", "3", "4", "5", "6"]);
    assert!(book.dirty);
    let mut book = rows_book(6);
    book.delete_rows(2, 3).expect("delete");
    assert_eq!(column(&book, 1, 1..=3), vec!["1", "5", "6"]);
}

#[test]
fn test_delete_rows_cursor_rules() {
    // In the deleted rows: stays at the first one.
    let mut book = rows_book(6);
    at(&mut book, 3, 1);
    book.delete_rows(2, 3).expect("delete");
    assert_eq!(book.location, phase2_addr(2, 1));
    // Above the deleted rows: untouched.
    let mut book = rows_book(6);
    at(&mut book, 1, 1);
    book.delete_rows(2, 3).expect("delete");
    assert_eq!(book.location, phase2_addr(1, 1));
    // Below: moves up so it points at the same data.
    let mut book = rows_book(10);
    at(&mut book, 10, 1);
    book.delete_rows(3, 4).expect("delete");
    assert_eq!(book.location, phase2_addr(6, 1));
    assert_eq!("10", book.get_current_cell_rendered().unwrap());
    // Last row with data: same index, now empty.
    let mut book = rows_book(4);
    at(&mut book, 4, 1);
    book.delete_rows(4, 1).expect("delete");
    assert_eq!(book.location, phase2_addr(4, 1));
    assert_eq!("", book.get_current_cell_rendered().unwrap());
}

#[test]
fn test_delete_rows_is_one_undo_step() {
    let mut book = rows_book(6);
    at(&mut book, 3, 1);
    book.delete_rows(2, 3).expect("delete");
    assert!(book.undo().expect("undo"));
    assert_eq!(column(&book, 1, 1..=6), vec!["1", "2", "3", "4", "5", "6"]);
    assert_eq!(book.location, phase2_addr(3, 1));
    assert!(!book.dirty, "back at the save point");
    assert!(!book.undo().expect("nothing before"), "one step only");
}

#[test]
fn test_delete_rows_errors_change_nothing() {
    let mut book = rows_book(3);
    at(&mut book, 2, 1);
    assert!(book.delete_rows(2, 0).is_err());
    assert!(book.delete_rows(1_048_576, 2).is_err());
    assert!(book.delete_rows(1_048_577, 1).is_err());
    assert_eq!(column(&book, 1, 1..=3), vec!["1", "2", "3"]);
    assert_eq!(book.location, phase2_addr(2, 1));
    assert!(!book.dirty);
    assert!(!book.undo().expect("undo"), "no undo step");
}

#[test]
fn test_delete_rows_partly_overlapping_an_array_formula_is_refused() {
    let mut book = csv_book("3,\n1,\n2,\n4,\n");
    book.model.set_user_array_formula(0, 1, 2, 1, 3, "=A1:A3*2").expect("set");
    book.evaluate();
    book.clear_history();
    book.dirty = false;
    at(&mut book, 3, 1);
    let before = column(&book, 2, 1..=3);
    let err = book.delete_rows(2, 1).expect_err("array");
    assert!(err.to_string().contains("array formula"), "{err}");
    assert_eq!(column(&book, 2, 1..=3), before);
    assert_eq!(book.location, phase2_addr(3, 1));
    assert!(!book.dirty);
    assert!(!book.undo().expect("undo"));
}

#[test]
fn test_delete_rows_wholly_holding_an_array_formula_deletes_it() {
    let mut book = csv_book("3,\n1,\n2,\n4,\n");
    book.model.set_user_array_formula(0, 1, 2, 1, 3, "=A1:A3*2").expect("set");
    book.evaluate();
    book.delete_rows(1, 3).expect("delete");
    assert_eq!(column(&book, 1, 1..=1), vec!["4"]);
    assert_eq!(column(&book, 2, 1..=3), vec!["", "", ""]);
}

#[test]
fn test_delete_rows_rewrites_references() {
    let mut book = rows_book(9);
    book.update_cell(&phase2_addr(1, 2), "=A5").expect("set");
    book.update_cell(&phase2_addr(2, 2), "=A9").expect("set");
    book.evaluate();
    book.delete_rows(5, 1).expect("delete");
    book.evaluate();
    assert_eq!("#REF!", rendered(&book, 1, 2));
    assert_eq!("=A8", book.get_cell_addr_contents(&phase2_addr(2, 2)).unwrap());
    assert_eq!("9", rendered(&book, 2, 2));
}

/// Characterization (not a spec): what ironcalc does to a range when its
/// first or last row is deleted.
#[test]
fn test_delete_rows_characterizes_range_references() {
    let mut book = rows_book(10);
    book.update_cell(&phase2_addr(12, 2), "=SUM(A1:A10)").expect("set");
    book.delete_rows(1, 1).expect("delete first");
    book.evaluate();
    let first = book.get_cell_addr_contents(&phase2_addr(11, 2)).unwrap();
    let first_value = rendered(&book, 11, 2);
    let mut book = rows_book(10);
    book.update_cell(&phase2_addr(12, 2), "=SUM(A1:A10)").expect("set");
    book.delete_rows(10, 1).expect("delete last");
    book.evaluate();
    let last = book.get_cell_addr_contents(&phase2_addr(11, 2)).unwrap();
    let last_value = rendered(&book, 11, 2);
    println!("first deleted: {first} = {first_value}; last deleted: {last} = {last_value}");
    // Unlike Excel, which shrinks the range, ironcalc breaks it.
    assert_eq!(("=SUM(#REF!:A9)", "#VALUE!"), (first.as_str(), first_value.as_str()));
    assert_eq!(("=SUM(A1:#REF!)", "#VALUE!"), (last.as_str(), last_value.as_str()));
}

/// ironcalc does not move merged ranges on delete (R8, known limitation).
#[test]
fn test_delete_rows_does_not_move_merged_ranges() {
    let mut model = Model::new_empty("test", "en", "America/New_York", "en").expect("model");
    model.workbook.worksheets[0].merge_cells.push("A5:B6".to_string());
    let mut book = Book::from_model(model);
    book.update_cell(&phase2_addr(1, 1), "x").expect("set");
    book.delete_rows(1, 2).expect("delete");
    let merged = &book.model.get_model().workbook.worksheets[0].merge_cells;
    assert_eq!(&vec!["A5:B6".to_string()], merged);
}

#[test]
fn test_delete_rows_shifts_conditional_format_ranges() {
    use ironcalc::base::cf_types::{CfRule, ConditionalFormatting};
    let mut model = Model::new_empty("test", "en", "America/New_York", "en").expect("model");
    model.workbook.worksheets[0].conditional_formatting.push(ConditionalFormatting {
        range: "A5:A8".to_string(),
        cf_rule: CfRule::ColorScale { thresholds: vec![] },
        priority: 1,
    });
    let mut book = Book::from_model(model);
    book.update_cell(&phase2_addr(1, 1), "x").expect("set");
    book.delete_rows(1, 2).expect("delete");
    let cf = &book.model.get_model().workbook.worksheets[0].conditional_formatting;
    assert_eq!("A3:A6", cf[0].range);
}

// -------------------------------------------------------------------------
// Delete columns

fn row_vals(book: &Book, row: usize, cols: std::ops::RangeInclusive<usize>) -> Vec<String> {
    cols.map(|c| rendered(book, row, c)).collect()
}

fn cols_book(n: usize) -> Book {
    let mut book = Book::default();
    for c in 1..=n {
        book.update_cell(&phase2_addr(1, c), c.to_string()).expect("set");
    }
    book.evaluate();
    book.clear_history();
    book.dirty = false;
    book
}

#[test]
fn test_delete_columns_shifts_columns_left() {
    let mut book = cols_book(6);
    book.delete_columns(2, 1).expect("delete");
    assert_eq!(row_vals(&book, 1, 1..=5), vec!["1", "3", "4", "5", "6"]);
    assert!(book.dirty);
    let mut book = cols_book(6);
    book.delete_columns(2, 3).expect("delete");
    assert_eq!(row_vals(&book, 1, 1..=3), vec!["1", "5", "6"]);
}

#[test]
fn test_delete_columns_cursor_rules() {
    let mut book = cols_book(6);
    at(&mut book, 1, 3);
    book.delete_columns(2, 3).expect("delete");
    assert_eq!(book.location, phase2_addr(1, 2));
    let mut book = cols_book(6);
    at(&mut book, 1, 1);
    book.delete_columns(2, 3).expect("delete");
    assert_eq!(book.location, phase2_addr(1, 1));
    let mut book = cols_book(10);
    at(&mut book, 1, 10);
    book.delete_columns(3, 4).expect("delete");
    assert_eq!(book.location, phase2_addr(1, 6));
    assert_eq!("10", book.get_current_cell_rendered().unwrap());
    let mut book = cols_book(4);
    at(&mut book, 1, 4);
    book.delete_columns(4, 1).expect("delete");
    assert_eq!(book.location, phase2_addr(1, 4));
    assert_eq!("", book.get_current_cell_rendered().unwrap());
}

#[test]
fn test_delete_columns_is_one_undo_step() {
    let mut book = cols_book(6);
    at(&mut book, 1, 3);
    book.delete_columns(2, 3).expect("delete");
    assert!(book.undo().expect("undo"));
    assert_eq!(row_vals(&book, 1, 1..=6), vec!["1", "2", "3", "4", "5", "6"]);
    assert_eq!(book.location, phase2_addr(1, 3));
    assert!(!book.dirty);
    assert!(!book.undo().expect("nothing before"));
}

#[test]
fn test_delete_columns_errors_change_nothing() {
    let mut book = cols_book(3);
    at(&mut book, 1, 2);
    assert!(book.delete_columns(2, 0).is_err());
    assert!(book.delete_columns(16_384, 2).is_err());
    assert!(book.delete_columns(16_385, 1).is_err());
    assert_eq!(row_vals(&book, 1, 1..=3), vec!["1", "2", "3"]);
    assert_eq!(book.location, phase2_addr(1, 2));
    assert!(!book.dirty);
    assert!(!book.undo().expect("undo"));
}

#[test]
fn test_delete_columns_partly_overlapping_an_array_formula_is_refused() {
    let mut book = csv_book("1,2,3,4\n");
    book.model.set_user_array_formula(0, 1, 2, 2, 1, "=A1:B1*2").expect("set");
    book.evaluate();
    book.clear_history();
    book.dirty = false;
    let before = row_vals(&book, 1, 1..=4);
    let err = book.delete_columns(3, 1).expect_err("array");
    assert!(err.to_string().contains("array formula"), "{err}");
    assert_eq!(row_vals(&book, 1, 1..=4), before);
    assert!(!book.dirty);
    assert!(!book.undo().expect("undo"));
}

#[test]
fn test_delete_columns_rewrites_references() {
    let mut book = cols_book(9);
    book.update_cell(&phase2_addr(2, 1), "=E1").expect("set");
    book.update_cell(&phase2_addr(2, 2), "=I1").expect("set");
    book.evaluate();
    book.delete_columns(5, 1).expect("delete");
    book.evaluate();
    assert_eq!("#REF!", rendered(&book, 2, 1));
    assert_eq!("=H1", book.get_cell_addr_contents(&phase2_addr(2, 2)).unwrap());
    assert_eq!("9", rendered(&book, 2, 2));
}

// -------------------------------------------------------------------------
// Fill down

fn fill_book(cells: &[(usize, usize, &str)]) -> Book {
    let mut book = Book::default();
    for (r, c, v) in cells {
        book.update_cell(&phase2_addr(*r, *c), v.to_string()).expect("set");
    }
    book.evaluate();
    book.clear_history();
    book.dirty = false;
    book
}

#[test]
fn test_fill_down_single_seed_copies_and_does_not_continue() {
    for (seed, expect) in [("5", "5"), ("Item 1", "Item 1"), ("Jan", "Jan")] {
        let mut book = fill_book(&[(1, 1, seed)]);
        at(&mut book, 2, 1);
        book.fill_down(3).expect("fill");
        assert_eq!(column(&book, 1, 1..=4), vec![expect; 4], "seed {seed}");
    }
}

#[test]
fn test_fill_down_moves_formula_references() {
    let mut book = fill_book(&[(1, 1, "1"), (2, 1, "2"), (1, 2, "=A1+1")]);
    at(&mut book, 2, 2);
    book.fill_down(1).expect("fill");
    assert_eq!("=A2+1", book.get_cell_addr_contents(&phase2_addr(2, 2)).unwrap());
    assert_eq!("3", book.get_cell_addr_rendered(&phase2_addr(2, 2)).unwrap());
}

#[test]
fn test_fill_down_count_fills_cursor_and_below_and_cursor_stays() {
    let mut book = fill_book(&[(1, 1, "x"), (4, 1, "keep")]);
    at(&mut book, 2, 1);
    book.fill_down(5).expect("fill");
    assert_eq!(column(&book, 1, 1..=7), vec!["x", "x", "x", "x", "x", "x", ""]);
    assert_eq!(book.location, phase2_addr(2, 1));
}

#[test]
fn test_fill_down_copies_formats_one_undo_step_and_dirty() {
    let mut book = fill_book(&[(1, 1, "x")]);
    let area = Area { sheet: 0, row: 1, column: 1, width: 1, height: 1 };
    book.set_cell_style(&[("font.b", "true")], &area).expect("bold");
    book.clear_history();
    book.dirty = false;
    at(&mut book, 2, 1);
    book.fill_down(3).expect("fill");
    assert!(book.dirty);
    for row in 2..=4 {
        assert_eq!(Some(true), book.get_cell_style(&phase2_addr(row, 1)).map(|s| s.font.b));
    }
    assert!(book.undo().expect("undo"));
    assert_eq!(column(&book, 1, 1..=4), vec!["x", "", "", ""]);
    assert_eq!(Some(false), book.get_cell_style(&phase2_addr(3, 1)).map(|s| s.font.b));
    assert!(!book.undo().expect("nothing more"));
}

#[test]
fn test_fill_down_with_nothing_above_changes_nothing() {
    let mut book = fill_book(&[(1, 1, "x")]);
    at(&mut book, 1, 1);
    assert!(book.fill_down(1).is_err());
    assert!(!book.dirty);
    assert!(!book.undo().expect("no step"));
}

#[test]
fn test_fill_down_past_the_last_row_changes_nothing() {
    let mut book = fill_book(&[(1, 1, "x")]);
    at(&mut book, 2, 1);
    assert!(book.fill_down(1_048_576).is_err());
    assert!(book.fill_down(0).is_err());
    assert!(!book.dirty);
    assert!(!book.undo().expect("no step"));
    assert_eq!("", book.get_cell_addr_rendered(&phase2_addr(2, 1)).unwrap());
}

#[test]
fn test_fill_down_refused_by_ironcalc_adds_no_step_or_dirty() {
    // A two row array formula in A3:A4; filling A3 only covers part of it.
    let mut book = fill_book(&[(2, 1, "x"), (3, 2, "1"), (4, 2, "2")]);
    book.model
        .set_user_array_formula(0, 3, 1, 1, 2, "=B3:B4")
        .expect("array");
    book.clear_history();
    book.dirty = false;
    at(&mut book, 3, 1);
    assert!(book.fill_down(1).is_err());
    assert!(!book.dirty);
    assert!(!book.undo().expect("no step"));
}

// -------------------------------------------------------------------------
// Fill series

fn series(book: &mut Book, from: (usize, usize), to: (usize, usize), dir: crate::book::FillDirection, n: usize) -> anyhow::Result<()> {
    book.fill_series(&phase2_addr(from.0, from.1), &phase2_addr(to.0, to.1), dir, n)
}

#[test]
fn test_fill_series_numbers_down_and_up() {
    use crate::book::FillDirection::*;
    let mut book = fill_book(&[(5, 1, "1"), (6, 1, "2")]);
    series(&mut book, (5, 1), (6, 1), Down, 3).expect("fill");
    assert_eq!(column(&book, 1, 5..=9), vec!["1", "2", "3", "4", "5"]);
    assert!(book.dirty);
    let mut book = fill_book(&[(5, 1, "10"), (6, 1, "8")]);
    series(&mut book, (5, 1), (6, 1), Down, 2).expect("fill");
    assert_eq!(column(&book, 1, 5..=8), vec!["10", "8", "6", "4"]);
    let mut book = fill_book(&[(5, 1, "3"), (6, 1, "4")]);
    series(&mut book, (5, 1), (6, 1), Up, 2).expect("fill");
    assert_eq!(column(&book, 1, 3..=6), vec!["1", "2", "3", "4"]);
}

#[test]
fn test_fill_series_text_and_names_and_left_right() {
    use crate::book::FillDirection::*;
    let mut book = fill_book(&[(1, 1, "Item 1"), (2, 1, "Item 2"), (1, 2, "JAN"), (2, 2, "FEB"), (1, 3, "Jan"), (2, 3, "Feb")]);
    series(&mut book, (1, 1), (2, 3), Down, 1).expect("fill");
    assert_eq!("Item 3", book.get_cell_addr_contents(&phase2_addr(3, 1)).unwrap());
    assert_eq!("MAR", book.get_cell_addr_contents(&phase2_addr(3, 2)).unwrap());
    assert_eq!("Mar", book.get_cell_addr_contents(&phase2_addr(3, 3)).unwrap());
    let mut book = fill_book(&[(1, 3, "1"), (1, 4, "2")]);
    series(&mut book, (1, 3), (1, 4), Right, 2).expect("fill");
    assert_eq!("4", book.get_cell_addr_contents(&phase2_addr(1, 6)).unwrap());
    series(&mut book, (1, 3), (1, 4), Left, 2).expect("fill");
    assert_eq!("0", book.get_cell_addr_contents(&phase2_addr(1, 2)).unwrap());
    assert_eq!("-1", book.get_cell_addr_contents(&phase2_addr(1, 1)).unwrap());
}

#[test]
fn test_fill_series_single_seed_copies_and_blocks_repeat() {
    use crate::book::FillDirection::*;
    let mut book = fill_book(&[(1, 1, "1")]);
    series(&mut book, (1, 1), (1, 1), Down, 3).expect("fill");
    assert_eq!(column(&book, 1, 1..=4), vec!["1"; 4]);
    let mut book = fill_book(&[(1, 1, "a"), (2, 1, "x"), (3, 1, "q")]);
    series(&mut book, (1, 1), (3, 1), Down, 3).expect("fill");
    assert_eq!(column(&book, 1, 1..=6), vec!["a", "x", "q", "a", "x", "q"]);
}

#[test]
fn test_fill_series_formulas_move_and_errors_change_nothing() {
    use crate::book::FillDirection::*;
    let mut book = fill_book(&[(1, 1, "1"), (1, 2, "=A1+1")]);
    series(&mut book, (1, 2), (1, 2), Down, 2).expect("fill");
    assert_eq!("=A3+1", book.get_cell_addr_contents(&phase2_addr(3, 2)).unwrap());
    let mut book = fill_book(&[(1, 1, "1")]);
    assert!(series(&mut book, (1, 1), (1, 1), Up, 1).is_err());
    assert!(series(&mut book, (1, 1), (1, 1), Left, 1).is_err());
    assert!(series(&mut book, (1, 1), (1, 1), Down, 2_000_000).is_err());
    assert!(series(&mut book, (1, 1), (1, 1), Down, 0).is_err());
    assert!(!book.dirty);
}

#[test]
fn test_fill_series_is_one_undo_step() {
    use crate::book::FillDirection::*;
    let mut book = fill_book(&[(1, 1, "1"), (2, 1, "2")]);
    series(&mut book, (1, 1), (2, 1), Down, 3).expect("fill");
    book.undo().expect("undo");
    assert_eq!(column(&book, 1, 1..=5), vec!["1", "2", "", "", ""]);
}

#[test]
fn test_set_col_widths_round_trip_sui_and_xlsx() {
    let mut book = Book::default();
    book.update_cell(&Address { sheet: 0, row: 1, col: 1 }, "1").unwrap();
    book.set_col_widths(2, 3, 30).unwrap();
    book.set_col_widths(1, 1, 255).unwrap();
    for ext in ["sui", "xlsx"] {
        let path = tmp_path(&format!("col_widths.{ext}"));
        book.save_as(&path).unwrap();
        let loaded = Book::load(&path, "en", "America/New_York").unwrap();
        assert_eq!(255, loaded.get_col_size(1).unwrap(), "{ext}");
        for c in 2..=4 {
            assert_eq!(30, loaded.get_col_size(c).unwrap(), "{ext}");
        }
    }
}

#[test]
fn test_set_col_widths_past_the_last_column_changes_nothing() {
    let mut book = Book::default();
    let last = crate::book::LAST_COLUMN as usize;
    let before = book.get_col_size(last).unwrap();
    assert!(book.set_col_widths(last, 2, 9).is_err());
    assert!(book.set_col_widths(1, 0, 9).is_err());
    assert_eq!(before, book.get_col_size(last).unwrap());
    assert!(!book.undo().unwrap());
}

fn autofit_book(cells: &[(usize, usize, &str)]) -> Book {
    let mut book = Book::default();
    for (row, col, value) in cells {
        book.update_cell(&Address { sheet: 0, row: *row, col: *col }, *value).unwrap();
    }
    book
}

#[test]
fn test_autofit_width_text_formula_format_multiline_and_cjk() {
    let book = autofit_book(&[(1, 1, "hello"), (2, 1, "hi"), (1, 2, "=\"abcdefgh\""), (1, 3, "a\nlonger line\nb"), (1, 4, "日本語")]);
    assert_eq!(Some(5), book.autofit_col_width(1, None).unwrap());
    assert_eq!(Some(8), book.autofit_col_width(2, None).unwrap());
    assert_eq!(Some(11), book.autofit_col_width(3, None).unwrap());
    assert_eq!(Some(6), book.autofit_col_width(4, None).unwrap());
    // An empty column has nothing to measure.
    assert_eq!(None, book.autofit_col_width(5, None).unwrap());
}

#[test]
fn test_autofit_width_measures_the_displayed_value() {
    let mut book = autofit_book(&[(1, 1, "12"), (1, 2, "=1/3")]);
    assert_eq!(Some(3), book.autofit_col_width(1, None).unwrap());
    let area = Area { sheet: 0, row: 1, column: 1, width: 1, height: 1 };
    book.set_cell_style(&[("num_fmt", "$#,##0.00")], &area).unwrap();
    assert_eq!("$12.00", book.get_cell_addr_rendered(&Address { sheet: 0, row: 1, col: 1 }).unwrap());
    assert_eq!(Some(6), book.autofit_col_width(1, None).unwrap());
    // A formula's result, not its source.
    let shown = book.get_cell_addr_rendered(&Address { sheet: 0, row: 1, col: 2 }).unwrap();
    assert!(shown.len() > 4, "{shown}");
    assert_eq!(Some(shown.len()), book.autofit_col_width(2, None).unwrap());
}

#[test]
fn test_autofit_width_clamps() {
    let book = autofit_book(&[(1, 1, &"x".repeat(100)), (1, 2, "x")]);
    assert_eq!(Some(60), book.autofit_col_width(1, None).unwrap());
    assert_eq!(Some(3), book.autofit_col_width(2, None).unwrap());
}

#[test]
fn test_autofit_width_counts_offscreen_rows_and_honors_row_range() {
    let book = autofit_book(&[(1, 1, "a"), (500, 1, "twelve chars"), (3, 1, "mid")]);
    assert_eq!(Some(12), book.autofit_col_width(1, None).unwrap());
    assert_eq!(Some(3), book.autofit_col_width(1, Some(1..=3)).unwrap());
    assert_eq!(None, book.autofit_col_width(1, Some(10..=20)).unwrap());
}

#[test]
fn test_autofit_width_skips_merged_cells() {
    let mut model = Model::new_empty("m", "en", "America/New_York", "en").unwrap();
    model.workbook.worksheets[0].merge_cells.push("A1:B2".to_string());
    let mut book = Book::from_model(model);
    book.update_cell(&Address { sheet: 0, row: 1, col: 1 }, "a very long merged cell").unwrap();
    book.update_cell(&Address { sheet: 0, row: 3, col: 1 }, "short").unwrap();
    assert_eq!(Some(5), book.autofit_col_width(1, None).unwrap());
    // A column holding only merged cells has nothing to measure.
    assert_eq!(None, book.autofit_col_width(2, None).unwrap());
}

#[test]
fn test_autofit_cols_resets_empty_columns_and_is_one_undo_step() {
    let mut book = autofit_book(&[(1, 1, "hello world"), (1, 3, "x")]);
    book.set_col_widths(1, 3, 40).unwrap();
    let report = book.autofit_cols(1, 3, None).unwrap();
    assert_eq!(AutofitReport { fitted: 2, empty: 1 }, report);
    assert_eq!(11, book.get_col_size(1).unwrap());
    assert_eq!(DEFAULT_COL_WIDTH, book.get_col_size(2).unwrap());
    assert_eq!(3, book.get_col_size(3).unwrap());
    assert!(book.undo().unwrap());
    for c in 1..=3 {
        assert_eq!(40, book.get_col_size(c).unwrap());
    }
}

#[test]
fn test_autofit_cols_past_the_last_column_changes_nothing() {
    let mut book = Book::default();
    let last = crate::book::LAST_COLUMN as usize;
    let before = book.get_col_size(last).unwrap();
    assert!(book.autofit_cols(last, 2, None).is_err());
    assert!(book.autofit_cols(1, 0, None).is_err());
    assert_eq!(before, book.get_col_size(last).unwrap());
    assert!(!book.undo().unwrap());
}

#[test]
fn test_unset_columns_follow_the_default_width() {
    let mut book = Book::default();
    book.set_col_widths(1, 1, 30).unwrap();
    book.set_default_width(24);
    assert_eq!(30, book.get_col_size(1).unwrap());
    assert_eq!(24, book.get_col_size(2).unwrap());
    // A column set to exactly 18 counts as unset: ironcalc cannot tell.
    book.set_col_widths(3, 1, 18).unwrap();
    assert_eq!(24, book.get_col_size(3).unwrap());
}

#[test]
fn test_clear_col_widths_past_the_last_column_changes_nothing() {
    let mut book = Book::default();
    let last = crate::book::LAST_COLUMN as usize;
    book.set_col_widths(last, 1, 9).unwrap();
    assert!(book.clear_col_widths(last, 2).is_err());
    assert_eq!(9, book.get_col_size(last).unwrap());
}

#[test]
fn test_default_width_is_never_saved() {
    let mut book = Book::default();
    book.set_default_width(24);
    book.set_col_widths(2, 1, 30).unwrap();
    let text = crate::book::sui::serialize_sui(&book);
    assert!(text.contains("col 2 width 30"), "{text}");
    assert!(!text.contains("width 24"), "{text}");
    assert!(!text.contains("col 1 width"), "{text}");
    // Read with another default, the unset columns show that default.
    let (mut read, _) = crate::book::sui::parse_sui(&text);
    read.set_default_width(10);
    assert_eq!(10, read.get_col_size(1).unwrap());
    assert_eq!(30, read.get_col_size(2).unwrap());
}

#[test]
fn test_xlsx_writes_no_width_for_columns_following_the_default() {
    let mut book = Book::default();
    book.set_default_width(24);
    book.set_col_widths(2, 1, 30).unwrap();
    let dir = std::env::temp_dir().join(format!("sheetui-dw-xlsx-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("w.xlsx");
    book.save_as(&path).unwrap();
    let mut read = Book::load(&path, "en", "America/New_York").unwrap();
    read.set_default_width(10);
    assert_eq!(10, read.get_col_size(1).unwrap());
    assert_eq!(30, read.get_col_size(2).unwrap());
}

fn fmt_area(row: i32, column: i32, width: i32, height: i32) -> Area {
    Area { sheet: 0, row, column, width, height }
}

#[test]
fn test_number_format_presets() {
    let expect = [
        ("general", "General"),
        ("number", "0.00"),
        ("integer", "0"),
        ("thousands", "#,##0"),
        ("currency", "$#,##0.00"),
        ("percent", "0.00%"),
        ("date", "yyyy-mm-dd"),
        ("time", "hh:mm:ss"),
        ("scientific", "0.00E+00"),
    ];
    for (name, code) in expect {
        assert_eq!(number_format_preset(name), Some(code));
        assert_eq!(number_format_preset(&name.to_uppercase()), Some(code));
        let mut book = Book::default();
        book.set_number_format(code, &fmt_area(1, 1, 1, 1)).expect("format");
        assert_eq!(book.get_number_format(&phase2_addr(1, 1)).as_deref(), Some(code));
    }
    assert_eq!(number_format_preset("0.00"), None);
}

#[test]
fn test_check_number_format() {
    for ok in ["0.00", "0.00 \"USD\"", "#,##0;[Red]-#,##0", "yyyy-mm-dd", "General"] {
        assert!(check_number_format(ok).is_ok(), "{ok} should be accepted");
    }
    for bad in ["", "0.00;0.00;0.00;@;0", "\"abc", "yyyy-mm-dd\"x"] {
        let err = check_number_format(bad).expect_err(bad).to_string();
        assert_eq!(err, format!("Invalid number format: {bad}"));
    }
}

#[test]
fn test_set_number_format_bad_code_changes_nothing() {
    let mut book = Book::default();
    book.update_cell(&phase2_addr(1, 1), "5").expect("edit");
    let before = book.get_number_format(&phase2_addr(1, 1));
    assert!(book.set_number_format("\"abc", &fmt_area(1, 1, 2, 2)).is_err());
    assert_eq!(book.get_number_format(&phase2_addr(1, 1)), before);
    assert_eq!(book.get_number_format(&phase2_addr(2, 2)), before);
}

#[test]
fn test_set_number_format_empty_text_and_number_cells() {
    let mut book = Book::default();
    book.update_cell(&phase2_addr(1, 1), "text").expect("edit");
    book.update_cell(&phase2_addr(1, 2), "0.5").expect("edit");
    book.set_number_format("0.00%", &fmt_area(1, 1, 3, 1)).expect("format");
    book.evaluate();
    assert_eq!(book.get_cell_addr_rendered(&phase2_addr(1, 1)).unwrap(), "text");
    assert_eq!(book.get_cell_addr_rendered(&phase2_addr(1, 2)).unwrap(), "50.00%");
    for col in 1..=3 {
        assert_eq!(book.get_number_format(&phase2_addr(1, col)).as_deref(), Some("0.00%"));
    }
    // A number typed later into the empty, formatted cell shows with it.
    book.update_cell(&phase2_addr(1, 3), "0.25").expect("edit");
    book.evaluate();
    assert_eq!(book.get_cell_addr_rendered(&phase2_addr(1, 3)).unwrap(), "25.00%");
}

#[test]
fn test_set_number_format_is_one_undo_step() {
    let mut book = Book::default();
    book.set_number_format("0.000", &fmt_area(1, 1, 3, 3)).expect("format");
    assert_eq!(book.get_number_format(&phase2_addr(3, 3)).as_deref(), Some("0.000"));
    assert!(book.undo().expect("undo"));
    for (r, c) in [(1, 1), (2, 2), (3, 3)] {
        assert_eq!(book.get_number_format(&phase2_addr(r, c)).as_deref(), Some("General"));
    }
    assert!(book.redo().expect("redo"));
    assert_eq!(book.get_number_format(&phase2_addr(1, 1)).as_deref(), Some("0.000"));
}

#[test]
fn test_number_format_does_not_change_col_width() {
    let mut book = Book::default();
    let before = book.model.get_column_width(0, 1).expect("width");
    book.set_number_format("$#,##0.00", &fmt_area(1, 1, 1, 5)).expect("format");
    assert_eq!(book.model.get_column_width(0, 1).expect("width"), before);
}

#[test]
fn test_number_format_preset_roundtrip_sui_and_xlsx() {
    let sui = tmp_path("fmt_roundtrip.sui");
    let xlsx = tmp_path("fmt_roundtrip.xlsx");
    let mut book = Book::default();
    for (i, (_, code)) in super::NUMBER_FORMAT_PRESETS.iter().enumerate() {
        let row = i as i32 + 1;
        book.update_cell(&phase2_addr(row as usize, 1), "1234.5").expect("edit");
        book.set_number_format(code, &fmt_area(row, 1, 1, 1)).expect("format");
    }
    book.save_as(&sui).expect("save sui");
    book.save_as(&xlsx).expect("save xlsx");
    for path in [&sui, &xlsx] {
        let loaded = Book::load(path, "en", "America/New_York").expect("load");
        for (i, (name, code)) in super::NUMBER_FORMAT_PRESETS.iter().enumerate() {
            let got = loaded.get_number_format(&phase2_addr(i + 1, 1));
            assert_eq!(got.as_deref(), Some(*code), "{name} via {path:?}");
        }
    }
}

#[test]
fn test_set_number_format_cols_overrides_and_is_one_undo() {
    let mut book = Book::default();
    book.update_cell(&phase2_addr(2, 2), "5").expect("edit");
    book.set_number_format("0.000", &fmt_area(2, 2, 1, 1)).expect("format");
    let start = std::time::Instant::now();
    book.set_number_format_cols(2, 2, "0.00%").expect("cols");
    assert!(start.elapsed().as_secs() < 5, "a whole column range is slow");
    for (r, c) in [(1, 2), (2, 2), (1000, 3), (1_048_576, 3)] {
        assert_eq!(book.get_number_format(&phase2_addr(r, c)).as_deref(), Some("0.00%"), "{r}:{c}");
    }
    assert_eq!(book.get_number_format(&phase2_addr(1, 1)).as_deref(), Some("General"));
    assert_eq!(book.get_number_format(&phase2_addr(1, 4)).as_deref(), Some("General"));
    assert!(book.undo().expect("undo"));
    assert_eq!(book.get_number_format(&phase2_addr(2, 2)).as_deref(), Some("0.000"));
    assert_eq!(book.get_number_format(&phase2_addr(1, 3)).as_deref(), Some("General"));
}

#[test]
fn test_set_number_format_rows_overrides_and_is_one_undo() {
    let mut book = Book::default();
    book.set_number_format("0.000", &fmt_area(2, 2, 1, 1)).expect("format");
    book.set_number_format_rows(2, 2, "yyyy-mm-dd").expect("rows");
    for (r, c) in [(2, 1), (2, 2), (3, 16_384)] {
        assert_eq!(book.get_number_format(&phase2_addr(r, c)).as_deref(), Some("yyyy-mm-dd"), "{r}:{c}");
    }
    assert_eq!(book.get_number_format(&phase2_addr(1, 1)).as_deref(), Some("General"));
    assert_eq!(book.get_number_format(&phase2_addr(4, 1)).as_deref(), Some("General"));
    assert!(book.undo().expect("undo"));
    assert_eq!(book.get_number_format(&phase2_addr(2, 2)).as_deref(), Some("0.000"));
}

#[test]
fn test_set_number_format_cols_rows_errors_change_nothing() {
    let mut book = Book::default();
    assert!(book.set_number_format_cols(1, 0, "0").is_err());
    assert!(book.set_number_format_cols(16_384, 2, "0").is_err());
    assert!(book.set_number_format_rows(1, 0, "0").is_err());
    assert!(book.set_number_format_rows(1_048_576, 2, "0").is_err());
    assert!(book.set_number_format_cols(1, 1, "\"abc").is_err());
    assert!(book.set_number_format_rows(1, 1, "\"abc").is_err());
    assert_eq!(book.get_number_format(&phase2_addr(1, 1)).as_deref(), Some("General"));
    book.set_number_format_cols(16_384, 1, "0").expect("last column");
    book.set_number_format_rows(1_048_576, 1, "0").expect("last row");
}

#[test]
fn test_change_decimals_plain_formats() {
    use DecimalsChange::{Add, Set};
    let cases = [
        ("General", Add(1), "0.0"),
        ("General", Add(-1), "0"),
        ("general", Add(2), "0.00"),
        ("0.00", Add(1), "0.000"),
        ("0.00", Add(-1), "0.0"),
        ("0.00", Add(-5), "0"),
        ("#,##0", Add(2), "#,##0.00"),
        ("$#,##0.00", Add(-2), "$#,##0"),
        ("0.00%", Add(1), "0.000%"),
        ("0.00", Set(0), "0"),
        ("0", Set(15), "0.000000000000000"),
        ("0.00", Add(20), "0.000000000000000"),
        ("#.##", Add(1), "#.##0"),
    ];
    for (code, change, want) in cases {
        assert_eq!(change_decimals(code, change).expect(code), want, "{code} {change:?}");
    }
}

#[test]
fn test_change_decimals_refuses_other_formats() {
    for code in ["yyyy-mm-dd", "hh:mm:ss", "0.00E+00", "0.00;[Red]0.00", "0.00 \"USD\"", "@", ""] {
        let err = change_decimals(code, DecimalsChange::Add(1)).expect_err(code).to_string();
        assert_eq!(err, format!("Cannot change decimals of {code}"));
    }
}

#[test]
fn test_book_change_decimals_mixed_area_uses_first_cell_one_undo() {
    let mut book = Book::default();
    book.set_number_format("0.00", &fmt_area(1, 1, 1, 1)).expect("format");
    book.set_number_format("0.0%", &fmt_area(1, 2, 1, 1)).expect("format");
    let area = fmt_area(1, 1, 2, 2);
    assert_eq!(book.change_decimals(DecimalsChange::Add(1), &area).expect("decimals"), "0.000");
    for (r, c) in [(1, 1), (1, 2), (2, 1), (2, 2)] {
        assert_eq!(book.get_number_format(&phase2_addr(r, c)).as_deref(), Some("0.000"));
    }
    assert!(book.undo().expect("undo"));
    assert_eq!(book.get_number_format(&phase2_addr(1, 1)).as_deref(), Some("0.00"));
    assert_eq!(book.get_number_format(&phase2_addr(1, 2)).as_deref(), Some("0.0%"));
    assert_eq!(book.get_number_format(&phase2_addr(2, 2)).as_deref(), Some("General"));
}

#[test]
fn test_book_change_decimals_refusal_changes_nothing() {
    let mut book = Book::default();
    book.set_number_format("yyyy-mm-dd", &fmt_area(1, 1, 1, 1)).expect("format");
    book.set_number_format("0.00", &fmt_area(1, 2, 1, 1)).expect("format");
    let err = book.change_decimals(DecimalsChange::Add(1), &fmt_area(1, 1, 2, 1)).expect_err("date");
    assert_eq!(err.to_string(), "Cannot change decimals of yyyy-mm-dd");
    assert_eq!(book.get_number_format(&phase2_addr(1, 2)).as_deref(), Some("0.00"));
}

#[test]
fn test_style_paths_accepted_by_ironcalc_for_text_styles() {
    let mut book = csv_book("a\n");
    let area = Area { sheet: 0, row: 1, column: 1, width: 1, height: 1 };
    let style = |b: &Book| b.get_cell_style(&phase2_addr(1, 1)).expect("style");
    book.set_cell_style(
        &[
            ("font.u", "true"),
            ("font.strike", "true"),
            ("alignment.horizontal", "center"),
            ("alignment.wrap_text", "true"),
            ("alignment.vertical", "top"),
        ],
        &area,
    )
    .expect("set");
    let s = style(&book);
    assert!(s.font.u && s.font.strike);
    let a = s.alignment.expect("alignment");
    assert_eq!(ironcalc::base::types::HorizontalAlignment::Center, a.horizontal);
    // `general` is stored as General, keeping wrap and vertical.
    book.set_cell_style(&[("alignment.horizontal", "general")], &area).expect("general");
    let a = style(&book).alignment.expect("alignment kept");
    assert_eq!(ironcalc::base::types::HorizontalAlignment::General, a.horizontal);
    assert!(a.wrap_text);
    // `alignment` with an empty string drops the whole struct.
    book.set_cell_style(&[("alignment", "")], &area).expect("clear");
    assert!(style(&book).alignment.is_none());
    // Only "true" and "false" are booleans.
    assert!(book.set_cell_style(&[("font.u", "yes")], &area).is_err());
    book.set_cell_style(&[("font.u", "false"), ("font.strike", "false")], &area).expect("off");
    assert!(!style(&book).font.u);
}

#[test]
fn test_full_column_style_merges_one_path_into_cells_with_a_style() {
    let mut book = csv_book("a\nb\n");
    let cell = Area { sheet: 0, row: 1, column: 1, width: 1, height: 1 };
    book.set_cell_style(&[("fill.color", "#ff0000"), ("num_fmt", "0.00")], &cell).expect("fill");
    book.set_col_style(&[("font.b", "true")], 0, 1).expect("col bold");
    let s = book.get_cell_style(&phase2_addr(1, 1)).expect("style");
    assert!(s.font.b);
    assert_eq!("0.00", s.num_fmt);
    assert_ne!(ironcalc::base::types::Color::None, s.fill.color);
    // A cell with no style of its own gets the column's.
    let s = book.get_cell_style(&phase2_addr(2, 1)).expect("style");
    assert!(s.font.b);
}

fn flags_of(book: &Book, row: usize, col: usize) -> (bool, bool, bool, bool) {
    let a = phase2_addr(row, col);
    (
        book.cell_has_style(&a, StyleFlag::Bold),
        book.cell_has_style(&a, StyleFlag::Italic),
        book.cell_has_style(&a, StyleFlag::Underline),
        book.cell_has_style(&a, StyleFlag::Strike),
    )
}

#[test]
fn test_apply_style_each_flag_on_a_cell_and_a_range() {
    let mut book = Book::default();
    book.update_cell(&phase2_addr(1, 1), "text").expect("set");
    for (i, flag) in [StyleFlag::Bold, StyleFlag::Italic, StyleFlag::Underline, StyleFlag::Strike].into_iter().enumerate() {
        book.apply_style(&fmt_area(1, 1, 1, 1), &[StyleAttr::Toggle(flag)]).expect("style");
        let f = flags_of(&book, 1, 1);
        assert_eq!([f.0, f.1, f.2, f.3].iter().filter(|b| **b).count(), i + 1);
    }
    // A range, including empty cells.
    book.apply_style(&fmt_area(3, 3, 3, 3), &[StyleAttr::Toggle(StyleFlag::Underline)]).expect("style");
    for r in 3..=5 {
        for c in 3..=5 {
            assert!(book.cell_has_style(&phase2_addr(r, c), StyleFlag::Underline), "{r}:{c}");
        }
    }
    assert!(!book.cell_has_style(&phase2_addr(6, 6), StyleFlag::Underline));
}

#[test]
fn test_apply_style_toggle_is_decided_by_first_cell_and_ends_uniform() {
    let mut book = Book::default();
    // Second cell bold, first not: the area gains bold.
    book.apply_style(&fmt_area(1, 2, 1, 1), &[StyleAttr::Toggle(StyleFlag::Bold)]).expect("style");
    let done = book.apply_style(&fmt_area(1, 1, 3, 1), &[StyleAttr::Toggle(StyleFlag::Bold)]).expect("style");
    assert_eq!(vec![(StyleFlag::Bold, true)], done);
    for c in 1..=3 {
        assert!(book.cell_has_style(&phase2_addr(1, c), StyleFlag::Bold), "{c}");
    }
    // First cell bold now: the area loses it.
    let done = book.apply_style(&fmt_area(1, 1, 3, 1), &[StyleAttr::Toggle(StyleFlag::Bold)]).expect("style");
    assert_eq!(vec![(StyleFlag::Bold, false)], done);
    for c in 1..=3 {
        assert!(!book.cell_has_style(&phase2_addr(1, c), StyleFlag::Bold), "{c}");
    }
    // Only the first cell decides, not any other.
    book.apply_style(&fmt_area(1, 1, 1, 1), &[StyleAttr::Toggle(StyleFlag::Italic)]).expect("style");
    book.apply_style(&fmt_area(1, 1, 3, 1), &[StyleAttr::Toggle(StyleFlag::Italic)]).expect("style");
    for c in 1..=3 {
        assert!(!book.cell_has_style(&phase2_addr(1, c), StyleFlag::Italic), "{c}");
    }
}

#[test]
fn test_apply_style_set_ignores_current_state_and_keeps_other_attributes() {
    let mut book = Book::default();
    book.set_cell_style(&[("fill.color", "#FF0000")], &fmt_area(1, 1, 1, 1)).expect("fill");
    book.set_number_format("0.00", &fmt_area(1, 1, 1, 1)).expect("fmt");
    book.apply_style(&fmt_area(1, 1, 1, 1), &[StyleAttr::Set(StyleFlag::Bold, false)]).expect("style");
    assert!(!book.cell_has_style(&phase2_addr(1, 1), StyleFlag::Bold));
    book.apply_style(&fmt_area(1, 1, 1, 1), &[StyleAttr::Set(StyleFlag::Bold, true), StyleAttr::Toggle(StyleFlag::Strike)]).expect("style");
    assert_eq!((true, false, false, true), flags_of(&book, 1, 1));
    let style = book.get_cell_style(&phase2_addr(1, 1)).expect("style");
    assert_eq!("0.00", style.num_fmt);
    assert!(!style.fill.color.is_none());
}

#[test]
fn test_apply_style_several_attributes_one_undo_step() {
    let mut book = Book::default();
    book.apply_style(
        &fmt_area(1, 1, 2, 2),
        &[StyleAttr::Toggle(StyleFlag::Bold), StyleAttr::Toggle(StyleFlag::Underline)],
    ).expect("style");
    assert_eq!((true, false, true, false), flags_of(&book, 2, 2));
    book.undo().expect("undo");
    assert_eq!((false, false, false, false), flags_of(&book, 1, 1));
    assert_eq!((false, false, false, false), flags_of(&book, 2, 2));
}

#[test]
fn test_style_round_trips_through_sui_and_xlsx() {
    for ext in ["sui", "xlsx"] {
        let path = tmp_path(&format!("style_round_trip.{ext}"));
        let mut book = Book::default();
        book.update_cell(&phase2_addr(1, 1), "x").expect("set");
        book.apply_style(
            &fmt_area(1, 1, 1, 1),
            &[
                StyleAttr::Toggle(StyleFlag::Bold),
                StyleAttr::Toggle(StyleFlag::Italic),
                StyleAttr::Toggle(StyleFlag::Underline),
                StyleAttr::Toggle(StyleFlag::Strike),
            ],
        ).expect("style");
        book.save_as(&path).expect("save");
        let loaded = Book::load(&path, "en", "America/New_York").expect("load");
        assert_eq!((true, true, true, true), flags_of(&loaded, 1, 1), "{ext}");
        let _ = std::fs::remove_file(&path);
    }
}

#[test]
fn test_apply_style_never_changes_column_width() {
    let mut book = Book::default();
    book.update_cell(&phase2_addr(1, 1), "hello").expect("set");
    let width = book.get_col_size(1).unwrap();
    book.apply_style(&fmt_area(1, 1, 1, 3), &[StyleAttr::Toggle(StyleFlag::Bold)]).expect("style");
    assert_eq!(width, book.get_col_size(1).unwrap());
}

#[test]
fn test_typing_into_a_styled_empty_cell_keeps_its_style() {
    let mut book = Book::default();
    book.apply_style(&fmt_area(2, 2, 1, 1), &[StyleAttr::Toggle(StyleFlag::Bold)]).expect("style");
    book.update_cell(&phase2_addr(2, 2), "typed").expect("set");
    assert!(book.cell_has_style(&phase2_addr(2, 2), StyleFlag::Bold));
}
