//! Color themes. A theme is a line-based text file parsed once into a ratatui
//! `Style` for each slot. See `planning/theme-design.md`.
use std::sync::OnceLock;

use ratatui::style::Style;

pub mod color;
pub mod lookup;
pub mod parse;

pub use parse::parse;

macro_rules! slots {
    ($($variant:ident => $name:literal),+ $(,)?) => {
        /// One themeable part of the screen.
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum Slot {
            $($variant),+
        }

        impl Slot {
            /// Every slot, in declaration order.
            pub const ALL: &'static [Slot] = &[$(Slot::$variant),+];
            /// The number of slots.
            pub const COUNT: usize = Slot::ALL.len();


            /// Finds a slot by its theme file name.
            pub fn from_name(name: &str) -> Option<Slot> {
                match name {
                    $($name => Some(Slot::$variant),)+
                    _ => None,
                }
            }
        }

        /// Displays as the slot's name in a theme file.
        impl std::fmt::Display for Slot {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(match self {
                    $(Slot::$variant => $name),+
                })
            }
        }
    };
}

slots! {
    Cell => "cell",
    CellNumber => "cell.number",
    CellString => "cell.string",
    CellBoolean => "cell.boolean",
    CellDate => "cell.date",
    CellNegative => "cell.negative",
    CellError => "cell.error",
    HeaderEven => "header.even",
    HeaderOdd => "header.odd",
    HeaderRow => "header.row",
    Cursor => "cursor",
    Selection => "selection",
    Search => "search",
    SearchCurrent => "search.current",
    Border => "border",
    Title => "title",
    Tab => "tab",
    TabActive => "tab.active",
    Status => "status",
    Command => "command",
    Edit => "edit",
    Info => "info",
    Error => "error",
    Dialog => "dialog",
    DialogBorder => "dialog.border",
    Heading => "heading",
    Link => "link",
    Hint => "hint",
}

/// A resolved theme: one style per slot. Built once, then only read.
#[derive(Debug, Clone, PartialEq)]
pub struct Theme {
    pub name: String,
    styles: [Style; Slot::COUNT],
}

impl Theme {
    /// A theme with every slot unset. The base that `default.theme` is parsed onto.
    pub fn blank(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            styles: [Style::default(); Slot::COUNT],
        }
    }

    pub fn get(&self, slot: Slot) -> Style {
        self.styles[slot as usize]
    }

    pub(crate) fn set(&mut self, slot: Slot, style: Style) {
        self.styles[slot as usize] = style;
    }
}

const DEFAULT_THEME: &str = include_str!("builtin/default.theme");

impl Default for Theme {
    fn default() -> Self {
        static DEFAULT: OnceLock<Theme> = OnceLock::new();
        DEFAULT
            .get_or_init(|| parse(DEFAULT_THEME, &Theme::blank("default")).0)
            .clone()
    }
}

#[cfg(test)]
mod test;
