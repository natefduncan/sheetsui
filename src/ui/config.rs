//! The `config` file: a script of `:` commands run at startup. Only `set`,
//! `theme` and the mapping commands are allowed. See
//! `planning/config-design.md`.
use std::path::{Path, PathBuf};

use super::{
    cmd::{self, Cmd},
    keys::{self, file::source_name, map::KeyMap},
    options::Options,
    theme::parse::Warning,
};

const ALLOWED: &str = "set, theme, nnoremap, vnoremap, nunmap and vunmap work in a config file";

/// Runs the lines of a config file against `options` and `keymap`.
///
/// Blank lines and lines starting with `#` are skipped. A leading `:` is
/// optional. A line that does not parse, is not allowed, or fails to apply is
/// skipped and reported with `source` and its 1-based line number. The lines
/// after it still run. `theme` lines belong to the theme loader and are
/// skipped here.
pub fn load(
    text: &str,
    source: &str,
    options: &mut Options,
    keymap: &mut KeyMap,
) -> Vec<Warning> {
    let mut warnings = Vec::new();
    for (idx, raw) in text.lines().enumerate() {
        let trimmed = raw.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let command_text = trimmed.strip_prefix(':').unwrap_or(trimmed);
        if let Err(msg) = run_line(command_text, options, keymap) {
            warnings.push(Warning {
                source: source.to_owned(),
                line: idx + 1,
                msg,
            });
        }
    }
    warnings
}

fn run_line(text: &str, options: &mut Options, keymap: &mut KeyMap) -> Result<(), String> {
    if let Some(command) = keys::command::parse(text)? {
        return keymap.apply(command);
    }
    let word = text.split_whitespace().next().unwrap_or(text);
    match cmd::parse(text) {
        Ok(Some(Cmd::Set(args))) => options.set(args).map(|_| ()),
        Ok(Some(Cmd::Theme(_))) => Ok(()),
        Ok(Some(_)) => Err(format!("{word} is not allowed: only {ALLOWED}")),
        Ok(None) => Err(format!("unknown command {word}: only {ALLOWED}")),
        Err(msg) => Err(msg.to_owned()),
    }
}

/// What loading the config file found.
#[derive(Debug, Default, PartialEq)]
pub struct Loaded {
    /// The file that was read, or tried.
    pub path: Option<PathBuf>,
    pub warnings: Vec<Warning>,
}

/// Loads `config` from `config_dir`. A missing file is normal and says
/// nothing; any other read error is a warning.
pub fn load_file(
    config_dir: Option<&Path>,
    options: &mut Options,
    keymap: &mut KeyMap,
) -> Loaded {
    let Some(dir) = config_dir else {
        return Loaded::default();
    };
    let path = dir.join("config");
    match std::fs::read_to_string(&path) {
        Ok(text) => {
            let warnings = load(&text, &source_name(&path), options, keymap);
            Loaded {
                path: Some(path),
                warnings,
            }
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Loaded::default(),
        Err(e) => {
            let msg = format!("cannot read {}: {e}", path.display());
            let source = source_name(&path);
            Loaded {
                path: Some(path),
                warnings: vec![Warning {
                    source,
                    line: 0,
                    msg,
                }],
            }
        }
    }
}

#[cfg(test)]
mod test;
