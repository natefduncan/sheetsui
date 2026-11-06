//! Command mode command parsers.
use ironcalc::base::expressions::utils::{column_to_number, is_valid_column};
use slice_utils::{Measured, Peekable, Seekable, Span, StrCursor};

use crate::book::FillDirection;
use crate::book::substitute::{is_delimiter, Scope};

/// A parsed command entered in during command mode.
#[derive(Debug, PartialEq, Eq)]
pub enum Cmd<'a> {
    Write(Option<&'a str>),
    InsertRows(usize),
    InsertColumns(usize),
    DeleteRows(usize),
    DeleteColumns(usize),
    FillDown(usize),
    FillRight(usize),
    FillSeries(FillDirection, usize),
    /// `:col-width <1-255> [count]`: set `count` columns, from the cursor
    /// going right, to `width` characters.
    ColWidth { width: usize, count: usize },
    /// `:col-width default [count]`: clear the saved width of `count` columns.
    ColWidthDefault { count: usize },
    /// `:autofit-cols [count]`: fit `count` columns, from the cursor going
    /// right, to their contents.
    AutofitCols(usize),
    ColorRows(Option<usize>, String),
    ColorColumns(Option<usize>, String),
    ColorCell(String),
    /// `:format [<preset|code>]`: set the number format of the cursor cell or
    /// selection, or with no argument show the cursor cell's format.
    Format(Option<&'a str>),
    /// `:format-cols [count] <preset|code>` and `:format-rows`: the count is
    /// `None` when not written.
    FormatCols(Option<usize>, &'a str),
    FormatRows(Option<usize>, &'a str),
    RenameSheet(Option<usize>, &'a str),
    NewSheet(Option<&'a str>),
    SelectSheet(&'a str),
    Edit(&'a str),
    Help(Option<&'a str>),
    Theme(Option<&'a str>),
    /// `:config`: list every startup warning.
    Config,
    /// `:set` and its settings, still to parse. See `options.rs`.
    Set(&'a str),
    /// `:noh`: hide the search highlight until the next search.
    NoHighlight,
    /// `:s/pat/rep/flags`, with `%` or `'<,'>` in front for a scope other
    /// than the current cell. `text` is from the delimiter on, still to
    /// parse. See `book/substitute.rs`.
    Substitute {
        scope: Scope,
        text: &'a str,
    },
    ExportCsv(&'a str),
    SystemPaste,
    Sort { skip: usize, keys: Vec<CmdSortKey> },
    /// `:freeze <rows> [cols]`: pin that many rows and columns.
    Freeze { rows: usize, cols: usize },
    /// `:freeze`: pin every row above and column left of the cursor.
    FreezeAtCursor,
    /// `:unfreeze`: clear the freeze.
    Unfreeze,
    Undo,
    Redo,
    Quit,
}

/// One key of a `sort` command.
#[derive(Debug, PartialEq, Eq)]
pub struct CmdSortKey {
    /// The key column. `None` means the cursor's column.
    pub col: Option<usize>,
    pub ascending: bool,
}

/// The range of the last range selection, which `:` in Range Select puts in
/// the prompt.
pub const SELECTION_RANGE: &str = "'<,'>";

/// Parse command text into a `Cmd`.
pub fn parse<'cmd, 'i: 'cmd>(input: &'i str) -> Result<Option<Cmd<'cmd>>, &'static str> {
    // Only `:s` gives the selection range a meaning. Other commands typed in
    // Range Select, like `:sort`, already act on the selection, so they
    // take the range `:` pre-fills and ignore it.
    if let Some(rest) = input.strip_prefix(SELECTION_RANGE) {
        if let Some(cmd) = try_consume_substitute(rest, Scope::Selection)? {
            return Ok(Some(cmd));
        }
        return parse(rest);
    }
    let cursor = StrCursor::new(input);
    // try consume write command.
    if let Some(cmd) = try_consume_write(cursor.clone())? {
        return Ok(Some(cmd));
    }
    if let Some(cmd) = try_consume_new_sheet(cursor.clone())? {
        return Ok(Some(cmd));
    }
    if let Some(cmd) = try_consume_select_sheet(cursor.clone())? {
        return Ok(Some(cmd));
    }
    // try consume insert-row command.
    if let Some(cmd) = try_consume_insert_row(cursor.clone())? {
        return Ok(Some(cmd));
    }
    // try consume insert-col command.
    if let Some(cmd) = try_consume_insert_column(cursor.clone())? {
        return Ok(Some(cmd));
    }
    // try consume delete-rows command.
    if let Some(cmd) = try_consume_delete_row(cursor.clone())? {
        return Ok(Some(cmd));
    }
    if let Some(cmd) = try_consume_delete_column(cursor.clone())? {
        return Ok(Some(cmd));
    }
    if let Some(cmd) = try_consume_fill_down(cursor.clone())? {
        return Ok(Some(cmd));
    }
    if let Some(cmd) = try_consume_fill_right(cursor.clone())? {
        return Ok(Some(cmd));
    }
    if let Some(cmd) = try_consume_fill_series(cursor.clone())? {
        return Ok(Some(cmd));
    }
    if let Some(cmd) = try_consume_col_width(cursor.clone())? {
        return Ok(Some(cmd));
    }
    if let Some(cmd) = try_consume_autofit_cols(cursor.clone())? {
        return Ok(Some(cmd));
    }
    // Try consume export
    if let Some(cmd) = try_consume_export_csv(cursor.clone())? {
        return Ok(Some(cmd));
    }
    // try consume edit command.
    if let Some(cmd) = try_consume_edit(cursor.clone())? {
        return Ok(Some(cmd));
    }
    // try consume help command.
    if let Some(cmd) = try_consume_help(cursor.clone())? {
        return Ok(Some(cmd));
    }
    if let Some(cmd) = try_consume_config(cursor.clone())? {
        return Ok(Some(cmd));
    }
    if let Some(cmd) = try_consume_theme(cursor.clone())? {
        return Ok(Some(cmd));
    }
    if let Some(cmd) = try_consume_set(cursor.clone())? {
        return Ok(Some(cmd));
    }
    if let Some(cmd) = try_consume_nohlsearch(cursor.clone())? {
        return Ok(Some(cmd));
    }
    // try consume quit command.
    if let Some(cmd) = try_consume_quit(cursor.clone())? {
        return Ok(Some(cmd));
    }
    if let Some(cmd) = try_consume_rename_sheet(cursor.clone())? {
        return Ok(Some(cmd));
    }
    if let Some(cmd) = try_consume_color_rows(cursor.clone())? {
        return Ok(Some(cmd));
    }
    if let Some(cmd) = try_consume_color_columns(cursor.clone())? {
        return Ok(Some(cmd));
    }
    if let Some(cmd) = try_consume_color_cell(cursor.clone())? {
        return Ok(Some(cmd));
    }
    if let Some(cmd) = try_consume_format_span(cursor.clone(), "format-cols")? {
        return Ok(Some(cmd));
    }
    if let Some(cmd) = try_consume_format_span(cursor.clone(), "format-rows")? {
        return Ok(Some(cmd));
    }
    if let Some(cmd) = try_consume_format(cursor.clone())? {
        return Ok(Some(cmd));
    }
    if let Some(cmd) = try_consume_system_paste(cursor.clone())? {
        return Ok(Some(cmd));
    }
    if let Some(cmd) = try_consume_sort(cursor.clone())? {
        return Ok(Some(cmd));
    }
    if let Some(cmd) = try_consume_unfreeze(cursor.clone())? {
        return Ok(Some(cmd));
    }
    if let Some(cmd) = try_consume_freeze(cursor.clone())? {
        return Ok(Some(cmd));
    }
    if let Some(cmd) = try_consume_undo(cursor.clone())? {
        return Ok(Some(cmd));
    }
    if let Some(cmd) = try_consume_redo(cursor.clone())? {
        return Ok(Some(cmd));
    }
    // Last: `s` starts other commands, which a letter after it tells apart.
    if let Some(cmd) = try_consume_substitute(input, Scope::Cell)? {
        return Ok(Some(cmd));
    }
    Ok(None)
}

fn compare<'i>(input: StrCursor<'i>, compare: &str) -> bool {
    input.remaining() >= compare.len() && input.span(0..compare.len()) == compare
}

fn is_ws<'r, 'i: 'r>(input: &'r mut StrCursor<'i>) -> bool {
    match input.peek_next() {
        Some(b) => {
            if *b == b' ' || *b == b'\t' || *b == b'\n' || *b == b'\r' {
                input.next();
                true
            } else {
                false
            }
        }
        _ => false,
    }
}

fn try_consume_write<'cmd, 'i: 'cmd>(
    mut input: StrCursor<'i>,
) -> Result<Option<Cmd<'cmd>>, &'static str> {
    const SHORT: &str = "w";
    const LONG: &str = "write";

    if compare(input.clone(), LONG) {
        input.seek(LONG.len());
    } else if compare(input.clone(), SHORT) {
        input.seek(SHORT.len());
        // Should we check for whitespace?
    } else {
        return Ok(None);
    }
    if input.remaining() > 0 && !is_ws(&mut input) {
        return Err("Invalid command: Did you mean to type `write <path>`?");
    }
    let arg = input.span(0..).trim();
    Ok(Some(Cmd::Write(if arg.is_empty() {
        None
    } else {
        Some(arg)
    })))
}

fn try_consume_export_csv<'cmd, 'i: 'cmd>(
    mut input: StrCursor<'i>,
) -> Result<Option<Cmd<'cmd>>, &'static str> {
    const LONG: &str = "export-csv";

    if compare(input.clone(), LONG) {
        input.seek(LONG.len());
    } else {
        return Ok(None);
    }
    if input.remaining() == 0 || !is_ws(&mut input) {
        return Err("Invalid command: Did you mean to type `export <path>`?");
    }
    let arg = input.span(0..).trim();
    Ok(Some(Cmd::ExportCsv(arg)))
}

fn try_consume_new_sheet<'cmd, 'i: 'cmd>(
    mut input: StrCursor<'i>,
) -> Result<Option<Cmd<'cmd>>, &'static str> {
    const LONG: &str = "new-sheet";

    if compare(input.clone(), LONG) {
        input.seek(LONG.len());
    } else {
        return Ok(None);
    }
    if input.remaining() > 0 && !is_ws(&mut input) {
        return Err("Invalid command: Did you mean to type `new-sheet <arg>`?");
    }
    let arg = input.span(0..).trim();
    Ok(Some(Cmd::NewSheet(if arg.is_empty() {
        None
    } else {
        Some(arg)
    })))
}

fn try_consume_select_sheet<'cmd, 'i: 'cmd>(
    mut input: StrCursor<'i>,
) -> Result<Option<Cmd<'cmd>>, &'static str> {
    const LONG: &str = "select-sheet";

    if compare(input.clone(), LONG) {
        input.seek(LONG.len());
    } else {
        return Ok(None);
    }
    if input.remaining() > 0 && !is_ws(&mut input) {
        return Err("Invalid command: Did you mean to type `select-sheet <sheet-name>`?");
    }
    let arg = input.span(0..).trim();
    if arg.is_empty() {
        return Err("Invalid command: Did you forget the sheet name? `select-sheet <sheet-name>`?");
    }
    Ok(Some(Cmd::SelectSheet(arg)))
}

fn try_consume_color_cell<'cmd, 'i: 'cmd>(
    mut input: StrCursor<'i>,
) -> Result<Option<Cmd<'cmd>>, &'static str> {
    const SHORT: &str = "cc";
    const LONG: &str = "color-cell";
    if compare(input.clone(), LONG) {
        input.seek(LONG.len());
    } else if compare(input.clone(), SHORT) {
        input.seek(SHORT.len());
    } else {
        return Ok(None);
    };
    if input.remaining() > 0 && !is_ws(&mut input) {
        return Err("Invalid command: Did you mean to type `color-cell <color>`?");
    }
    let arg = parse_color(input.span(0..).trim())?;
    Ok(Some(Cmd::ColorCell(arg)))
}

fn try_consume_format<'cmd, 'i: 'cmd>(
    mut input: StrCursor<'i>,
) -> Result<Option<Cmd<'cmd>>, &'static str> {
    const LONG: &str = "format";
    const USAGE: &str = "Usage: format [<preset|code>]";
    if !compare(input.clone(), LONG) {
        return Ok(None);
    }
    input.seek(LONG.len());
    if input.remaining() > 0 && !is_ws(&mut input) {
        return Err(USAGE);
    }
    // The format is everything after the command, so a raw code may hold
    // spaces and quotes.
    let arg = input.span(0..).trim();
    Ok(Some(Cmd::Format(if arg.is_empty() { None } else { Some(arg) })))
}

/// `format-cols` and `format-rows`: `[count] <preset|code>`. The count comes
/// first and only when a format follows it, so `3 0` is three of format `0`
/// and a lone `0` is the format `0`.
fn try_consume_format_span<'cmd, 'i: 'cmd>(
    mut input: StrCursor<'i>,
    long: &'static str,
) -> Result<Option<Cmd<'cmd>>, &'static str> {
    const COLS_USAGE: &str = "Usage: format-cols [count] <preset|code>";
    const ROWS_USAGE: &str = "Usage: format-rows [count] <preset|code>";
    let cols = long == "format-cols";
    let usage = if cols { COLS_USAGE } else { ROWS_USAGE };
    if !compare(input.clone(), long) {
        return Ok(None);
    }
    input.seek(long.len());
    if input.remaining() > 0 && !is_ws(&mut input) {
        return Err(usage);
    }
    let arg = input.span(0..).trim();
    let (word, rest) = match arg.split_once(char::is_whitespace) {
        Some((w, r)) => (w, r.trim()),
        None => (arg, ""),
    };
    let (count, fmt) = if !rest.is_empty() && word.bytes().all(|b| b.is_ascii_digit()) {
        (Some(word.parse::<usize>().map_err(|_| usage)?), rest)
    } else {
        (None, arg)
    };
    if fmt.is_empty() {
        return Err(usage);
    }
    Ok(Some(if cols { Cmd::FormatCols(count, fmt) } else { Cmd::FormatRows(count, fmt) }))
}

fn try_consume_insert_row<'cmd, 'i: 'cmd>(
    mut input: StrCursor<'i>,
) -> Result<Option<Cmd<'cmd>>, &'static str> {
    const SHORT: &str = "ir";
    const LONG: &str = "insert-rows";

    if compare(input.clone(), LONG) {
        input.seek(LONG.len());
    } else if compare(input.clone(), SHORT) {
        input.seek(SHORT.len());
    } else {
        return Ok(None);
    };
    if input.remaining() > 0 && !is_ws(&mut input) {
        return Err("Invalid command: Did you mean to type `insert-rows <arg>`?");
    }
    let arg = input.span(0..).trim();
    Ok(Some(Cmd::InsertRows(if arg.is_empty() {
        1
    } else if let Ok(count) = arg.parse() {
        count
    } else {
        return Err("You must pass in a non negative number for the row count");
    })))
}

