//! The state of `/` search: the last pattern, compiled once, and its
//! direction. See `planning/search-design.md`.
use regex::Regex;

use crate::book::search::Direction;

/// The last search, kept so `n` repeats it without compiling it again.
#[derive(Debug, Default)]
pub struct SearchState {
    /// The last pattern searched for, as typed, and its compiled form.
    last: Option<(String, Regex)>,
    /// The direction of the last search.
    pub direction: Direction,
}

impl SearchState {
    /// The last pattern and its matcher, if there has been a search.
    pub fn last(&self) -> Option<(&str, &Regex)> {
        self.last.as_ref().map(|(text, re)| (text.as_str(), re))
    }

    /// Compiles a pattern and makes it the last search. An invalid pattern
    /// is an error message, and the last search does not change.
    pub fn set_pattern(&mut self, pattern: &str) -> Result<&Regex, String> {
        let re = compile(pattern)?;
        Ok(&self.last.insert((pattern.to_owned(), re)).1)
    }
}

/// Compiles a pattern in the `regex` crate's syntax. The crate's default size
/// limit stays on, so a hostile pattern is an error, not a hang.
pub fn compile(pattern: &str) -> Result<Regex, String> {
    Regex::new(pattern).map_err(|e| format!("Invalid pattern: {}", short_error(&e)))
}

/// The `regex` crate's errors span several lines, with the pattern and a
/// caret. The message line has room for one: the last line says what is wrong.
fn short_error(e: &regex::Error) -> String {
    let text = e.to_string();
    let last = text.lines().rev().find(|l| !l.trim().is_empty()).unwrap_or("");
    last.trim().trim_start_matches("error: ").to_owned()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn an_invalid_pattern_is_a_one_line_error() {
        let err = compile("a(b").expect_err("invalid");
        assert_eq!("Invalid pattern: unclosed group", err);
        let err = compile("a\\").expect_err("invalid");
        assert!(!err.contains('\n'), "{err}");
        assert!(err.starts_with("Invalid pattern: "), "{err}");
    }

    #[test]
    fn a_huge_pattern_is_an_error_not_a_hang() {
        let err = compile("\\w{1000}{1000}").expect_err("too big");
        assert!(err.starts_with("Invalid pattern: "), "{err}");
    }

    #[test]
    fn an_invalid_pattern_keeps_the_last_search() {
        let mut state = SearchState::default();
        state.set_pattern("ok").expect("valid");
        state.set_pattern("(").expect_err("invalid");
        assert_eq!(Some("ok"), state.last().map(|(text, _)| text));
    }
}
