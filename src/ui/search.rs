//! The state of `/` search: the last pattern, compiled once, its
//! direction, and the `incsearch` preview. See `planning/search-design.md`.
use regex::{Regex, RegexBuilder};

use super::{render::viewport::ViewportState, Address};
use crate::book::search::Direction;

impl Direction {
    /// The other way.
    pub fn reverse(self) -> Self {
        match self {
            Direction::Forward => Direction::Backward,
            Direction::Backward => Direction::Forward,
        }
    }

    /// The key that opens a prompt going this way.
    pub fn prompt_key(self) -> &'static str {
        match self {
            Direction::Forward => "/",
            Direction::Backward => "?",
        }
    }
}

/// The last search, kept so `n` repeats it without compiling it again.
#[derive(Debug, Default)]
pub struct SearchState {
    /// The last search, as typed, and its compiled form.
    last: Option<Last>,
    /// The direction of the last search. `n` goes this way, `N` the other.
    pub direction: Direction,
    /// The direction of the open prompt: forward for `/`, backward for `?`.
    /// It becomes the search's direction on Enter, not before, so `Esc`
    /// leaves `n` going the way it went.
    pub prompt: Direction,
    /// Whether the last search's matches are drawn highlighted. A search
    /// turns it on and `:noh` off; the `hlsearch` option can still hide it.
    pub highlight: bool,
    /// While the prompt is open with `incsearch`: what to put back on Esc,
    /// and what the prompt matches so far.
    pub preview: Option<Preview>,
}

/// The `incsearch` preview of an open prompt.
#[derive(Debug)]
pub struct Preview {
    /// The cursor when the prompt opened. Each key searches from here.
    pub start: Address,
    /// The scroll position when the prompt opened.
    pub view: ViewportState,
    /// The last pattern typed that compiled, if the prompt is not empty. A
    /// pattern that does not compile yet, like `a(`, keeps the one before
    /// so the preview does not flicker while typing.
    pub matcher: Option<Regex>,
}

#[derive(Debug)]
struct Last {
    /// The pattern as typed, `\c` and `\C` included.
    text: String,
    /// Whether `regex` was built ignoring case. `:set ic` after a search
    /// changes what `n` matches, as in vim, so this says when to rebuild.
    ignore_case: bool,
    regex: Regex,
}

/// How a pattern treats case: the `ignorecase` and `smartcase` options.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct CaseOptions {
    pub ignorecase: bool,
    pub smartcase: bool,
}

impl SearchState {
    /// The last pattern and its matcher, if there has been a search. The
    /// matcher is rebuilt only if `case` now gives the pattern a different
    /// case rule than when it was compiled.
    pub fn last(&mut self, case: CaseOptions) -> Option<Result<(&str, &Regex), String>> {
        let last = self.last.as_mut()?;
        let (body, ignore_case) = case_rule(&last.text, case);
        if ignore_case != last.ignore_case {
            match build(&body, ignore_case) {
                Ok(regex) => {
                    last.regex = regex;
                    last.ignore_case = ignore_case;
                }
                Err(e) => return Some(Err(e)),
            }
        }
        Some(Ok((last.text.as_str(), &last.regex)))
    }

    /// Compiles a pattern and makes it the last search. An invalid pattern
    /// is an error message, and the last search does not change.
    pub fn set_pattern(&mut self, pattern: &str, case: CaseOptions) -> Result<&Regex, String> {
        let (body, ignore_case) = case_rule(pattern, case);
        let regex = build(&body, ignore_case)?;
        let last = self.last.insert(Last {
            text: pattern.to_owned(),
            ignore_case,
            regex,
        });
        Ok(&last.regex)
    }
}

/// Compiles a pattern without making it the last search, for the preview.
pub fn compile(pattern: &str, case: CaseOptions) -> Result<Regex, String> {
    let (body, ignore_case) = case_rule(pattern, case);
    build(&body, ignore_case)
}