fn try_consume_insert_column<'cmd, 'i: 'cmd>(
    mut input: StrCursor<'i>,
) -> Result<Option<Cmd<'cmd>>, &'static str> {
    const SHORT: &str = "ic";
    const LONG: &str = "insert-cols";

    if compare(input.clone(), LONG) {
        input.seek(LONG.len());
    } else if compare(input.clone(), SHORT) {
        input.seek(SHORT.len());
    } else {
        return Ok(None);
    };
    if input.remaining() > 0 && !is_ws(&mut input) {
        return Err("Invalid command: Did you mean to type `insert-cols <arg>`?");
    }
    let arg = input.span(0..).trim();
    Ok(Some(Cmd::InsertColumns(if arg.is_empty() {
        1
    } else if let Ok(count) = arg.parse() {
        count
    } else {
        return Err("You must pass in a non negative number for the column count");
    })))
}

fn try_consume_fill_down<'cmd, 'i: 'cmd>(
    mut input: StrCursor<'i>,
) -> Result<Option<Cmd<'cmd>>, &'static str> {
    const LONG: &str = "fill-down";

    if !compare(input.clone(), LONG) {
        return Ok(None);
    }
    input.seek(LONG.len());
    if input.remaining() > 0 && !is_ws(&mut input) {
        return Err("Invalid command: Did you mean to type `fill-down <count>`?");
    }
    let arg = input.span(0..).trim();
    Ok(Some(Cmd::FillDown(if arg.is_empty() {
        1
    } else if let Ok(count) = arg.parse() {
        count
    } else {
        return Err("You must pass in a whole number for the fill count");
    })))
}

