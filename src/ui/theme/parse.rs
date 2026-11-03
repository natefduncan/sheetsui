//! The `.theme` file parser. Lenient: a bad line or part of a line is skipped
//! and recorded as a warning, and the rest of the file still loads.
use std::collections::HashMap;

use ratatui::style::{Color, Modifier};

use super::{color::parse_color, Slot, Theme};

/// A problem found while parsing a theme file.
#[derive(Debug, Clone, PartialEq)]
pub struct Warning {
    /// 1-based line number.
    pub line: usize,
    pub msg: String,
}

/// Parses `text` on top of `base`. A slot the text does not set keeps `base`'s style.
pub fn parse(text: &str, base: &Theme) -> (Theme, Vec<Warning>) {
    let mut theme = base.clone();
    let mut warnings = Vec::new();
    let mut palette: HashMap<String, Color> = HashMap::new();
    for (idx, raw) in text.lines().enumerate() {
        let line = idx + 1;
        let trimmed = raw.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let mut warn = |msg: String| warnings.push(Warning { line, msg });
        let tokens: Vec<&str> = trimmed.split_whitespace().collect();
        match tokens[0] {
            "palette" => {
                if tokens.len() != 3 {
                    warn("expected: palette <name> <color>".to_owned());
                } else if let Some(color) = parse_color(tokens[2], &palette) {
                    palette.insert(tokens[1].to_owned(), color);
                } else {
                    warn(format!("unknown color '{}'", tokens[2]));
                }
            }
            "style" => {
                let Some(slot_name) = tokens.get(1) else {
                    warn("expected: style <slot> <key> <value> ...".to_owned());
                    continue;
                };
                let Some(slot) = Slot::from_name(slot_name) else {
                    warn(format!("unknown slot '{}'", slot_name));
                    continue;
                };
                let mut style = theme.get(slot);
                let mut rest = tokens[2..].chunks(2);
                for pair in &mut rest {
                    let [key, value] = pair else {
                        warn(format!("key '{}' has no value", pair[0]));
                        break;
                    };
                    match *key {
                        "fg" | "bg" => match parse_color(value, &palette) {
                            Some(c) if *key == "fg" => style = style.fg(c),
                            Some(c) => style = style.bg(c),
                            None => warn(format!("unknown color '{}'", value)),
                        },
                        "bold" | "italic" | "underline" | "dim" | "reverse" => {
                            let modifier = match *key {
                                "bold" => Modifier::BOLD,
                                "italic" => Modifier::ITALIC,
                                "underline" => Modifier::UNDERLINED,
                                "dim" => Modifier::DIM,
                                _ => Modifier::REVERSED,
                            };
                            match *value {
                                "true" => style = style.add_modifier(modifier),
                                "false" => style = style.remove_modifier(modifier),
                                _ => warn(format!(
                                    "expected true or false for '{}', got '{}'",
                                    key, value
                                )),
                            }
                        }
                        _ => warn(format!("unknown key '{}'", key)),
                    }
                }
                theme.set(slot, style);
            }
            other => warn(format!("unknown line type '{}'", other)),
        }
    }
    (theme, warnings)
}