/// The pattern without its `\c` and `\C` atoms, and whether it ignores
/// case. As in vim, `\c` wins over `\C`, and either wins over the options;
/// `smartcase` only matters with `ignorecase`.
fn case_rule(pattern: &str, case: CaseOptions) -> (String, bool) {
    let (body, atom) = strip_case_atoms(pattern);
    let ignore_case = match atom {
        Some(ignore) => ignore,
        None => case.ignorecase && !(case.smartcase && has_uppercase(&body)),
    };
    (body, ignore_case)
}

/// Removes every `\c` and `\C`. Returns what is left and `Some(true)` if
/// there was a `\c`, `Some(false)` if only `\C`. An escaped backslash, as
/// in `\\c`, is a literal backslash and a `c`, not the atom.
fn strip_case_atoms(pattern: &str) -> (String, Option<bool>) {
    let mut body = String::with_capacity(pattern.len());
    let mut lower = false;
    let mut upper = false;
    let mut chars = pattern.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            body.push(c);
            continue;
        }
        match chars.next() {
            Some('c') => lower = true,
            Some('C') => upper = true,
            Some(next) => {
                body.push(c);
                body.push(next);
            }
            None => body.push(c),
        }
    }
    let atom = if lower {
        Some(true)
    } else if upper {
        Some(false)
    } else {
        None
    };
    (body, atom)
}

/// Whether a pattern has an uppercase letter for `smartcase`. Like vim, the
/// letter after a backslash does not count, so `\S` and `\D` are not
/// uppercase; nor does a Unicode class name, as in `\p{Lu}`.
fn has_uppercase(pattern: &str) -> bool {
    let mut chars = pattern.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            // `\pL` names a class in one letter, `\p{Lu}` in braces.
            if let Some('p' | 'P') = chars.next() {
                if chars.next() == Some('{') {
                    for c in chars.by_ref() {
                        if c == '}' {
                            break;
                        }
                    }
                }
            }
            continue;
        }
        if c.is_uppercase() {
            return true;
        }
    }
    false
}