fn try_consume_fill_right<'cmd, 'i: 'cmd>(
    mut input: StrCursor<'i>,
) -> Result<Option<Cmd<'cmd>>, &'static str> {
    const LONG: &str = "fill-right";

    if !compare(input.clone(), LONG) {
        return Ok(None);
    }
    input.seek(LONG.len());
    if input.remaining() > 0 && !is_ws(&mut input) {
        return Err("Invalid command: Did you mean to type `fill-right <count>`?");
    }
    let arg = input.span(0..).trim();
    Ok(Some(Cmd::FillRight(if arg.is_empty() {
        1
    } else if let Ok(count) = arg.parse() {
        count
    } else {
        return Err("You must pass in a whole number for the fill count");
    })))
}

fn try_consume_fill_series<'cmd, 'i: 'cmd>(
    mut input: StrCursor<'i>,
) -> Result<Option<Cmd<'cmd>>, &'static str> {
    const LONG: &str = "fill-series";
    const USAGE: &str = "Usage: `fill-series <down|up|left|right> <count>`";

    if !compare(input.clone(), LONG) {
        return Ok(None);
    }
    input.seek(LONG.len());
    if input.remaining() > 0 && !is_ws(&mut input) {
        return Err(USAGE);
    }
    let arg = input.span(0..).trim();
    let mut words = arg.split_whitespace();
    let direction = match words.next() {
        Some("down") => FillDirection::Down,
        Some("up") => FillDirection::Up,
        Some("left") => FillDirection::Left,
        Some("right") => FillDirection::Right,
        _ => return Err(USAGE),
    };
    let count = match words.next().map(|w| w.parse::<usize>()) {
        Some(Ok(count)) if count > 0 => count,
        _ => return Err(USAGE),
    };
    if words.next().is_some() {
        return Err(USAGE);
    }
    Ok(Some(Cmd::FillSeries(direction, count)))
}

