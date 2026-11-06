//! Finding a theme: the config directory, the `config` file, user theme files
//! and the built-ins. Every function takes its directories as arguments so
//! tests never touch the environment.
use std::path::{Path, PathBuf};

use super::{parse, parse::Warning, Theme};

/// Built-in themes: name and file contents.
pub const BUILTIN: &[(&str, &str)] = &[
    ("default", include_str!("builtin/default.theme")),
    ("light", include_str!("builtin/light.theme")),
    ("terminal", include_str!("builtin/terminal.theme")),
    ("dracula", include_str!("builtin/dracula.theme")),
    ("papercolor-dark", include_str!("builtin/papercolor-dark.theme")),
];

/// Where a listed theme comes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Source {
    Builtin,
    User,
}

/// `$XDG_CONFIG_HOME/sheetui`, else `$HOME/.config/sheetui`.
pub fn config_dir() -> Option<PathBuf> {
    let base = match std::env::var_os("XDG_CONFIG_HOME") {
        Some(dir) if !dir.is_empty() => PathBuf::from(dir),
        _ => PathBuf::from(std::env::var_os("HOME")?).join(".config"),
    };
    Some(base.join("sheetui"))
}

fn builtin(name: &str) -> Option<&'static str> {
    BUILTIN.iter().find(|(n, _)| *n == name).map(|(_, t)| *t)
}

/// Loads the theme `value` names: a path if it contains `/`, else a user theme
/// in `themes_dir`, else a built-in. `Err` says why it could not be loaded.
pub fn load(value: &str, themes_dir: Option<&Path>) -> Result<(Theme, Vec<Warning>), String> {
    let base = Theme::default();
    let (name, text) = if value.contains('/') {
        let path = Path::new(value);
        let text = std::fs::read_to_string(path)
            .map_err(|e| format!("cannot read theme file {value}: {e}"))?;
        let name = path
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| value.to_owned());
        (name, text)
    } else {
        let user_path = themes_dir.map(|d| d.join(format!("{value}.theme")));
        match user_path.as_deref().filter(|p| p.exists()) {
            Some(path) => {
                let text = std::fs::read_to_string(path)
                    .map_err(|e| format!("cannot read theme file {}: {e}", path.display()))?;
                (value.to_owned(), text)
            }
            None => match builtin(value) {
                Some(text) => (value.to_owned(), text.to_owned()),
                None => return Err(format!("theme {value} not found")),
            },
        }
    };
    let (mut theme, warnings) = parse(&text, &base);
    let warnings = Warning::with_source(warnings, &name);
    theme.name = name;
    Ok((theme, warnings))
}

/// Picks the theme at startup: the flag, then the config value, then `default`.
/// A theme that cannot be loaded gives `default` and a warning on line 0.
pub fn resolve(
    flag: Option<&str>,
    config: Option<&str>,
    themes_dir: Option<&Path>,
) -> (Theme, Vec<Warning>) {
    let Some(value) = flag.or(config) else {
        return (Theme::default(), Vec::new());
    };
    match load(value, themes_dir) {
        Ok(found) => found,
        Err(msg) => (
            Theme::default(),
            vec![Warning {
                source: value.to_owned(),
                line: 0,
                msg,
            }],
        ),
    }
}

/// Built-in and user themes, sorted by name. A user theme shadows a built-in
/// of the same name.
pub fn list(themes_dir: Option<&Path>) -> Vec<(String, Source)> {
    let mut found: Vec<(String, Source)> = BUILTIN
        .iter()
        .map(|(n, _)| ((*n).to_owned(), Source::Builtin))
        .collect();
    let entries = themes_dir.and_then(|d| std::fs::read_dir(d).ok());
    for entry in entries.into_iter().flatten().flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "theme") {
            if let Some(name) = path.file_stem().map(|s| s.to_string_lossy().into_owned()) {
                found.retain(|(n, _)| *n != name);
                found.push((name, Source::User));
            }
        }
    }
    found.sort();
    found
}

#[cfg(test)]
mod test;
