//! The key lists in the help, built from the live tables.
use super::{
    builtin::{self, HandlerInfo, Lookup, Table},
    format_keys,
    map::{code_span, mapping_lines, KeyMap, MapMode, RhsItem, Target, UserLookup, UserTable},
    Key,
};

/// Whether a built-in key sequence still runs its handler, given the user's
/// mappings. A user mapping on the keys hides it, or on the start of them, as
/// `g` hides `gg`. A user mapping that only continues from the keys (`gt` next
/// to the built-in `gg`) does not hide the start, but it does hide the keys.
fn reachable(user: &UserTable, keys: &[Key]) -> bool {
    if user.lookup(keys) != UserLookup::None {
        return false;
    }
    (1..keys.len()).all(|n| !matches!(user.lookup(&keys[..n]), UserLookup::Match(_)))
}

/// The built-in handler a mapping runs, if its right-hand side is exactly the
/// keys of one built-in entry.
fn handler_of<H: Copy>(table: &Table<H>, target: &Target) -> Option<H> {
    let Target::Replay(items) = target else {
        return None;
    };
    let keys: Option<Vec<Key>> = items
        .iter()
        .map(|item| match item {
            RhsItem::Key(key) => Some(*key),
            RhsItem::Cmd(_) => None,
        })
        .collect();
    match table.lookup(&keys?) {
        Lookup::Match(handler) => Some(handler),
        _ => None,
    }
}

/// The keys that run a handler now: the built-in keys the user has not
/// hidden, then the user's mappings that run it.
pub fn keys_for<H: HandlerInfo>(table: &Table<H>, user: &UserTable, handler: H) -> Vec<Vec<Key>> {
    let mut keys: Vec<Vec<Key>> = table
        .entries()
        .iter()
        .filter(|(keys, h)| *h == handler && reachable(user, keys))
        .map(|(keys, _)| keys.clone())
        .collect();
    for mapping in user.mappings() {
        if handler_of(table, &mapping.target) == Some(handler) && !keys.contains(&mapping.lhs) {
            keys.push(mapping.lhs.clone());
        }
    }
    keys
}

/// A bullet for each handler in a group that has a key: the keys, then what
/// they do. `None` if no handler is in the group.
pub fn group_text<H: HandlerInfo>(
    table: &Table<H>,
    user: &UserTable,
    group: &str,
) -> Option<String> {
    let handlers: Vec<H> = H::all()
        .iter()
        .copied()
        .filter(|h| h.group_name() == group)
        .collect();
    if handlers.is_empty() {
        return None;
    }
    let mut text = String::new();
    for handler in handlers {
        let keys = keys_for(table, user, handler);
        if keys.is_empty() {
            continue;
        }
        let keys: Vec<String> = keys.iter().map(|k| code_span(&format_keys(k))).collect();
        text.push_str(&format!("* {}: {}\n", keys.join(", "), handler.describe()));
    }
    Some(text)
}

/// The help groups of a mode, in order, without repeats.
pub fn groups(mode: MapMode) -> Vec<&'static str> {
    fn of<H: HandlerInfo>() -> Vec<&'static str> {
        let mut groups: Vec<&'static str> = Vec::new();
        for h in H::all() {
            if !groups.contains(&h.group_name()) {
                groups.push(h.group_name());
            }
        }
        groups
    }
    match mode {
        MapMode::Navigate => of::<builtin::NavHandler>(),
        MapMode::RangeSelect => of::<builtin::RangeHandler>(),
    }
}

/// `{{keys:name}}` on a line of its own gives `name`.
fn placeholder(line: &str) -> Option<&str> {
    line.trim().strip_prefix("{{keys:")?.strip_suffix("}}")
}

/// Replaces each `{{keys:<group>}}` line with the keys of that group in a
/// mode, and adds the user's mappings at the end. A group the mode does not
/// have is left as written, so a typo shows.
///
/// Each placeholder becomes a bullet list. Two lists with only a blank line
/// between them are one list in markdown, so a doc puts a heading or some
/// prose between placeholders.
pub fn expand(text: &str, mode: MapMode, keymap: &KeyMap) -> String {
    let user = keymap.table(mode);
    let mut out = String::with_capacity(text.len());
    for line in text.lines() {
        let expanded = placeholder(line).and_then(|group| match mode {
            MapMode::Navigate => group_text(builtin::navigation(), user, group),
            MapMode::RangeSelect => group_text(builtin::range_select(), user, group),
        });
        match expanded {
            Some(bullets) => out.push_str(&bullets),
            None => {
                out.push_str(line);
                out.push('\n');
            }
        }
    }
    let mine = mapping_lines(mode, keymap);
    if !mine.is_empty() {
        out.push_str("\n## Your mappings\n\n");
        out.push_str(&mine);
    }
    out
}