/// The widest column `:col-width` and `<C-l>` allow.
pub const MAX_COL_WIDTH: usize = 255;

fn try_consume_col_width<'cmd, 'i: 'cmd>(
    mut input: StrCursor<'i>,
) -> Result<Option<Cmd<'cmd>>, &'static str> {
    const LONG: &str = "col-width";
    const USAGE: &str = "Usage: col-width <1-255|default> [count]";

    if !compare(input.clone(), LONG) {
        return Ok(None);
    }
    input.seek(LONG.len());
    if input.remaining() > 0 && !is_ws(&mut input) {
        return Err(USAGE);
    }
    let arg = input.span(0..).trim();
    let mut words = arg.split_whitespace();
    let first = words.next();
    let width = match first.map(|w| w.parse::<usize>()) {
        Some(Ok(width)) if (1..=MAX_COL_WIDTH).contains(&width) => Some(width),
        _ if first == Some("default") => None,
        _ => return Err(USAGE),
    };
    let count = match words.next().map(|w| w.parse::<usize>()) {
        None => 1,
        Some(Ok(count)) if count > 0 => count,
        _ => return Err(USAGE),
    };
    if words.next().is_some() {
        return Err(USAGE);
    }
    Ok(Some(match width {
        Some(width) => Cmd::ColWidth { width, count },
        None => Cmd::ColWidthDefault { count },
    }))
}

