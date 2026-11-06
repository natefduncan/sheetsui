//! The `keys` file: a script of mapping commands, run at startup.
use std::path::{Path, PathBuf};

use crate::ui::theme::parse::Warning;

use super::{command, map::KeyMap};

/// Runs the lines of a keys file against `keymap`.
///
/// Blank lines and lines starting with `#` are skipped. A leading `:` is
/// optional. A line that is not a mapping command, does not parse, or cannot
/// be applied is skipped and reported with `source` and its 1-based line number. The lines
/// after it still run.
pub fn load(text: &str, source: &str, keymap: &mut KeyMap) -> Vec<Warning> {
    let mut warnings = Vec::new();
    for (idx, raw) in text.lines().enumerate() {
        let line = idx + 1;
        let trimmed = raw.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let command_text = trimmed.strip_prefix(':').unwrap_or(trimmed);
        let result = match command::parse(command_text) {
            Ok(Some(command)) => keymap.apply(command),
            Ok(None) => Err(
                "only nnoremap, vnoremap, nunmap and vunmap work in a keys file".to_owned(),
            ),
            Err(msg) => Err(msg),
        };
        if let Err(msg) = result {
            warnings.push(Warning {
                source: source.to_owned(),
                line,
                msg,
            });
        }
    }
    warnings
}

/// The name a file goes by in warnings: its file name.
fn source_name(path: &Path) -> String {
    path.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.display().to_string())
}

/// What loading the keys file found.
#[derive(Debug, Default, PartialEq)]
pub struct Loaded {
    /// The file that was read, or tried.
    pub path: Option<PathBuf>,
    pub warnings: Vec<Warning>,
}

/// Loads the keys file into `keymap`: `flag` if given, else `keys` in the
/// config directory. A missing file in the config directory is normal and says
/// nothing. A file named by the flag that cannot be read is a warning, as is
/// any other read error.
pub fn load_file(flag: Option<&Path>, config_dir: Option<&Path>, keymap: &mut KeyMap) -> Loaded {
    let (path, named) = match flag {
        Some(path) => (path.to_path_buf(), true),
        None => match config_dir {
            Some(dir) => (dir.join("keys"), false),
            None => return Loaded::default(),
        },
    };
    match std::fs::read_to_string(&path) {
        Ok(text) => {
            let warnings = load(&text, &source_name(&path), keymap);
            Loaded {
                path: Some(path),
                warnings,
            }
        }
        Err(e) if !named && e.kind() == std::io::ErrorKind::NotFound => Loaded::default(),
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
