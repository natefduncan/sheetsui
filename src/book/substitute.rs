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
    /// The replacement as typed, for the `c` prompt.
    pub typed_replacement: String,
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
    /// `i` (`Some(true)`) or `I` (`Some(false)`): ignore or match case,
    /// whatever `ignorecase` and `smartcase` say. The last one wins, as in
    /// vim. `\c` and `\C` in the pattern still win over both.
    pub ignore_case: Option<bool>,
    /// `n`: count the matches and change nothing.
    pub count: bool,
    /// `c`: ask before each replacement.
    pub confirm: bool,
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
        self.describe("substitution", "substitutions")
    }

    /// The message for the `n` flag, which changes nothing:
    /// `3 matches on 1 cell`.
    pub fn match_message(&self) -> String {
        self.describe("match", "matches")
    }

    fn describe(&self, one: &str, many: &str) -> String {
        format!(
            "{} {} on {} {}",
            self.substitutions,
            plural(self.substitutions, one, many),
            self.cells,
            plural(self.cells, "cell", "cells"),
        )
    }
}

fn plural<'a>(n: usize, one: &'a str, many: &'a str) -> &'a str {
    if n == 1 {
        one
    } else {
        many
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
        typed_replacement: replacement.to_owned(),
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
            'i' => flags.ignore_case = Some(true),
            'I' => flags.ignore_case = Some(false),
            'n' => flags.count = true,
            'c' => flags.confirm = true,
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
    let count = count_matches(text, matcher, global);
    if count == 0 {
        return (text.to_owned(), 0);
    }
    let limit = if global { 0 } else { 1 };
    let new = matcher.replacen(text, limit, replacement).into_owned();
    (new, count)
}

/// How many matches a replacement would replace in `text`: all of them with
/// `global`, else at most one.
fn count_matches(text: &str, matcher: &Regex, global: bool) -> usize {
    if global {
        matcher.find_iter(text).count()
    } else {
        usize::from(matcher.is_match(text))
    }
}

/// The error for a `:s` that would have to change an array formula.
const ARRAY_FORMULA: &str = "Cannot substitute in an array formula.";

/// An answer to the `c` prompt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Answer {
    /// `y`: replace this match and go on.
    Yes,
    /// `n`: skip this match.
    No,
    /// `a`: replace this match and all the rest.
    All,
    /// `l`: replace this match and stop.
    Last,
    /// `q` or Esc: stop.
    Quit,
}

/// The match the `c` prompt asks about: its byte range in the cell's raw
/// text and the text that would replace it.
#[derive(Debug, Clone)]
struct Pending {
    start: usize,
    end: usize,
    with: String,
}

/// A `:s` with the `c` flag: walks the matches in row-major order, one at a
/// time, and replaces the ones the user says yes to. The whole session is
/// one undo step, from [`Confirm::start`] to [`Confirm::finish`], which must
/// be called however the session ends.
#[derive(Debug)]
pub struct Confirm {
    /// The cells with a match, in row-major order.
    cells: Vec<Address>,
    matcher: Regex,
    /// In the `regex` crate's form.
    replacement: String,
    global: bool,
    /// The cell being asked about, as an index in `cells`.
    index: usize,
    /// Where in that cell's text the next match is looked for.
    from: usize,
    /// A match just ended at `from`, so an empty match there is skipped, as
    /// [`Regex::find_iter`] does.
    skip_empty: bool,
    pending: Option<Pending>,
    counts: Counts,
    /// Whether the cell being asked about has been changed yet.
    cell_changed: bool,
    last_changed: Option<Address>,
}

impl Confirm {
    /// Starts a session over `cells`, on its first match. Nothing changes if
    /// an array formula matches. When nothing matches there is nothing to
    /// ask, but [`Confirm::finish`] must still be called.
    pub fn start(
        book: &mut Book,
        cells: &[Address],
        matcher: Regex,
        replacement: String,
        global: bool,
    ) -> Result<Self> {
        let mut matching = Vec::new();
        for address in cells {
            let text = book.search_text(address, SearchIn::Raw)?;
            if matcher.is_match(&text) {
                if book.is_array_cell(address)? {
                    return Err(anyhow!(ARRAY_FORMULA));
                }
                matching.push(address.clone());
            }
        }
        let mut confirm = Confirm {
            cells: matching,
            matcher,
            replacement,
            global,
            index: 0,
            from: 0,
            skip_empty: false,
            pending: None,
            counts: Counts::default(),
            cell_changed: false,
            last_changed: None,
        };
        confirm.seek(book)?;
        book.begin_action();
        Ok(confirm)
    }

    /// The cell of the match being asked about, or `None` when the session
    /// is over.
    pub fn current(&self) -> Option<&Address> {
        self.pending.as_ref().map(|_| &self.cells[self.index])
    }

