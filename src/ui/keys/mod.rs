//! Key mappings. A `Key` is one normalized key press. See
//! `planning/keys-design.md`.
use std::fmt;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub mod builtin;
pub mod command;
pub mod map;

#[cfg(test)]
mod test;

/// The modifiers a `Key` can carry. Others (super, hyper, meta) are dropped.
const KEPT: KeyModifiers = KeyModifiers::SHIFT
    .union(KeyModifiers::CONTROL)
    .union(KeyModifiers::ALT);

/// One normalized key press.
///
/// A terminal reports the same key in more than one way, so every `Key` is
/// built through [`Key::new`], which folds the variants into one:
///
/// * Shift on a character is part of the character: `B`, not `<S-b>`.
/// * `Ctrl` on a capital letter stays a different key from `Ctrl` on the
///   lowercase letter. Most terminals cannot send it, but some do (Ctrl-Shift-C
///   copies a cell as it is displayed). The notation cannot write it; the
///   built-in tables add it directly.
/// * `Ctrl-i`, `Ctrl-m` and `Ctrl-[` are `<Tab>`, `<CR>` and `<Esc>`: a
///   terminal sends the same code for each pair.
/// * `BackTab` is `<S-Tab>`.
/// * Super, hyper and meta are dropped.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Key {
    pub code: KeyCode,
    pub mods: KeyModifiers,
}

impl Key {
    /// Builds a key from a code and modifiers, folding the variants described
    /// on the type.
    pub fn new(code: KeyCode, mods: KeyModifiers) -> Self {
        let mut mods = mods & KEPT;
        let mut code = code;
        if code == KeyCode::BackTab {
            code = KeyCode::Tab;
            mods |= KeyModifiers::SHIFT;
        }
        if let KeyCode::Char(c) = code {
            mods.remove(KeyModifiers::SHIFT);
            if mods.contains(KeyModifiers::CONTROL) {
                let folded = match c {
                    'i' => Some(KeyCode::Tab),
                    'm' => Some(KeyCode::Enter),
                    '[' => Some(KeyCode::Esc),
                    _ => None,
                };
                if let Some(folded) = folded {
                    code = folded;
                    mods.remove(KeyModifiers::CONTROL);
                }
            }
        }
        Key { code, mods }
    }

    /// A key with no modifiers.
    pub fn plain(code: KeyCode) -> Self {
        Key::new(code, KeyModifiers::NONE)
    }

    /// A key press from the terminal. The caller filters on the event kind.
    pub fn from_event(event: &KeyEvent) -> Self {
        Key::new(event.code, event.modifiers)
    }

    /// The character, if this is a character key with no Ctrl or Alt.
    pub fn as_char(&self) -> Option<char> {
        match (self.code, self.mods.intersects(KeyModifiers::CONTROL | KeyModifiers::ALT)) {
            (KeyCode::Char(c), false) => Some(c),
            _ => None,
        }
    }
}

impl From<&KeyEvent> for Key {
    fn from(event: &KeyEvent) -> Self {
        Key::from_event(event)
    }
}

/// Names for non-character keys, in canonical spelling. The first name for a
/// code is the one that is printed. Names are matched without regard to case.
const NAMES: &[(&str, KeyCode)] = &[
    ("CR", KeyCode::Enter),
    ("Enter", KeyCode::Enter),
    ("Return", KeyCode::Enter),
    ("Esc", KeyCode::Esc),
    ("Escape", KeyCode::Esc),
    ("Tab", KeyCode::Tab),
    ("Space", KeyCode::Char(' ')),
    ("BS", KeyCode::Backspace),
    ("Backspace", KeyCode::Backspace),
    ("Del", KeyCode::Delete),
    ("Delete", KeyCode::Delete),
    ("Insert", KeyCode::Insert),
    ("Up", KeyCode::Up),
    ("Down", KeyCode::Down),
    ("Left", KeyCode::Left),
    ("Right", KeyCode::Right),
    ("Home", KeyCode::Home),
    ("End", KeyCode::End),
    ("PageUp", KeyCode::PageUp),
    ("PageDown", KeyCode::PageDown),
    ("lt", KeyCode::Char('<')),
];

/// Why a key notation did not parse.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyError(pub String);

impl fmt::Display for KeyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for KeyError {}

fn err<T>(msg: impl Into<String>) -> Result<T, KeyError> {
    Err(KeyError(msg.into()))
}

/// Parses a sequence of keys: `gg`, `<Space>c`, `<C-s>`.
///
/// Every character outside `<…>` is one key, including a space. A `<` always
/// starts a bracketed name: write `<lt>` for a literal `<`.
pub fn parse_keys(text: &str) -> Result<Vec<Key>, KeyError> {
    let mut keys = Vec::new();
    let mut rest = text;
    while !rest.is_empty() {
        let (key, used) = parse_key(rest)?;
        keys.push(key);
        rest = &rest[used..];
    }
    Ok(keys)
}