fn try_consume_autofit_cols<'cmd, 'i: 'cmd>(
    mut input: StrCursor<'i>,
) -> Result<Option<Cmd<'cmd>>, &'static str> {
    const LONG: &str = "autofit-cols";
    const USAGE: &str = "Usage: autofit-cols [count]";

    if !compare(input.clone(), LONG) {
        return Ok(None);
    }
    input.seek(LONG.len());
    if input.remaining() > 0 && !is_ws(&mut input) {
        return Err(USAGE);
    }
    let mut words = input.span(0..).split_whitespace();
    let count = match words.next().map(|w| w.parse::<usize>()) {
        None => 1,
        Some(Ok(count)) if count > 0 => count,
        _ => return Err(USAGE),
    };
    if words.next().is_some() {
        return Err(USAGE);
    }
    Ok(Some(Cmd::AutofitCols(count)))
}

fn try_consume_delete_row<'cmd, 'i: 'cmd>(
    mut input: StrCursor<'i>,
) -> Result<Option<Cmd<'cmd>>, &'static str> {
    const SHORT: &str = "dr";
    const LONG: &str = "delete-rows";

    if compare(input.clone(), LONG) {
        input.seek(LONG.len());
    } else if compare(input.clone(), SHORT) {
        input.seek(SHORT.len());
    } else {
        return Ok(None);
    };
    if input.remaining() > 0 && !is_ws(&mut input) {
        return Err("Invalid command: Did you mean to type `delete-rows <arg>`?");
    }
    let arg = input.span(0..).trim();
    Ok(Some(Cmd::DeleteRows(if arg.is_empty() {
        1
    } else if let Ok(count) = arg.parse() {
        count
    } else {
        return Err("You must pass in a whole number for the row count");
    })))
}

fn try_consume_delete_column<'cmd, 'i: 'cmd>(
    mut input: StrCursor<'i>,
) -> Result<Option<Cmd<'cmd>>, &'static str> {
    const SHORT: &str = "dc";
    const LONG: &str = "delete-cols";

    if compare(input.clone(), LONG) {
        input.seek(LONG.len());
    } else if compare(input.clone(), SHORT) {
        input.seek(SHORT.len());
    } else {
        return Ok(None);
    };
    if input.remaining() > 0 && !is_ws(&mut input) {
        return Err("Invalid command: Did you mean to type `delete-cols <arg>`?");
    }
    let arg = input.span(0..).trim();
    Ok(Some(Cmd::DeleteColumns(if arg.is_empty() {
        1
    } else if let Ok(count) = arg.parse() {
        count
    } else {
        return Err("You must pass in a whole number for the column count");
    })))
}

fn try_consume_edit<'cmd, 'i: 'cmd>(
    mut input: StrCursor<'i>,
) -> Result<Option<Cmd<'cmd>>, &'static str> {
    const SHORT: &str = "e";
    const LONG: &str = "edit";

    if compare(input.clone(), LONG) {
        input.seek(LONG.len());
    } else if compare(input.clone(), SHORT) {
        input.seek(SHORT.len());
    } else {
        return Ok(None);
    };
    if input.remaining() > 0 && !is_ws(&mut input) {
        return Err("Invalid command: Did you mean to type `edit <arg>`?");
    }
    let arg = input.span(0..).trim();
    Ok(Some(Cmd::Edit(if arg.is_empty() {
        return Err("You must pass in a path to edit");
    } else {
        arg
    })))
}

fn try_consume_help<'cmd, 'i: 'cmd>(
    mut input: StrCursor<'i>,
) -> Result<Option<Cmd<'cmd>>, &'static str> {
    const SHORT: &str = "?";
    const LONG: &str = "help";

    if compare(input.clone(), LONG) {
        input.seek(LONG.len());
    } else if compare(input.clone(), SHORT) {
        input.seek(SHORT.len());
        // Should we check for whitespace?
    } else {
        return Ok(None);
    }
    if input.remaining() > 0 && !is_ws(&mut input) {
        return Err("Invalid command: Did you mean to type `help <arg>`?");
    }
    let arg = input.span(0..).trim();
    Ok(Some(Cmd::Help(if arg.is_empty() {
        None
    } else {
        Some(arg)
    })))
}