    /// The regex being replaced, for the highlight.
    pub fn matcher(&self) -> &Regex {
        &self.matcher
    }

    /// Acts on the user's answer about the current match. Returns whether
    /// there is another match to ask about.
    pub fn answer(&mut self, book: &mut Book, answer: Answer) -> Result<bool> {
        if self.pending.is_none() {
            return Ok(false);
        }
        let changed = self.counts.substitutions;
        let result = self.act(book, answer);
        if self.counts.substitutions != changed {
            book.evaluate();
        }
        result?;
        Ok(self.pending.is_some())
    }

    fn act(&mut self, book: &mut Book, answer: Answer) -> Result<()> {
        match answer {
            Answer::Yes => {
                self.replace(book)?;
                self.seek(book)
            }
            Answer::No => {
                let pending = self.pending.take().expect("a pending match");
                self.go_past(pending.start, pending.end, pending.end);
                self.seek(book)
            }
            Answer::All => {
                while self.pending.is_some() {
                    self.replace(book)?;
                    self.seek(book)?;
                }
                Ok(())
            }
            Answer::Last => {
                self.replace(book)?;
                self.pending = None;
                Ok(())
            }
            Answer::Quit => {
                self.pending = None;
                Ok(())
            }
        }
    }

    /// Ends the session and commits its undo step, if anything changed.
    /// Returns the counts and the last cell changed.
    pub fn finish(self, book: &mut Book) -> (Counts, Option<Address>) {
        book.end_action();
        (self.counts, self.last_changed)
    }

    /// Replaces the pending match, and moves past the replacement so that it
    /// is never matched again.
    fn replace(&mut self, book: &mut Book) -> Result<()> {
        let pending = self.pending.take().expect("a pending match");
        let address = self.cells[self.index].clone();
        let text = book.search_text(&address, SearchIn::Raw)?;
        let new = format!(
            "{}{}{}",
            &text[..pending.start],
            pending.with,
            &text[pending.end..]
        );
        if new != text {
            if self.last_changed.is_none() {
                book.set_action_location(&address);
            }
            book.update_cell(&address, new)?;
        }
        self.counts.substitutions += 1;
        if !self.cell_changed {
            self.cell_changed = true;
            self.counts.cells += 1;
        }
        self.last_changed = Some(address);
        let after = pending.start + pending.with.len();
        self.go_past(pending.start, pending.end, after);
        Ok(())
    }

    /// Moves the search on past a match from `start` to `end` that now ends
    /// at `after`. Past an empty match, the next one is a character later.
    /// Without `g` there is no next match in the cell.
    fn go_past(&mut self, start: usize, end: usize, after: usize) {
        if !self.global {
            self.next_cell();
            return;
        }
        self.from = after;
        self.skip_empty = start != end;
        if start == end {
            // One past the next character. `seek` checks it is in the text.
            self.from = after + 1;
            self.skip_empty = false;
        }
    }

    fn next_cell(&mut self) {
        self.index += 1;
        self.from = 0;
        self.skip_empty = false;
        self.cell_changed = false;
    }

    /// Finds the next match, from `from` in the current cell onwards.
    fn seek(&mut self, book: &Book) -> Result<()> {
        while self.index < self.cells.len() {
            let text = book.search_text(&self.cells[self.index], SearchIn::Raw)?;
            if let Some(pending) = self.find(&text) {
                self.pending = Some(pending);
                return Ok(());
            }
            self.next_cell();
        }
        self.pending = None;
        Ok(())
    }

    /// The first match in `text` at `from` or after, if any.
    fn find(&self, text: &str) -> Option<Pending> {
        let mut at = self.from;
        loop {
            // A cell may hold other text than was put in it: `01` is
            // stored as `1`, so the position can be past the end.
            while at <= text.len() && !text.is_char_boundary(at) {
                at += 1;
            }
            if at > text.len() {
                return None;
            }
            let captures = self.matcher.captures_at(text, at)?;
            let found = captures.get(0).expect("group 0 is the match");
            if self.skip_empty && found.is_empty() && found.start() == self.from {
                at = found.start() + 1;
                continue;
            }
            let mut with = String::new();
            captures.expand(&self.replacement, &mut with);
            return Some(Pending {
                start: found.start(),
                end: found.end(),
                with,
            });
        }
    }
}