/// Parses the first key of `text`. Returns the key and how many bytes of
/// `text` it used.
pub fn parse_key(text: &str) -> Result<(Key, usize), KeyError> {
    let mut chars = text.chars();
    match chars.next() {
        None => err("expected a key, found nothing"),
        Some('<') => {
            let Some(end) = text.find('>') else {
                return err("'<' starts a key name but there is no '>'; write <lt> for a literal '<'");
            };
            Ok((parse_bracketed(&text[1..end])?, end + 1))
        }
        Some(c) => Ok((Key::plain(KeyCode::Char(c)), c.len_utf8())),
    }
}

/// Parses what is inside `<…>`: modifiers, then a name or one character.
fn parse_bracketed(inner: &str) -> Result<Key, KeyError> {
    if inner.is_empty() {
        return err("'<>' is not a key name; write <lt> for a literal '<'");
    }
    let mut rest = inner;
    let mut mods = KeyModifiers::NONE;
    // A modifier is one letter and a dash, and something must follow it.
    while rest.len() > 2 && rest.as_bytes()[1] == b'-' {
        let m = match rest.as_bytes()[0].to_ascii_uppercase() {
            b'C' => KeyModifiers::CONTROL,
            b'A' | b'M' => KeyModifiers::ALT,
            b'S' => KeyModifiers::SHIFT,
            _ => break,
        };
        if mods.contains(m) {
            return err(format!("<{inner}> repeats a modifier"));
        }
        mods |= m;
        rest = &rest[2..];
    }
    let (code, is_char) = if let Some(code) = named(rest) {
        (code, matches!(code, KeyCode::Char(_)))
    } else if let Some(n) = function_key(rest) {
        (KeyCode::F(n), false)
    } else {
        let mut cs = rest.chars();
        match (cs.next(), cs.next()) {
            (Some(c), None) => (KeyCode::Char(c), true),
            _ => return err(format!("<{inner}> is not a key name")),
        }
    };
    if mods.contains(KeyModifiers::SHIFT) {
        if !is_char {
            // Shift on a named key such as Tab is a real key.
        } else if mods.contains(KeyModifiers::CONTROL) {
            return err(format!(
                "<{inner}> cannot be mapped: a terminal cannot report Ctrl and Shift on a letter"
            ));
        } else {
            return err(format!(
                "<{inner}> is not needed: Shift is part of the character, write it as the capital letter"
            ));
        }
    }
    // As in vim, `<C-X>` is Ctrl and the letter, whatever its case.
    let code = match code {
        KeyCode::Char(c) if mods.contains(KeyModifiers::CONTROL) => {
            KeyCode::Char(c.to_ascii_lowercase())
        }
        code => code,
    };
    Ok(Key::new(code, mods))
}

fn named(name: &str) -> Option<KeyCode> {
    NAMES
        .iter()
        .find(|(n, _)| n.eq_ignore_ascii_case(name))
        .map(|(_, code)| *code)
}

fn function_key(name: &str) -> Option<u8> {
    let digits = name.strip_prefix(['F', 'f'])?;
    match digits.parse::<u8>() {
        Ok(n) if (1..=12).contains(&n) && !digits.starts_with('0') => Some(n),
        _ => None,
    }
}

/// Prints a key in its canonical notation.
impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ctrl = self.mods.contains(KeyModifiers::CONTROL);
        let alt = self.mods.contains(KeyModifiers::ALT);
        let shift = self.mods.contains(KeyModifiers::SHIFT);
        // The text for the key itself, and whether it needs brackets to be
        // read back.
        let (text, needs_brackets) = match self.code {
            KeyCode::Char(c) if c != '<' && c != ' ' => (c.to_string(), false),
            KeyCode::F(n) => (format!("F{n}"), true),
            code => match NAMES.iter().find(|(_, c)| *c == code) {
                Some((name, _)) => ((*name).to_owned(), true),
                None => (format!("{code:?}"), true),
            },
        };
        if !(needs_brackets || ctrl || alt || shift) {
            return f.write_str(&text);
        }
        // Ctrl on a capital letter is Ctrl and Shift. It reads as `<C-S-c>`,
        // which the parser rejects, so it is never mistaken for `<C-c>`.
        let ctrl_capital = matches!(self.code, KeyCode::Char(c) if ctrl && c.is_ascii_uppercase());
        f.write_str("<")?;
        if ctrl {
            f.write_str("C-")?;
        }
        if alt {
            f.write_str("A-")?;
        }
        if shift || ctrl_capital {
            f.write_str("S-")?;
        }
        if ctrl_capital {
            f.write_str(&text.to_ascii_lowercase())?;
        } else {
            f.write_str(&text)?;
        }
        f.write_str(">")
    }
}

/// Prints a sequence of keys in canonical notation.
pub fn format_keys(keys: &[Key]) -> String {
    keys.iter().map(|k| k.to_string()).collect()
}
