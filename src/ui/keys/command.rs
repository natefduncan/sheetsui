//! The mapping commands: `nnoremap`, `vnoremap`, `nunmap`, `vunmap`.
use crossterm::event::KeyCode;

use super::{
    format_keys,
    map::{parse_rhs, MapCommand, MapMode},
    parse_keys, Key,
};

/// Parses a mapping command. `Ok(None)` means the text is some other command.
pub fn parse(input: &str) -> Result<Option<MapCommand>, String> {
    let (word, rest) = split_word(input.trim_start());
    if let "nmap" | "vmap" = word {
        if !rest.trim().is_empty() {
            return Err(format!(
                "Usage: `{word}`. It takes no arguments, `{}` adds a mapping",
                if word == "nmap" { "nnoremap" } else { "vnoremap" }
            ));
        }
        let mode = if word == "nmap" { MapMode::Navigate } else { MapMode::RangeSelect };
        return Ok(Some(MapCommand::List(mode)));
    }
    let (mode, is_map) = match word {
        "nnoremap" => (MapMode::Navigate, true),
        "vnoremap" => (MapMode::RangeSelect, true),
        "nunmap" => (MapMode::Navigate, false),
        "vunmap" => (MapMode::RangeSelect, false),
        _ => return Ok(None),
    };
    let (lhs_text, rest) = split_word(rest.trim_start());
    if lhs_text.is_empty() {
        return Err(usage(word, is_map));
    }
    let lhs = parse_lhs(word, lhs_text)?;
    if !is_map {
        if !rest.trim().is_empty() {
            return Err(usage(word, is_map));
        }
        return Ok(Some(MapCommand::Unmap { mode, lhs }));
    }
    let rhs_text = rest.trim();
    if rhs_text.is_empty() {
        return Err(usage(word, is_map));
    }
    let rhs = parse_rhs(rhs_text).map_err(|e| format!("{word}: {e}"))?;
    Ok(Some(MapCommand::Map { mode, lhs, rhs }))
}

fn usage(word: &str, is_map: bool) -> String {
    if is_map {
        format!("Usage: `{word} <keys> <rhs>`, for example `{word} <Space>c yj`")
    } else {
        format!("Usage: `{word} <keys>`")
    }
}

/// Parses the left-hand side. A mapping cannot start with a digit, which is the
/// numeric prefix, or with `<Esc>`, which cancels pending keys.
fn parse_lhs(word: &str, text: &str) -> Result<Vec<Key>, String> {
    let keys = parse_keys(text).map_err(|e| format!("{word}: {e}"))?;
    match keys.first() {
        None => Err(format!("{word}: no keys to map")),
        Some(first) if first.as_char().is_some_and(|c| c.is_ascii_digit()) => Err(format!(
            "{word}: {} cannot start a mapping, digits are the numeric prefix",
            format_keys(&keys[..1])
        )),
        Some(first) if *first == Key::plain(KeyCode::Esc) => Err(format!(
            "{word}: <Esc> cannot start a mapping, it cancels pending keys"
        )),
        Some(_) => Ok(keys),
    }
}

fn split_word(text: &str) -> (&str, &str) {
    match text.find(char::is_whitespace) {
        Some(end) => (&text[..end], &text[end..]),
        None => (text, ""),
    }
}