impl Book {
    /// Counts what [`Book::substitute`] would replace in `cells`, for the
    /// `n` flag. It changes nothing, so it is no undo step, and an array
    /// formula is counted like any other cell.
    pub fn count_substitutions(
        &self,
        cells: &[Address],
        matcher: &Regex,
        global: bool,
    ) -> Result<Counts> {
        let mut counts = Counts::default();
        for address in cells {
            let text = self.search_text(address, SearchIn::Raw)?;
            let count = count_matches(&text, matcher, global);
            if count > 0 {
                counts.substitutions += count;
                counts.cells += 1;
            }
        }
        Ok(counts)
    }

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
                return Err(anyhow!(ARRAY_FORMULA));
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
    fn i_and_big_i_set_the_case_and_the_last_wins() {
        assert_eq!(parse("/a/b/i").expect("i").flags.ignore_case, Some(true));
        assert_eq!(parse("/a/b/I").expect("I").flags.ignore_case, Some(false));
        assert_eq!(parse("/a/b/iI").expect("iI").flags.ignore_case, Some(false));
        assert_eq!(parse("/a/b/Ii").expect("Ii").flags.ignore_case, Some(true));
        assert_eq!(parse("/a/b/").expect("none").flags.ignore_case, None);
    }

    #[test]
    fn n_flag_combines_with_g() {
        let flags = parse("/a/b/gn").expect("gn").flags;
        assert!(flags.count && flags.global);
        let flags = parse("/a/b/n").expect("n").flags;
        assert!(flags.count && !flags.global);
    }

    #[test]
    fn count_matches_follows_g() {
        let matcher = Regex::new("a").expect("regex");
        assert_eq!(count_matches("banana", &matcher, true), 3);
        assert_eq!(count_matches("banana", &matcher, false), 1);
        assert_eq!(count_matches("xyz", &matcher, false), 0);
    }

    #[test]
    fn count_substitutions_changes_nothing() {
        let mut book = Book::from_csv_reader("banana,apple,kiwi\n".as_bytes(), None).expect("csv");
        book.evaluate();
        let cells: Vec<Address> = (1..=3).map(|col| Address { sheet: 0, row: 1, col }).collect();
        let matcher = Regex::new("a").expect("regex");
        let all = book.count_substitutions(&cells, &matcher, true).expect("count");
        assert_eq!(all, Counts { substitutions: 4, cells: 2 });
        let first = book.count_substitutions(&cells, &matcher, false).expect("count");
        assert_eq!(first, Counts { substitutions: 2, cells: 2 });
        assert_eq!(book.get_cell_addr_contents(&cells[0]).expect("contents"), "banana");
    }

    #[test]
    fn match_message_is_plural_aware() {
        let one = Counts { substitutions: 1, cells: 1 };
        assert_eq!(one.match_message(), "1 match on 1 cell");
        let many = Counts { substitutions: 3, cells: 2 };
        assert_eq!(many.match_message(), "3 matches on 2 cells");
    }

    #[test]
    fn c_flag_is_parsed() {
        let sub = parse("/a/b/gc").expect("parse");
        assert!(sub.flags.confirm && sub.flags.global);
        assert_eq!(sub.typed_replacement, "b");
    }

