//! User key mappings, and how typed keys resolve against them and the
//! built-in table. See `planning/keys-design.md`, "Lookup".
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

use crate::ui::theme::parse::Warning;

use super::{
    builtin::{self, Lookup, Table},
    format_keys, parse_key, Key, KeyError,
};

/// How long a key that starts a longer user mapping waits for the rest.
pub const KEY_TIMEOUT: Duration = Duration::from_millis(1000);

/// One step of a mapping's right-hand side.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RhsItem {
    /// A key, replayed against the built-ins.
    Key(Key),
    /// `<Cmd>text<CR>`: a `:` command, run without entering command mode.
    Cmd(String),
}

/// Parses a right-hand side: keys, and `<Cmd>…<CR>` commands.
pub fn parse_rhs(text: &str) -> Result<Vec<RhsItem>, KeyError> {
    const OPEN: &str = "<cmd>";
    const CLOSE: &str = "<cr>";
    let mut items = Vec::new();
    let mut rest = text;
    while !rest.is_empty() {
        if rest.len() >= OPEN.len() && rest[..OPEN.len()].eq_ignore_ascii_case(OPEN) {
            let after = &rest[OPEN.len()..];
            let Some(end) = after.to_ascii_lowercase().find(CLOSE) else {
                return Err(KeyError("<Cmd> needs a closing <CR>".to_owned()));
            };
            let command = after[..end].trim();
            if command.is_empty() {
                return Err(KeyError("<Cmd><CR> has no command in it".to_owned()));
            }
            items.push(RhsItem::Cmd(command.to_owned()));
            rest = &after[end + CLOSE.len()..];
        } else {
            let (key, used) = parse_key(rest)?;
            items.push(RhsItem::Key(key));
            rest = &rest[used..];
        }
    }
    Ok(items)
}

/// Prints a right-hand side in canonical notation.
pub fn format_rhs(items: &[RhsItem]) -> String {
    items
        .iter()
        .map(|item| match item {
            RhsItem::Key(key) => format_keys(&[*key]),
            RhsItem::Cmd(text) => format!("<Cmd>{text}<CR>"),
        })
        .collect()
}

/// What a user mapping does.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Target {
    /// Replay these steps. Keys mean the built-ins, not user mappings.
    Replay(Vec<RhsItem>),
    /// `nunmap`: the key does nothing, even if it is a built-in.
    Unmapped,
}

/// One user mapping.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mapping {
    pub lhs: Vec<Key>,
    pub target: Target,
}

/// What the typed keys are, among user mappings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserLookup<'a> {
    /// Exactly one mapping, and no longer mapping starts with it.
    Match(&'a Target),
    /// Exactly one mapping, but a longer mapping starts with it too.
    MatchOrMore(&'a Target),
    /// The start of a longer mapping and not a mapping itself.
    Prefix,
    /// Not a mapping and not the start of one.
    None,
}

/// The user mappings for one mode.
#[derive(Debug, Default, Clone)]
pub struct UserTable {
    mappings: Vec<Mapping>,
}

impl UserTable {
    /// Adds a mapping, replacing an earlier one with the same left-hand side.
    pub fn insert(&mut self, lhs: Vec<Key>, target: Target) {
        match self.mappings.iter_mut().find(|m| m.lhs == lhs) {
            Some(existing) => existing.target = target,
            None => self.mappings.push(Mapping { lhs, target }),
        }
    }

    /// Removes a user mapping. Returns whether there was one.
    pub fn remove(&mut self, lhs: &[Key]) -> bool {
        let before = self.mappings.len();
        self.mappings.retain(|m| m.lhs != lhs);
        self.mappings.len() != before
    }

    /// Every mapping, in the order they were first made.
    pub fn mappings(&self) -> &[Mapping] {
        &self.mappings
    }

    pub fn is_empty(&self) -> bool {
        self.mappings.is_empty()
    }

    /// Looks up the keys typed so far.
    pub fn lookup(&self, typed: &[Key]) -> UserLookup<'_> {
        let mut exact = None;
        let mut longer = false;
        for m in &self.mappings {
            if m.lhs == typed {
                exact = Some(&m.target);
            } else if m.lhs.len() > typed.len() && m.lhs.starts_with(typed) {
                longer = true;
            }
        }
        match (exact, longer) {
            (Some(t), false) => UserLookup::Match(t),
            (Some(t), true) => UserLookup::MatchOrMore(t),
            (None, true) => UserLookup::Prefix,
            (None, false) => UserLookup::None,
        }
    }

    /// The target of the longest mapping that is the start of `typed`. The
    /// whole of `typed` counts.
    pub fn longest_match(&self, typed: &[Key]) -> Option<&Target> {
        self.mappings
            .iter()
            .filter(|m| typed.starts_with(&m.lhs))
            .max_by_key(|m| m.lhs.len())
            .map(|m| &m.target)
    }
}

