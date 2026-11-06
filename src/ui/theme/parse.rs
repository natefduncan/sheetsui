//! The `.theme` file parser. Lenient: a bad line or part of a line is skipped
//! and recorded as a warning, and the rest of the file still loads.
use std::collections::HashMap;

use ratatui::style::{Color, Modifier};

use super::{color::parse_color, Slot, Theme};

/// A problem found while parsing a theme file.
#[derive(Debug, Clone, PartialEq)]
pub struct Warning {
    /// Where it came from: a file name or theme name. `parse` leaves it empty
    /// for the caller, which knows the file, to fill in.
    pub source: String,
    /// 1-based line number, or 0 for a problem with the file as a whole.
    pub line: usize,
    pub msg: String,
}

impl Warning {
    /// Sets the source of every warning in `warnings`.
    pub fn with_source(mut warnings: Vec<Warning>, source: &str) -> Vec<Warning> {
        for w in &mut warnings {
            w.source = source.to_owned();
        }
        warnings
    }
}

impl std::fmt::Display for Warning {
    /// `source:line: msg`, or `source: msg` when there is no line.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.source.is_empty() {
            write!(f, "{}", self.source)?;
            if self.line != 0 {
                write!(f, ":{}", self.line)?;
            }
            write!(f, ": ")?;
        } else if self.line != 0 {
            write!(f, "line {}: ", self.line)?;
        }
        write!(f, "{}", self.msg)
    }
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
        let mut warn = |msg: String| warnings.push(Warning { source: String::new(), line, msg });
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