    #[test]
    fn e_flag_is_parsed() {
        let sub = parse("/a/b/ge").expect("parse");
        assert_eq!(sub.flags, Flags { global: true, quiet: true, ..Flags::default() });
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

    /// A book with `texts` in row 1, a session of `command` over them, and
    /// the cells in order.
    fn confirm(texts: &str, command: &str) -> (Book, Confirm, Vec<Address>) {
        let mut book = Book::from_csv_reader(format!("{}\n", texts).as_bytes(), None).expect("csv");
        book.evaluate();
        let n = texts.split(',').count();
        let cells: Vec<Address> = (1..=n).map(|col| Address { sheet: 0, row: 1, col }).collect();
        let sub = parse(command).expect("parse");
        let matcher = Regex::new(&sub.pattern).expect("regex");
        let session = Confirm::start(&mut book, &cells, matcher, sub.replacement, sub.flags.global)
            .expect("start");
        (book, session, cells)
    }

    fn texts(book: &Book, cells: &[Address]) -> Vec<String> {
        cells.iter().map(|a| book.get_cell_addr_contents(a).expect("contents")).collect()
    }

    fn answers(book: &mut Book, session: &mut Confirm, keys: &[Answer]) -> Vec<Option<Address>> {
        let mut at = vec![session.current().cloned()];
        for &key in keys {
            session.answer(book, key).expect("answer");
            at.push(session.current().cloned());
        }
        at
    }

    #[test]
    fn confirm_asks_about_the_first_match_of_each_cell_without_g() {
        let (mut book, mut session, cells) = confirm("aa,b,a", "/a/x/");
        let at = answers(&mut book, &mut session, &[Answer::Yes, Answer::Yes]);
        assert_eq!(at, vec![Some(cells[0].clone()), Some(cells[2].clone()), None]);
        assert_eq!(texts(&book, &cells), ["xa", "b", "x"]);
        let (counts, last) = session.finish(&mut book);
        assert_eq!(counts, Counts { substitutions: 2, cells: 2 });
        assert_eq!(last, Some(cells[2].clone()));
    }

    #[test]
    fn confirm_with_g_asks_about_each_match_in_a_cell() {
        let (mut book, mut session, cells) = confirm("aaa,a", "/a/x/g");
        answers(&mut book, &mut session, &[Answer::Yes, Answer::No, Answer::Yes, Answer::No]);
        assert!(session.current().is_none());
        assert_eq!(texts(&book, &cells), ["xax", "a"]);
        let (counts, last) = session.finish(&mut book);
        assert_eq!(counts, Counts { substitutions: 2, cells: 1 });
        assert_eq!(last, Some(cells[0].clone()));
    }

    #[test]
    fn confirm_does_not_match_the_replacement_again() {
        let (mut book, mut session, cells) = confirm("aa", "/a/aa/g");
        let at = answers(&mut book, &mut session, &[Answer::Yes, Answer::Yes]);
        assert_eq!(at.last(), Some(&None));
        assert_eq!(texts(&book, &cells), ["aaaa"]);
        let (mut book, mut session, cells) = confirm("aa", "/a/aa/g");
        session.answer(&mut book, Answer::All).expect("all");
        assert_eq!(texts(&book, &cells), ["aaaa"]);
    }

    #[test]
    fn confirm_all_replaces_this_and_the_rest() {
        let (mut book, mut session, cells) = confirm("a,a,a", "/a/x/");
        session.answer(&mut book, Answer::No).expect("no");
        assert!(!session.answer(&mut book, Answer::All).expect("all"));
        assert_eq!(texts(&book, &cells), ["a", "x", "x"]);
        assert_eq!(session.finish(&mut book).0, Counts { substitutions: 2, cells: 2 });
    }

    #[test]
    fn confirm_last_and_quit_stop() {
        let (mut book, mut session, cells) = confirm("a,a,a", "/a/x/");
        assert!(!session.answer(&mut book, Answer::Last).expect("last"));
        assert_eq!(texts(&book, &cells), ["x", "a", "a"]);
        let (mut book, mut session, cells) = confirm("a,a,a", "/a/x/");
        assert!(!session.answer(&mut book, Answer::Quit).expect("quit"));
        assert_eq!(texts(&book, &cells), ["a", "a", "a"]);
        assert_eq!(session.finish(&mut book), (Counts::default(), None));
    }

    #[test]
    fn confirm_uses_groups_and_ampersand() {
        let (mut book, mut session, cells) = confirm("ab ab", "/(a)(b)/\\2\\1&/g");
        session.answer(&mut book, Answer::All).expect("all");
        assert_eq!(texts(&book, &cells), ["baab baab"]);
    }

    #[test]
    fn confirm_walks_empty_matches_like_g() {
        let (mut book, mut session, cells) = confirm("ab", "/x*/_/g");
        session.answer(&mut book, Answer::All).expect("all");
        assert_eq!(texts(&book, &cells), ["_a_b_"]);
        let (mut book, mut session, cells) = confirm("ab", "/x*/_/g");
        answers(&mut book, &mut session, &[Answer::No, Answer::Yes, Answer::No]);
        assert!(session.current().is_none());
        assert_eq!(texts(&book, &cells), ["a_b"]);
    }

    #[test]
    fn confirm_with_no_match_asks_nothing() {
        let (mut book, session, _) = confirm("a,b", "/z/x/");
        assert!(session.current().is_none());
        assert_eq!(session.finish(&mut book), (Counts::default(), None));
        assert!(!book.undo().expect("undo"));
    }

    #[test]
    fn confirm_session_is_one_undo_step_going_to_the_first_change() {
        let (mut book, mut session, cells) = confirm("a,a,a", "/a/x/");
        answers(&mut book, &mut session, &[Answer::No, Answer::Yes, Answer::Yes]);
        session.finish(&mut book);
        assert_eq!(texts(&book, &cells), ["a", "x", "x"]);
        assert!(book.undo().expect("undo"));
        assert_eq!(texts(&book, &cells), ["a", "a", "a"]);
        assert_eq!(book.location, cells[1]);
        assert!(!book.undo().expect("undo"));
    }

    #[test]
    fn confirm_refuses_an_array_formula() {
        let mut book = Book::from_csv_reader("3,\n1,\n".as_bytes(), None).expect("csv");
        let array = Address { sheet: 0, row: 1, col: 2 };
        book.update_cell(&array, "=SORT(A1:A2)").expect("set");
        book.evaluate();
        let matcher = Regex::new("SORT").expect("regex");
        let err = Confirm::start(&mut book, &[array], matcher, "SUM".to_owned(), false)
            .expect_err("array");
        assert_eq!(err.to_string(), ARRAY_FORMULA);
    }
}
