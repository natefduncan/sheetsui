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
/// after it still run. A `theme <name>` line sets `theme` to the name; the last
/// one wins, as `:theme` does at runtime. The theme itself is loaded later.
pub fn load(
    text: &str,
    source: &str,
    options: &mut Options,
    keymap: &mut KeyMap,
    theme: &mut Option<String>,
) -> Vec<Warning> {
    let mut warnings = Vec::new();
    for (idx, raw) in text.lines().enumerate() {
        let trimmed = raw.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let command_text = trimmed.strip_prefix(':').unwrap_or(trimmed);
        if let Err(msg) = run_line(command_text, options, keymap, theme) {
            warnings.push(Warning {
                source: source.to_owned(),
                line: idx + 1,
                msg,
            });
        }
    }
    warnings
}

fn run_line(
    text: &str,
    options: &mut Options,
    keymap: &mut KeyMap,
    theme: &mut Option<String>,
) -> Result<(), String> {
    if let Some(command) = keys::command::parse(text)? {
        return keymap.apply(command);
    }
    let word = text.split_whitespace().next().unwrap_or(text);
    match cmd::parse(text) {
        Ok(Some(Cmd::Set(args))) => options.set(args).map(|_| ()),
        Ok(Some(Cmd::Theme(Some(name)))) => {
            *theme = Some(name.to_owned());
            Ok(())
        }
        Ok(Some(Cmd::Theme(None))) => Err("theme needs a name".to_owned()),
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
    /// The name from the last `theme` line.
    pub theme: Option<String>,
    pub warnings: Vec<Warning>,
}

/// Loads the file named by `flag`, else `config` from `config_dir`. A missing
/// default file is normal and says nothing; a missing flag file, or any other
/// read error, is a warning.
pub fn load_file(
    flag: Option<&Path>,
    config_dir: Option<&Path>,
    options: &mut Options,
    keymap: &mut KeyMap,
) -> Loaded {
    let path = match (flag, config_dir) {
        (Some(path), _) => path.to_owned(),
        (None, Some(dir)) => dir.join("config"),
        (None, None) => return Loaded::default(),
    };
    let mut theme = None;
    match std::fs::read_to_string(&path) {
        Ok(text) => {
            let warnings = load(&text, &source_name(&path), options, keymap, &mut theme);
            Loaded {
                path: Some(path),
                theme,
                warnings,
            }
        }
        Err(e) if flag.is_none() && e.kind() == std::io::ErrorKind::NotFound => Loaded::default(),
        Err(e) => {
            let msg = format!("cannot read {}: {e}", path.display());
            let source = source_name(&path);
            Loaded {
                path: Some(path),
                theme: None,
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