fn try_consume_config<'cmd, 'i: 'cmd>(
    mut input: StrCursor<'i>,
) -> Result<Option<Cmd<'cmd>>, &'static str> {
    const LONG: &str = "config";

    if !compare(input.clone(), LONG) {
        return Ok(None);
    }
    input.seek(LONG.len());
    if input.remaining() > 0 && !is_ws(&mut input) {
        return Err("Invalid command: Did you mean to type `config`?");
    }
    if !input.span(0..).trim().is_empty() {
        return Err("config takes no arguments");
    }
    Ok(Some(Cmd::Config))
}

fn try_consume_theme<'cmd, 'i: 'cmd>(
    mut input: StrCursor<'i>,
) -> Result<Option<Cmd<'cmd>>, &'static str> {
    const LONG: &str = "theme";

    if !compare(input.clone(), LONG) {
        return Ok(None);
    }
    input.seek(LONG.len());
    if input.remaining() > 0 && !is_ws(&mut input) {
        return Err("Invalid command: Did you mean to type `theme <name>`?");
    }
    let arg = input.span(0..).trim();
    Ok(Some(Cmd::Theme(if arg.is_empty() {
        None
    } else {
        Some(arg)
    })))
}

fn try_consume_set<'cmd, 'i: 'cmd>(
    mut input: StrCursor<'i>,
) -> Result<Option<Cmd<'cmd>>, &'static str> {
    const LONG: &str = "set";

    if !compare(input.clone(), LONG) {
        return Ok(None);
    }
    input.seek(LONG.len());
    if input.remaining() > 0 && !is_ws(&mut input) {
        return Err("Invalid command: Did you mean to type `set <option>`?");
    }
    Ok(Some(Cmd::Set(input.span(0..).trim())))
}

/// `:noh`, or any longer start of `:nohlsearch`, as in vim.
fn try_consume_nohlsearch<'cmd, 'i: 'cmd>(
    input: StrCursor<'i>,
) -> Result<Option<Cmd<'cmd>>, &'static str> {
    const SHORT: &str = "noh";
    const LONG: &str = "nohlsearch";

    if !compare(input.clone(), SHORT) {
        return Ok(None);
    }
    let text = input.span(0..).trim_end();
    if text.len() <= LONG.len() && LONG.starts_with(text) {
        Ok(Some(Cmd::NoHighlight))
    } else {
        Err("Invalid command: nohlsearch does not take an argument")
    }
}

/// `:s` or `:substitute`, then a delimiter that cannot start a longer
/// command name, as in vim: `:s/a/b/`, `:s#a#b#`. `scope` is the range
/// already read in front of it. A `%` in front is the whole sheet:
/// `:%s/a/b/`. Only `:s` takes `%`.
fn try_consume_substitute<'cmd, 'i: 'cmd>(
    text: &'i str,
    scope: Scope,
) -> Result<Option<Cmd<'cmd>>, &'static str> {
    const RANGE_ERROR: &str = "Invalid command: Only `s` takes a range: `%s/pattern/replacement/`";
    let (scope, text) = match text.strip_prefix('%') {
        Some(rest) if scope == Scope::Cell => (Scope::Sheet, rest),
        _ => (scope, text),
    };
    let Some(rest) = ["substitute", "s"].iter().find_map(|name| text.strip_prefix(name)) else {
        return if scope == Scope::Sheet { Err(RANGE_ERROR) } else { Ok(None) };
    };
    match rest.chars().next() {
        None => Err("Invalid command: Did you mean to type `s/pattern/replacement/`?"),
        Some(c) if is_delimiter(c) => Ok(Some(Cmd::Substitute { scope, text: rest })),
        Some(_) if scope == Scope::Sheet => Err(RANGE_ERROR),
        Some(_) => Ok(None),
    }
}

fn try_consume_quit<'cmd, 'i: 'cmd>(
    mut input: StrCursor<'i>,
) -> Result<Option<Cmd<'cmd>>, &'static str> {
    const SHORT: &str = "q";
    const LONG: &str = "quit";

    if compare(input.clone(), LONG) {
        input.seek(LONG.len());
    } else if compare(input.clone(), SHORT) {
        input.seek(SHORT.len());
        // Should we check for whitespace?
    } else {
        return Ok(None);
    }
    if input.remaining() > 0 {
        return Err("Invalid command: Quit does not take an argument");
    }
    Ok(Some(Cmd::Quit))
}

fn try_consume_system_paste<'cmd, 'i: 'cmd>(
    mut input: StrCursor<'i>,
) -> Result<Option<Cmd<'cmd>>, &'static str> {
    const LONG: &str = "system-paste";

    if compare(input.clone(), LONG) {
        input.seek(LONG.len());
    } else {
        return Ok(None);
    }
    if input.remaining() > 0 {
        return Err("Invalid command: system-paste does not take an argument");
    }
    Ok(Some(Cmd::SystemPaste))
}

const FREEZE_USAGE: &str =
    "Invalid command: usage is `freeze <rows> [cols]`, with whole numbers, or `unfreeze`";

