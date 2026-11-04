//! `:s`, replacing text in cells.
//!
//! A `:s` command edits the raw contents of a cell, what CellEdit shows,
//! whatever `searchin` is. The new text goes in as if typed, so a formula
//! stays a formula and a number stays a number. It has no UI in it, so it is
//! tested on a book alone. See `planning/search-design.md`.
use anyhow::{anyhow, Result};
use ironcalc::base::types::Cell;
use regex::Regex;

use super::{search::SearchIn, Book};
use crate::ui::Address;

/// A parsed `:s/pat/rep/flags` command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Substitute {
    /// The pattern, with an escaped delimiter made literal. Empty means the
    /// last search pattern, as in vim.
    pub pattern: String,
    /// The replacement in the `regex` crate's form, ready for
    /// [`Regex::replacen`].
    pub replacement: String,
    pub flags: Flags,
}

/// The cells a `:s` command visits: the range in front of the `s`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Scope {
    /// `:s`: the current cell.
    #[default]
    Cell,
    /// `:%s`: every cell with content in the current sheet.
    Sheet,
    /// `:'<,'>s`: the cells of the last range selection.
    Selection,
}

/// The flags after the replacement.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Flags {
    /// `g`: every match in a cell, not only the first.
    pub global: bool,
    /// `e`: no error when nothing matches.
    pub quiet: bool,
}

/// What a substitution did.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Counts {
    pub substitutions: usize,
    pub cells: usize,
}

impl Counts {
    /// The message for the user: `3 substitutions on 1 cell`.
    pub fn message(&self) -> String {
        format!(
            "{} {} on {} {}",
            self.substitutions,
            plural(self.substitutions, "substitution"),
            self.cells,
            plural(self.cells, "cell"),
        )
    }
}

fn plural(n: usize, word: &str) -> String {
    if n == 1 {
        word.to_owned()
    } else {
        format!("{}s", word)
    }
}

/// Whether `c` can be the delimiter of a `:s` command. As in vim, a letter,
/// a digit, a backslash, `"` and `|` cannot, nor can white space.
pub fn is_delimiter(c: char) -> bool {
    !(c.is_alphanumeric() || c.is_whitespace() || matches!(c, '\\' | '"' | '|'))
}

/// Parses what follows the `s` of a `:s` command: `/pat/rep/flags`. The
/// first character is the delimiter. The closing delimiter after the
/// replacement may be left out when there are no flags.
pub fn parse(text: &str) -> Result<Substitute, String> {
    let mut chars = text.chars();
    let delimiter = match chars.next() {
        Some(c) if is_delimiter(c) => c,
        Some(c) => return Err(format!("E146: Regular expressions can't be delimited by {}", c)),
        None => return Err("Usage: s/pattern/replacement/[flags]".to_owned()),
    };
    let rest = chars.as_str();
    let (pattern, rest) = split_at_delimiter(rest, delimiter);
    let pattern = unescape_delimiter(pattern, delimiter);
    let (replacement, flags) = match rest {
        Some(rest) => split_at_delimiter(rest, delimiter),
        None => ("", None),
    };
    Ok(Substitute {
        pattern,
        replacement: replacement_form(replacement)?,
        flags: parse_flags(flags.unwrap_or(""))?,
    })
}

/// Splits `text` at its first delimiter that no backslash escapes. Returns
/// the text before it and, if there was one, the text after it.
fn split_at_delimiter(text: &str, delimiter: char) -> (&str, Option<&str>) {
    let mut escaped = false;
    for (i, c) in text.char_indices() {
        if escaped {
            escaped = false;
        } else if c == '\\' {
            escaped = true;
        } else if c == delimiter {
            return (&text[..i], Some(&text[i + c.len_utf8()..]));
        }
    }
    (text, None)
}

/// Turns `\<delimiter>` in a pattern into a literal delimiter for the regex
/// engine. Every other escape is left for the engine.
fn unescape_delimiter(pattern: &str, delimiter: char) -> String {
    let mut out = String::with_capacity(pattern.len());
    let mut chars = pattern.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some(next) if next == delimiter => {
                out.push_str(&regex::escape(&next.to_string()))
            }
            Some(next) => {
                out.push(c);
                out.push(next);
            }
            None => out.push(c),
        }
    }
    out
}