/// What to do about the keys typed so far.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Resolution<H> {
    /// Run a built-in handler.
    Builtin(H),
    /// Run a user mapping.
    User(Target),
    /// Wait for more keys. `timeout` is whether the wait ends by itself: it
    /// does when a user mapping could still run, and does not when only a
    /// built-in is waiting for its next key.
    Wait { timeout: bool },
    /// The keys are nothing. Drop them.
    Drop,
}

/// Resolves the keys typed so far.
///
/// User mappings come first. A user mapping that matches, or that the keys
/// start, wins over the built-ins. Only when the keys are neither does the
/// built-in table see them.
pub fn resolve<H: Copy>(
    user: &UserTable,
    builtin: &Table<H>,
    typed: &[Key],
) -> Resolution<H> {
    match user.lookup(typed) {
        UserLookup::Match(target) => Resolution::User(target.clone()),
        UserLookup::MatchOrMore(_) | UserLookup::Prefix => Resolution::Wait { timeout: true },
        UserLookup::None => match builtin.lookup(typed) {
            Lookup::Match(h) => Resolution::Builtin(h),
            Lookup::Prefix => Resolution::Wait { timeout: false },
            Lookup::None => Resolution::Drop,
        },
    }
}

/// What the timeout does with the keys left pending: the longest user mapping
/// that starts them, if any. The rest of the keys are dropped.
pub fn resolve_timeout(user: &UserTable, typed: &[Key]) -> Option<Target> {
    user.longest_match(typed).cloned()
}

/// The mode a mapping command applies to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MapMode {
    /// `n`: navigation mode.
    Navigate,
    /// `v`: range-select mode.
    RangeSelect,
}

/// A parsed `nnoremap`, `vnoremap`, `nunmap` or `vunmap`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MapCommand {
    Map {
        mode: MapMode,
        lhs: Vec<Key>,
        rhs: Vec<RhsItem>,
    },
    Unmap {
        mode: MapMode,
        lhs: Vec<Key>,
    },
    /// `nmap` or `vmap` with no arguments: show the mappings.
    List(MapMode),
}

/// The user mappings, and the state of a key sequence being typed.
#[derive(Debug, Default)]
pub struct KeyMap {
    /// Navigation mode mappings (`nnoremap`).
    pub nav: UserTable,
    /// Range-select mode mappings (`vnoremap`).
    pub range: UserTable,
    /// When the pending keys give up waiting. `None` if nothing is waiting on
    /// a timeout.
    pub deadline: Option<Instant>,
    /// True while a mapping's keys are being replayed. User mappings do not
    /// apply then: the keys mean what the built-ins say.
    pub replaying: bool,
    /// Problems found loading the keys file.
    pub warnings: Vec<Warning>,
    /// The keys file that was read, or tried.
    pub loaded_from: Option<PathBuf>,
}

impl KeyMap {
    /// Starts the clock on a key that is waiting for the rest.
    pub fn start_timeout(&mut self) {
        self.deadline = Some(Instant::now() + KEY_TIMEOUT);
    }

    pub fn clear_timeout(&mut self) {
        self.deadline = None;
    }
}