fn try_consume_unfreeze<'cmd, 'i: 'cmd>(
    mut input: StrCursor<'i>,
) -> Result<Option<Cmd<'cmd>>, &'static str> {
    const LONG: &str = "unfreeze";

    if compare(input.clone(), LONG) {
        input.seek(LONG.len());
    } else {
        return Ok(None);
    }
    if input.remaining() > 0 {
        return Err("Invalid command: unfreeze does not take an argument");
    }
    Ok(Some(Cmd::Unfreeze))
}

fn try_consume_freeze<'cmd, 'i: 'cmd>(
    mut input: StrCursor<'i>,
) -> Result<Option<Cmd<'cmd>>, &'static str> {
    const LONG: &str = "freeze";

    if compare(input.clone(), LONG) {
        input.seek(LONG.len());
    } else {
        return Ok(None);
    }
    if input.remaining() > 0 && !is_ws(&mut input) {
        return Err(FREEZE_USAGE);
    }
    let mut words = input.span(0..).split_whitespace();
    // A count is a run of digits, so `-1`, `1.5` and `x` are all errors.
    let count = |word: Option<&str>| -> Result<Option<usize>, &'static str> {
        match word {
            None => Ok(None),
            Some(w) if w.bytes().all(|b| b.is_ascii_digit()) => w
                .parse()
                .map(Some)
                .map_err(|_| "That is more rows or columns than a sheet has"),
            Some(_) => Err(FREEZE_USAGE),
        }
    };
    let Some(rows) = count(words.next())? else {
        return Ok(Some(Cmd::FreezeAtCursor));
    };
    let cols = count(words.next())?.unwrap_or(0);
    if words.next().is_some() {
        return Err(FREEZE_USAGE);
    }
    Ok(Some(Cmd::Freeze { rows, cols }))
}

fn try_consume_undo<'cmd, 'i: 'cmd>(
    mut input: StrCursor<'i>,
) -> Result<Option<Cmd<'cmd>>, &'static str> {
    const LONG: &str = "undo";

    if compare(input.clone(), LONG) {
        input.seek(LONG.len());
    } else {
        return Ok(None);
    }
    if input.remaining() > 0 {
        return Err("Invalid command: undo does not take an argument");
    }
    Ok(Some(Cmd::Undo))
}

fn try_consume_redo<'cmd, 'i: 'cmd>(
    mut input: StrCursor<'i>,
) -> Result<Option<Cmd<'cmd>>, &'static str> {
    const LONG: &str = "redo";

    if compare(input.clone(), LONG) {
        input.seek(LONG.len());
    } else {
        return Ok(None);
    }
    if input.remaining() > 0 {
        return Err("Invalid command: redo does not take an argument");
    }
    Ok(Some(Cmd::Redo))
}

fn try_consume_rename_sheet<'cmd, 'i: 'cmd>(
    mut input: StrCursor<'i>,
) -> Result<Option<Cmd<'cmd>>, &'static str> {
    const LONG: &str = "rename-sheet";
    if compare(input.clone(), LONG) {
        input.seek(LONG.len());
    } else {
        return Ok(None);
    }
    if input.remaining() > 0 && !is_ws(&mut input) {
        return Err("Invalid command: Did you mean to type `rename-sheet [idx] <new-name>`?");
    }
    let (idx, rest) = try_consume_usize(input.clone());
    let arg = rest.span(0..).trim();
    if arg.is_empty() {
        return Err("Invalid command: `rename-sheet` requires a sheet name argument");
    }
    Ok(Some(Cmd::RenameSheet(idx, arg)))
}

fn try_consume_color_rows<'cmd, 'i: 'cmd>(
    mut input: StrCursor<'i>,
) -> Result<Option<Cmd<'cmd>>, &'static str> {
    const LONG: &str = "color-rows";
    if compare(input.clone(), LONG) {
        input.seek(LONG.len());
    } else {
        return Ok(None);
    }
    if input.remaining() > 0 && !is_ws(&mut input) {
        return Err("Invalid command: Did you mean to type `color-rows [count] <color>`?");
    }
    let (idx, rest) = try_consume_usize(input.clone());
    let arg = parse_color(rest.span(0..).trim())?;
    Ok(Some(Cmd::ColorRows(idx, arg)))
}

fn try_consume_color_columns<'cmd, 'i: 'cmd>(
    mut input: StrCursor<'i>,
) -> Result<Option<Cmd<'cmd>>, &'static str> {
    const LONG: &str = "color-columns";
    if compare(input.clone(), LONG) {
        input.seek(LONG.len());
    } else {
        return Ok(None);
    }
    if input.remaining() > 0 && !is_ws(&mut input) {
        return Err("Invalid command: Did you mean to type `color-columns [count] <color>`?");
    }
    let (idx, rest) = try_consume_usize(input.clone());
    let arg = parse_color(rest.span(0..).trim())?;
    Ok(Some(Cmd::ColorColumns(idx, arg)))
}