/// Turns vim's replacement text into the `regex` crate's form: `&` and `\0`
/// are the whole match, `\1` to `\9` a group, `\&`, `\\` and
/// `\<delimiter>` the character itself, and `$` is escaped. A delimiter is
/// never a letter, so any escaped non-letter is the character itself. A cell
/// holds one line, so `\n` and `\r` are errors, as is any other escaped
/// letter.
pub fn replacement_form(text: &str) -> Result<String, String> {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars();
    while let Some(c) = chars.next() {
        match c {
            '$' => out.push_str("$$"),
            '&' => out.push_str("${0}"),
            '\\' => match chars.next() {
                Some(d @ '0'..='9') => {
                    out.push_str("${");
                    out.push(d);
                    out.push('}');
                }
                Some('n' | 'r') => {
                    return Err(
                        "Cannot use \\n or \\r in a replacement: a cell holds one line".to_owned(),
                    )
                }
                Some(other) if other.is_alphabetic() => {
                    return Err(format!("Unsupported escape in replacement: \\{}", other))
                }
                Some('$') => out.push_str("$$"),
                Some(other) => out.push(other),
                // A backslash at the end stands for itself, as in vim.
                None => out.push('\\'),
            },
            other => out.push(other),
        }
    }
    Ok(out)
}

fn parse_flags(text: &str) -> Result<Flags, String> {
    let mut flags = Flags::default();
    for c in text.trim_end().chars() {
        match c {
            'g' => flags.global = true,
            'e' => flags.quiet = true,
            'c' | 'i' | 'I' | 'n' => {
                return Err(format!("The {} flag of :s is not supported yet", c))
            }
            other => return Err(format!("E488: Unknown flag of :s: {}", other)),
        }
    }
    Ok(flags)
}

/// Runs a replacement on one cell's text. Returns the new text and how many
/// matches were replaced: the first, or with `global` all of them.
pub fn substitute_text(
    text: &str,
    matcher: &Regex,
    replacement: &str,
    global: bool,
) -> (String, usize) {
    let limit = if global { 0 } else { 1 };
    let found = matcher.find_iter(text).count();
    if found == 0 {
        return (text.to_owned(), 0);
    }
    let count = if global { found } else { 1 };
    let new = matcher.replacen(text, limit, replacement).into_owned();
    (new, count)
}

impl Book {
    /// Replaces `matcher` with `replacement` (in the `regex` crate's form) in
    /// the raw contents of `cells`, as one undo step. Each changed cell gets
    /// its new text as if typed. A cell the pattern does not match is not
    /// touched, and nothing changes if an array formula would have to. Undo
    /// puts the cursor on the first changed cell.
    pub fn substitute(
        &mut self,
        cells: &[Address],
        matcher: &Regex,
        replacement: &str,
        global: bool,
    ) -> Result<Counts> {
        let mut counts = Counts::default();
        let mut changes = Vec::new();
        for address in cells {
            let text = self.search_text(address, SearchIn::Raw)?;
            let (new, count) = substitute_text(&text, matcher, replacement, global);
            if count == 0 {
                continue;
            }
            if self.is_array_cell(address)? {
                return Err(anyhow!("Cannot substitute in an array formula."));
            }
            counts.substitutions += count;
            counts.cells += 1;
            if new != text {
                changes.push((address, new));
            }
        }
        if changes.is_empty() {
            return Ok(counts);
        }
        self.as_action(|book| {
            book.set_action_location(changes[0].0);
            for (address, new) in changes {
                book.update_cell(address, new)?;
            }
            Ok(())
        })?;
        self.evaluate();
        Ok(counts)
    }

