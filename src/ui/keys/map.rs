//! User key mappings, and how typed keys resolve against them and the
//! built-in table. See `planning/keys-design.md`, "Lookup".
use std::time::{Duration, Instant};

use super::{builtin::{Lookup, Table}, Key};

/// How long a key that starts a longer user mapping waits for the rest.
pub const KEY_TIMEOUT: Duration = Duration::from_millis(1000);

/// What a user mapping does.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Target {
    /// Replay these keys, against the built-ins only.
    Keys(Vec<Key>),
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