/// Builds a matcher. The `regex` crate's default size limit stays on, so a
/// hostile pattern is an error, not a hang.
fn build(pattern: &str, ignore_case: bool) -> Result<Regex, String> {
    RegexBuilder::new(pattern)
        .case_insensitive(ignore_case)
        .build()
        .map_err(|e| format!("Invalid pattern: {}", short_error(&e)))
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
        let err = build("a(b", false).expect_err("invalid");
        assert_eq!("Invalid pattern: unclosed group", err);
        let err = build("a\\", false).expect_err("invalid");
        assert!(!err.contains('\n'), "{err}");
        assert!(err.starts_with("Invalid pattern: "), "{err}");
    }

    #[test]
    fn a_huge_pattern_is_an_error_not_a_hang() {
        let err = build("\\w{1000}{1000}", false).expect_err("too big");
        assert!(err.starts_with("Invalid pattern: "), "{err}");
    }

    #[test]
    fn an_invalid_pattern_keeps_the_last_search() {
        let mut state = SearchState::default();
        let case = CaseOptions::default();
        state.set_pattern("ok", case).expect("valid");
        state.set_pattern("(", case).expect_err("invalid");
        let last = state.last(case).expect("a search").expect("valid");
        assert_eq!("ok", last.0);
    }

    const OFF: CaseOptions = CaseOptions {
        ignorecase: false,
        smartcase: false,
    };
    const IC: CaseOptions = CaseOptions {
        ignorecase: true,
        smartcase: false,
    };
    const SCS: CaseOptions = CaseOptions {
        ignorecase: false,
        smartcase: true,
    };
    const IC_SCS: CaseOptions = CaseOptions {
        ignorecase: true,
        smartcase: true,
    };

    fn matches(pattern: &str, case: CaseOptions, text: &str) -> bool {
        let mut state = SearchState::default();
        state.set_pattern(pattern, case).expect("valid").is_match(text)
    }

    #[test]
    fn case_is_matched_by_default() {
        assert!(matches("foo", OFF, "foo"));
        assert!(!matches("foo", OFF, "FOO"));
        assert!(!matches("Foo", OFF, "foo"));
    }

    #[test]
    fn ignorecase_ignores_case_both_ways() {
        assert!(matches("foo", IC, "FOO"));
        assert!(matches("FOO", IC, "foo"));
    }

    #[test]
    fn smartcase_alone_does_nothing() {
        assert!(!matches("foo", SCS, "FOO"));
        assert!(!matches("Foo", SCS, "foo"));
        assert!(matches("Foo", SCS, "Foo"));
    }

    #[test]
    fn smartcase_matches_case_when_the_pattern_has_an_uppercase_letter() {
        assert!(matches("foo", IC_SCS, "FOO"));
        assert!(!matches("Foo", IC_SCS, "foo"));
        assert!(!matches("Foo", IC_SCS, "FOO"));
        assert!(matches("Foo", IC_SCS, "Foo"));
        // Non-ASCII uppercase counts too.
        assert!(!matches("Élan", IC_SCS, "élan"));
    }

    #[test]
    fn an_escaped_letter_is_not_uppercase_for_smartcase() {
        // `\S` and `\D` are classes, not letters.
        assert!(matches("a\\Sc", IC_SCS, "AXC"));
        assert!(matches("\\D+", IC_SCS, "ABC"));
        // Nor is a Unicode class name.
        assert!(matches("\\p{Lu}x", IC_SCS, "AX"));
        assert!(matches("\\pLx", IC_SCS, "AX"));
        // A letter after the class still counts.
        assert!(!matches("\\p{L}X", IC_SCS, "Ax"));
    }

    #[test]
    fn lower_c_ignores_case_whatever_the_options() {
        for case in [OFF, IC, SCS, IC_SCS] {
            assert!(matches("\\cfoo", case, "FOO"), "{case:?}");
            assert!(matches("Foo\\c", case, "fOO"), "{case:?}");
            assert!(matches("f\\coo", case, "FOO"), "{case:?}");
        }
    }

    #[test]
    fn upper_c_matches_case_whatever_the_options() {
        for case in [OFF, IC, SCS, IC_SCS] {
            assert!(!matches("\\Cfoo", case, "FOO"), "{case:?}");
            assert!(matches("foo\\C", case, "foo"), "{case:?}");
        }
    }

    #[test]
    fn lower_c_wins_over_upper_c() {
        assert!(matches("\\Cfoo\\c", OFF, "FOO"));
        assert!(matches("\\cfoo\\C", OFF, "FOO"));
    }

    #[test]
    fn the_atoms_are_taken_out_of_the_pattern() {
        assert_eq!(("ab".to_owned(), Some(true)), strip_case_atoms("a\\cb"));
        assert_eq!(("ab".to_owned(), Some(false)), strip_case_atoms("\\Cab"));
        assert_eq!(("a\\.b".to_owned(), None), strip_case_atoms("a\\.b"));
        // An escaped backslash then `c` is literal text, not the atom.
        assert_eq!(("\\\\c".to_owned(), None), strip_case_atoms("\\\\c"));
        assert!(matches("a\\\\c", OFF, "a\\c"));
        assert!(!matches("a\\\\c", OFF, "a\\C"));
        // A trailing backslash is left for the regex crate to reject.
        assert_eq!(("a\\".to_owned(), None), strip_case_atoms("a\\"));
    }

    #[test]
    fn a_pattern_that_is_only_an_atom_matches_everything() {
        assert!(matches("\\c", OFF, "anything"));
    }

    #[test]
    fn the_matcher_is_cached_and_rebuilt_when_the_case_rule_changes() {
        let mut state = SearchState::default();
        state.set_pattern("foo", OFF).expect("valid");
        let (_, re) = state.last(OFF).expect("search").expect("valid");
        assert!(!re.is_match("FOO"));
        let before = re as *const Regex;
        // The same rule keeps the same matcher.
        let (_, re) = state.last(SCS).expect("search").expect("valid");
        assert_eq!(before, re as *const Regex);
        // `:set ic` after the search changes what `n` matches.
        let (text, re) = state.last(IC).expect("search").expect("valid");
        assert_eq!("foo", text);
        assert!(re.is_match("FOO"));
        // `\C` in the pattern keeps it case-sensitive whatever the options.
        state.set_pattern("\\Cfoo", OFF).expect("valid");
        let (text, re) = state.last(IC).expect("search").expect("valid");
        assert_eq!("\\Cfoo", text);
        assert!(!re.is_match("FOO"));
    }
}