/// Whether the keys are exactly a built-in key in a mode.
fn is_builtin(mode: MapMode, keys: &[Key]) -> bool {
    match mode {
        MapMode::Navigate => matches!(builtin::navigation().lookup(keys), Lookup::Match(_)),
        MapMode::RangeSelect => matches!(builtin::range_select().lookup(keys), Lookup::Match(_)),
    }
}

impl KeyMap {
    pub fn table(&self, mode: MapMode) -> &UserTable {
        match mode {
            MapMode::Navigate => &self.nav,
            MapMode::RangeSelect => &self.range,
        }
    }

    fn table_mut(&mut self, mode: MapMode) -> &mut UserTable {
        match mode {
            MapMode::Navigate => &mut self.nav,
            MapMode::RangeSelect => &mut self.range,
        }
    }

    /// Runs a mapping command. An error says why nothing changed.
    ///
    /// `unmap` of a user mapping removes it, so the built-in meaning of the
    /// keys, if any, comes back. `unmap` of a built-in key hides it. `unmap` of
    /// keys that are neither is an error.
    pub fn apply(&mut self, command: MapCommand) -> Result<(), String> {
        match command {
            MapCommand::Map { mode, lhs, rhs } => {
                self.table_mut(mode).insert(lhs, Target::Replay(rhs));
                Ok(())
            }
            MapCommand::List(_) => {
                Err("nmap and vmap list mappings and do nothing in a keys file".to_owned())
            }
            MapCommand::Unmap { mode, lhs } => {
                let existing = self
                    .table(mode)
                    .mappings()
                    .iter()
                    .find(|m| m.lhs == lhs)
                    .map(|m| m.target.clone());
                match existing {
                    Some(Target::Unmapped) => Ok(()),
                    Some(Target::Replay(_)) => {
                        self.table_mut(mode).remove(&lhs);
                        Ok(())
                    }
                    None if is_builtin(mode, &lhs) => {
                        self.table_mut(mode).insert(lhs, Target::Unmapped);
                        Ok(())
                    }
                    None => Err(format!("No mapping for {}", format_keys(&lhs))),
                }
            }
        }
    }
}

/// Wraps text in a markdown code span, with a longer fence if it holds a
/// backtick.
pub fn code_span(text: &str) -> String {
    if text.contains('`') {
        format!("`` {text} ``")
    } else {
        format!("`{text}`")
    }
}

/// One bullet for each user mapping in a mode, as markdown. Empty if there are
/// none.
pub fn mapping_lines(mode: MapMode, keymap: &KeyMap) -> String {
    let mut text = String::new();
    for mapping in keymap.table(mode).mappings() {
        let rhs = match &mapping.target {
            Target::Replay(items) => code_span(&format_rhs(items)),
            Target::Unmapped => "(unmapped)".to_owned(),
        };
        text.push_str(&format!(
            "* {} \u{2192} {rhs}\n",
            code_span(&format_keys(&mapping.lhs))
        ));
    }
    text
}

/// The text of the `:nmap` and `:vmap` dialog, as markdown: the user's
/// mappings in a mode, then any warnings from loading the keys file.
pub fn listing_text(mode: MapMode, keymap: &KeyMap) -> String {
    let (title, add) = match mode {
        MapMode::Navigate => ("Navigation mappings", "nnoremap"),
        MapMode::RangeSelect => ("Range-select mappings", "vnoremap"),
    };
    let mut text = format!("# {title}\n\n");
    if keymap.table(mode).is_empty() {
        text.push_str(&format!(
            "No mappings. Add one with `{add} <keys> <rhs>`, or put it in the keys file.\n"
        ));
    }
    text.push_str(&mapping_lines(mode, keymap));
    if let Some(path) = &keymap.loaded_from {
        text.push_str(&format!("\nKeys file: {}\n", path.display()));
    }
    if !keymap.warnings.is_empty() {
        text.push_str("\n## Warnings\n\n");
        for w in &keymap.warnings {
            if w.line == 0 {
                text.push_str(&format!("* {}\n", w.msg));
            } else {
                text.push_str(&format!("* line {}: {}\n", w.line, w.msg));
            }
        }
    }
    text
}
