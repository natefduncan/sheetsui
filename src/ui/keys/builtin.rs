//! The built-in key tables for navigation and range-select mode.
//!
//! A table maps a sequence of keys to a handler. The handlers are enums, one
//! for each mode, so the key tables, the help and the code that runs a key
//! all read from the same place.
use std::sync::OnceLock;

use crossterm::event::{KeyCode, KeyModifiers};

use super::{format_keys, parse_keys, Key};

macro_rules! handlers {
    ($(#[$meta:meta])* $name:ident { $($variant:ident => ($group:literal, $desc:literal)),+ $(,)? }) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum $name {
            $($variant),+
        }

        impl $name {
            /// Every handler, in declaration order.
            pub const ALL: &'static [$name] = &[$($name::$variant),+];

            /// The help group the handler is listed under.
            pub fn group(self) -> &'static str {
                match self {
                    $($name::$variant => $group),+
                }
            }

            /// A short description for the help.
            pub fn description(self) -> &'static str {
                match self {
                    $($name::$variant => $desc),+
                }
            }
        }
    };
}

handlers! {
    /// What a key does in navigation mode.
    NavHandler {
        MoveLeft => ("cell-navigation", "move one cell to the left"),
        MoveRight => ("cell-navigation", "move one cell to the right"),
        MoveDown => ("cell-navigation", "move one cell down"),
        MoveUp => ("cell-navigation", "move one cell up"),
        MoveTop => ("cell-navigation", "go to the top row in the current column"),
        ClearCell => ("cell-navigation", "delete the cell contents and keep its style"),
        ClearCellAll => ("cell-navigation", "delete the cell contents and its style"),
        NextSheet => ("sheet-navigation", "move to the next sheet"),
        PrevSheet => ("sheet-navigation", "move to the previous sheet"),
        EnterEdit => ("modify", "edit the current cell"),
        Substitute => ("modify", "clear the current cell and edit it"),
        ToggleBold => ("modify", "toggle bold on the cell"),
        ToggleItalic => ("modify", "toggle italic on the cell"),
        ColNarrower => ("modify", "shorten the width of the column"),
        ColWider => ("modify", "lengthen the width of the column"),
        InsertRowBelow => ("modify", "insert a row below, move down and edit"),
        InsertRowAbove => ("modify", "insert a row above, move up and edit"),
        Undo => ("undo", "undo the last change"),
        Redo => ("undo", "redo the last change that was undone"),
        EnterRangeSelect => ("other", "enter range selection mode"),
        EnterVisual => ("other", "enter range selection mode with the start selected"),
        Save => ("other", "save the sheet"),
        CopyCell => ("other", "copy the cell contents"),
        CopyCellFormatted => ("other", "copy the cell as it is displayed"),
        Paste => ("other", "paste into the sheet"),
        EnterCommand => ("other", "enter command mode"),
        Help => ("other", "show the help"),
        Quit => ("other", "quit"),
    }
}

handlers! {
    /// What a key does in range-select mode.
    RangeHandler {
        MoveLeft => ("movement", "move the cursor left"),
        MoveRight => ("movement", "move the cursor right"),
        MoveDown => ("movement", "move the cursor down"),
        MoveUp => ("movement", "move the cursor up"),
        NextSheet => ("movement", "move to the next sheet"),
        PrevSheet => ("movement", "move to the previous sheet"),
        Select => ("selection", "mark the start of the range, or finish it"),
        Extend => ("selection", "extend the cell you started in over the range"),
        CopyFormatted => ("selection", "copy the range as it is displayed"),
        CopyRaw => ("selection", "copy the range contents"),
        ClearRange => ("selection", "delete the contents of the range and keep its style"),
        ClearRangeAll => ("selection", "delete the contents of the range and its style"),
        SortDesc => ("selection", "sort the rows of the range by the cursor column, descending"),
        SortAsc => ("selection", "sort the rows of the range by the cursor column, ascending"),
        EnterCommand => ("other", "enter command mode"),
        Help => ("other", "show the help"),
    }
}

/// Navigation mode keys. A terminal with Alt or Ctrl held sends a different
/// key than the bare one, so each is its own entry.
const NAV_KEYS: &[(&str, NavHandler)] = &[
    ("h", NavHandler::MoveLeft),
    ("<Left>", NavHandler::MoveLeft),
    ("<S-Tab>", NavHandler::MoveLeft),
    ("l", NavHandler::MoveRight),
    ("<Right>", NavHandler::MoveRight),
    ("<Tab>", NavHandler::MoveRight),
    ("j", NavHandler::MoveDown),
    ("<Down>", NavHandler::MoveDown),
    ("<CR>", NavHandler::MoveDown),
    ("k", NavHandler::MoveUp),
    ("<Up>", NavHandler::MoveUp),
    ("<S-CR>", NavHandler::MoveUp),
    ("gg", NavHandler::MoveTop),
    ("d", NavHandler::ClearCell),
    ("D", NavHandler::ClearCellAll),
    ("<C-n>", NavHandler::NextSheet),
    ("<C-p>", NavHandler::PrevSheet),
    ("e", NavHandler::EnterEdit),
    ("i", NavHandler::EnterEdit),
    ("s", NavHandler::Substitute),
    ("B", NavHandler::ToggleBold),
    ("I", NavHandler::ToggleItalic),
    ("<C-h>", NavHandler::ColNarrower),
    ("<C-l>", NavHandler::ColWider),
    ("o", NavHandler::InsertRowBelow),
    ("O", NavHandler::InsertRowAbove),
    ("u", NavHandler::Undo),
    ("r", NavHandler::Redo),
    ("<C-r>", NavHandler::EnterRangeSelect),
    ("v", NavHandler::EnterVisual),
    ("<C-s>", NavHandler::Save),
    ("<C-c>", NavHandler::CopyCell),
    ("y", NavHandler::CopyCell),
    ("Y", NavHandler::CopyCellFormatted),
    ("<C-v>", NavHandler::Paste),
    ("p", NavHandler::Paste),
    (":", NavHandler::EnterCommand),
    ("<A-h>", NavHandler::Help),
    ("q", NavHandler::Quit),
];