    /// Whether a cell is an array formula or a value one spills.
    fn is_array_cell(&self, address: &Address) -> Result<bool> {
        let worksheet = self
            .model
            .get_model()
            .workbook
            .worksheet(address.sheet)
            .map_err(|e| anyhow!(e))?;
        let cell = worksheet.cell(address.row as i32, address.col as i32);
        Ok(matches!(
            cell,
            Some(Cell::ArrayFormula { .. } | Cell::SpillCell { .. })
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(command: &str, text: &str) -> (String, usize) {
        let sub = parse(command).expect("parse");
        let matcher = Regex::new(&sub.pattern).expect("regex");
        substitute_text(text, &matcher, &sub.replacement, sub.flags.global)
    }

    fn new_text(command: &str, text: &str) -> String {
        run(command, text).0
    }

    #[test]
    fn replaces_the_first_match_only() {
        assert_eq!(run("/a/x/", "banana"), ("bxnana".to_owned(), 1));
    }

    #[test]
    fn g_replaces_every_match() {
        assert_eq!(run("/a/x/g", "banana"), ("bxnxnx".to_owned(), 3));
    }

    #[test]
    fn no_match_leaves_the_text() {
        assert_eq!(run("/z/x/g", "banana"), ("banana".to_owned(), 0));
    }

    #[test]
    fn ampersand_is_the_whole_match() {
        assert_eq!(new_text("/an/[&]/", "banana"), "b[an]ana");
    }

    #[test]
    fn backslash_zero_is_the_whole_match() {
        assert_eq!(new_text("/an/<\\0>/", "banana"), "b<an>ana");
    }

    #[test]
    fn backslash_digits_are_groups() {
        assert_eq!(new_text("/(\\w+) (\\w+)/\\2 \\1/", "hello world"), "world hello");
        assert_eq!(new_text("/(a)(b)(c)(d)(e)(f)(g)(h)(i)/\\9\\1/", "abcdefghi"), "ia");
    }

    #[test]
    fn a_group_followed_by_a_digit_is_still_the_group() {
        assert_eq!(new_text("/(a)/\\10/", "a"), "a0");
    }

    #[test]
    fn escaped_ampersand_is_literal() {
        assert_eq!(new_text("/and/\\&/", "salt and pepper"), "salt & pepper");
    }

    #[test]
    fn escaped_backslash_is_literal() {
        assert_eq!(new_text("/-/\\\\/", "a-b"), "a\\b");
    }

    #[test]
    fn dollar_is_literal() {
        assert_eq!(new_text("/USD /$/", "USD 12"), "$12");
        assert_eq!(new_text("/x/$1/", "x"), "$1");
        assert_eq!(new_text("/x/${0}/", "x"), "${0}");
    }

    #[test]
    fn escaped_delimiter_is_literal_in_both_parts() {
        assert_eq!(new_text("/a\\/b/c\\/d/", "a/b"), "c/d");
    }

    #[test]
    fn any_delimiter_works() {
        assert_eq!(new_text("#a/b#c#", "a/b"), "c");
        assert_eq!(new_text(",a,b,g", "aa"), "bb");
        assert_eq!(new_text("+a+b+", "a"), "b");
    }

    #[test]
    fn an_escaped_regex_metachar_delimiter_is_literal() {
        // With `.` as delimiter, `\.` is a literal dot, not "any character".
        assert_eq!(new_text(".a\\.b.x.", "a.b"), "x");
        assert_eq!(new_text(".a\\.b.x.", "azb"), "azb");
    }

    #[test]
    fn an_escaped_non_letter_is_the_character() {
        assert_eq!(new_text("/a/\\-/", "a"), "-");
    }

    #[test]
    fn the_trailing_delimiter_may_be_left_out() {
        assert_eq!(new_text("/a/x", "a"), "x");
        assert_eq!(new_text("/a", "ba"), "b");
        assert_eq!(new_text("/a/", "ba"), "b");
    }

    #[test]
    fn a_trailing_backslash_is_literal() {
        assert_eq!(new_text("/a/x\\", "a"), "x\\");
    }

    #[test]
    fn newline_and_return_are_errors() {
        for command in ["/a/\\n/", "/a/\\r/"] {
            let err = parse(command).expect_err(command);
            assert!(err.contains("one line"), "{}", err);
        }
    }

    #[test]
    fn other_escaped_letters_are_errors() {
        let err = parse("/a/\\t/").expect_err("\\t");
        assert_eq!(err, "Unsupported escape in replacement: \\t");
    }

    #[test]
    fn unknown_flags_name_the_flag() {
        assert_eq!(parse("/a/b/gz").expect_err("z"), "E488: Unknown flag of :s: z");
    }

    #[test]
    fn c_flag_is_not_supported_yet() {
        let err = parse("/a/b/c").expect_err("c");
        assert!(err.contains("c flag") && err.contains("not supported yet"), "{}", err);
    }

    #[test]
    fn e_flag_is_parsed() {
        let sub = parse("/a/b/ge").expect("parse");
        assert_eq!(sub.flags, Flags { global: true, quiet: true });
    }

    #[test]
    fn bad_delimiters_are_errors() {
        for command in ["a/b/", "\\a\\b\\", "\"a\"b\"", "|a|b|", " a b "] {
            let err = parse(command).expect_err(command);
            assert!(err.starts_with("E146"), "{}", err);
        }
        assert!(parse("").is_err());
    }

    #[test]
    fn an_empty_pattern_is_kept_empty() {
        assert_eq!(parse("//x/").expect("parse").pattern, "");
    }

    #[test]
    fn empty_matches_count_like_vim() {
        assert_eq!(run("/x*/-/g", "ab"), ("-a-b-".to_owned(), 3));
        assert_eq!(run("/^/> /", "ab"), ("> ab".to_owned(), 1));
    }

    #[test]
    fn counts_message_is_plural_aware() {
        let one = Counts { substitutions: 1, cells: 1 };
        assert_eq!(one.message(), "1 substitution on 1 cell");
        let many = Counts { substitutions: 3, cells: 2 };
        assert_eq!(many.message(), "3 substitutions on 2 cells");
    }
}