const SORT_USAGE: &str =
    "Invalid command: usage is `sort [skip <N>] [asc|desc]` or `sort [skip <N>] <column> [asc|desc] ...`";

fn try_consume_sort<'cmd, 'i: 'cmd>(
    mut input: StrCursor<'i>,
) -> Result<Option<Cmd<'cmd>>, &'static str> {
    const LONG: &str = "sort";

    if compare(input.clone(), LONG) {
        input.seek(LONG.len());
    } else {
        return Ok(None);
    }
    if input.remaining() > 0 && !is_ws(&mut input) {
        return Err(SORT_USAGE);
    }
    let mut words = input.span(0..).split_whitespace().peekable();
    let mut skip = 0;
    if words.next_if(|w| w.eq_ignore_ascii_case("skip")).is_some() {
        skip = words
            .next()
            .and_then(|n| n.parse().ok())
            .ok_or(SORT_USAGE)?;
    }
    let mut keys: Vec<CmdSortKey> = Vec::new();
    // Whether the last key already has a direction written after it.
    let mut has_direction = false;
    for word in words {
        let ascending = if word.eq_ignore_ascii_case("asc") {
            Some(true)
        } else if word.eq_ignore_ascii_case("desc") {
            Some(false)
        } else {
            None
        };
        match (ascending, keys.last_mut()) {
            // A lone direction applies to the cursor's column.
            (Some(ascending), None) => {
                keys.push(CmdSortKey {
                    col: None,
                    ascending,
                });
                has_direction = true;
            }
            // A direction applies to the column just before it.
            (Some(ascending), Some(key)) if !has_direction => {
                key.ascending = ascending;
                has_direction = true;
            }
            (None, last) if last.as_deref().is_none_or(|k| k.col.is_some()) => {
                let name = word.to_ascii_uppercase();
                if !is_valid_column(&name) {
                    return Err(SORT_USAGE);
                }
                let col = column_to_number(&name).map_err(|_| SORT_USAGE)?;
                keys.push(CmdSortKey {
                    col: Some(col as usize),
                    ascending: false,
                });
                has_direction = false;
            }
            _ => return Err(SORT_USAGE),
        }
    }
    if keys.is_empty() {
        keys.push(CmdSortKey {
            col: None,
            ascending: false,
        });
    }
    Ok(Some(Cmd::Sort { skip, keys }))
}

pub(crate) fn parse_color(color: &str) -> Result<String, &'static str> {
    use colorsys::{Ansi256, Rgb};
    if color.is_empty() {
        return Err("Invalid command: `color-columns` requires a color argument");
    }
    let parsed = match color.to_lowercase().as_str() {
        "black" => Ansi256::new(0).as_rgb().to_hex_string(),
        "red" => Ansi256::new(1).as_rgb().to_hex_string(),
        "green" => Ansi256::new(2).as_rgb().to_hex_string(),
        "yellow" => Ansi256::new(3).as_rgb().to_hex_string(),
        "blue" => Ansi256::new(4).as_rgb().to_hex_string(),
        "magenta" => Ansi256::new(5).as_rgb().to_hex_string(),
        "cyan" => Ansi256::new(6).as_rgb().to_hex_string(),
        "gray" | "grey" => Ansi256::new(7).as_rgb().to_hex_string(),
        "darkgrey" | "darkgray" => Ansi256::new(8).as_rgb().to_hex_string(),
        "lightred" => Ansi256::new(9).as_rgb().to_hex_string(),
        "lightgreen" => Ansi256::new(10).as_rgb().to_hex_string(),
        "lightyellow" => Ansi256::new(11).as_rgb().to_hex_string(),
        "lightblue" => Ansi256::new(12).as_rgb().to_hex_string(),
        "lightmagenta" => Ansi256::new(13).as_rgb().to_hex_string(),
        "lightcyan" => Ansi256::new(14).as_rgb().to_hex_string(),
        "white" => Ansi256::new(15).as_rgb().to_hex_string(),
        candidate => {
            if candidate.starts_with("#") {
                candidate.to_string()
            } else if candidate.starts_with("rgb(") {
                if let Ok(rgb) = <Rgb as std::str::FromStr>::from_str(candidate) {
                    // Note that the colorsys rgb model clamps the f64 values to no more
                    // than 255.0 so the below casts are safe.
                    rgb.to_hex_string()
                } else {
                    return Err("Invalid color");
                }
            } else {
                return Err("Invalid color");
            }
        }
    };
    Ok(parsed)
}

fn try_consume_usize<'cmd, 'i: 'cmd>(mut input: StrCursor<'i>) -> (Option<usize>, StrCursor<'i>) {
    let mut out = String::new();
    let original_input = input.clone();
    while input
        .peek_next()
        .map(|c| (*c as char).is_ascii_digit())
        .unwrap_or(false)
    {
        out.push(*input.next().unwrap() as char);
    }
    if !out.is_empty() {
        return (Some(out.parse().unwrap()), input.clone());
    }
    (None, original_input)
}