/// Range-select mode keys.
const RANGE_KEYS: &[(&str, RangeHandler)] = &[
    ("h", RangeHandler::MoveLeft),
    ("l", RangeHandler::MoveRight),
    ("j", RangeHandler::MoveDown),
    ("k", RangeHandler::MoveUp),
    ("<C-n>", RangeHandler::NextSheet),
    ("<C-p>", RangeHandler::PrevSheet),
    ("<Space>", RangeHandler::Select),
    ("<CR>", RangeHandler::Select),
    ("x", RangeHandler::Extend),
    ("<C-c>", RangeHandler::CopyFormatted),
    ("y", RangeHandler::CopyFormatted),
    ("Y", RangeHandler::CopyRaw),
    ("d", RangeHandler::ClearRange),
    ("D", RangeHandler::ClearRangeAll),
    ("s", RangeHandler::SortDesc),
    ("S", RangeHandler::SortAsc),
    (":", RangeHandler::EnterCommand),
    ("<A-h>", RangeHandler::Help),
];

/// Keys the notation cannot write, so they are built directly: a terminal that
/// reports Ctrl-Shift-C sends a capital `C` with Ctrl.
fn ctrl_capital(c: char) -> Vec<Key> {
    vec![Key::new(KeyCode::Char(c), KeyModifiers::CONTROL)]
}

/// The result of looking up the keys typed so far.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lookup<H> {
    /// The keys are exactly one entry.
    Match(H),
    /// The keys are the start of a longer entry. More keys are needed.
    Prefix,
    /// The keys match nothing.
    None,
}

/// A built-in table with the keys parsed.
#[derive(Debug)]
pub struct Table<H> {
    entries: Vec<(Vec<Key>, H)>,
}

impl<H: Copy> Table<H> {
    /// Parses a table. A bad entry is a bug in the table, so this panics, and
    /// the tests build every table.
    fn build(source: &[(&str, H)], direct: Vec<(Vec<Key>, H)>) -> Self {
        let mut entries: Vec<(Vec<Key>, H)> = Vec::with_capacity(source.len() + direct.len());
        let parsed = source.iter().map(|(text, handler)| {
            let keys = parse_keys(text)
                .unwrap_or_else(|e| panic!("built-in key {text:?} does not parse: {e}"));
            (keys, *handler)
        });
        for (keys, handler) in parsed.chain(direct) {
            assert!(!keys.is_empty(), "a built-in key is empty");
            for (other, _) in &entries {
                assert!(
                    !other.starts_with(&keys) && !keys.starts_with(other),
                    "built-in keys {:?} and {:?} are the same or one starts the other",
                    format_keys(&keys),
                    format_keys(other),
                );
            }
            entries.push((keys, handler));
        }
        Table { entries }
    }

    /// Looks up the keys typed so far.
    pub fn lookup(&self, typed: &[Key]) -> Lookup<H> {
        let mut prefix = false;
        for (keys, handler) in &self.entries {
            if keys.as_slice() == typed {
                return Lookup::Match(*handler);
            }
            if keys.len() > typed.len() && keys.starts_with(typed) {
                prefix = true;
            }
        }
        if prefix {
            Lookup::Prefix
        } else {
            Lookup::None
        }
    }

    /// Every entry, in table order.
    pub fn entries(&self) -> &[(Vec<Key>, H)] {
        &self.entries
    }
}

/// The navigation mode table.
pub fn navigation() -> &'static Table<NavHandler> {
    static TABLE: OnceLock<Table<NavHandler>> = OnceLock::new();
    TABLE.get_or_init(|| Table::build(
            NAV_KEYS,
            vec![(ctrl_capital('C'), NavHandler::CopyCellFormatted)],
        ))
}

/// The range-select mode table.
pub fn range_select() -> &'static Table<RangeHandler> {
    static TABLE: OnceLock<Table<RangeHandler>> = OnceLock::new();
    TABLE.get_or_init(|| Table::build(
            RANGE_KEYS,
            vec![(ctrl_capital('C'), RangeHandler::CopyRaw)],
        ))
}
