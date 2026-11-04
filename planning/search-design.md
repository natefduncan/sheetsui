# Design: Search and Replace

Status: proposed 2026-11-04 — decisions D1–D10.

## User

A person with a sheet bigger than one screen who needs to find a value, jump
between cells that look alike, or change text in many cells at once. They know
vim, and expect `/`, `?`, `n`, `N`, `*`, `#`, and `:s` to work the way they do
there.

## Goal

Excel parity for `Ctrl-F` and `Ctrl-H`, with vim's interface. Where vim and
Excel differ, vim wins (D3), unless a sheet has no equivalent for a vim idea
(then see D5 and D7).

## Flow

### Search

1. In Navigate mode, the user types `/`. A prompt opens at the bottom, as for
   `:`. They type a pattern and press `Enter`.
2. The cursor goes to the first match after the cursor, in row-major order.
3. `n` goes to the next match in the same direction. `N` goes the other way.
4. `?` is the same as `/` but starts backward. `n` then keeps going backward.
5. `*` searches forward for cells like the current one. `#` searches backward
   (D10).
6. `/` followed by `Enter` with no pattern repeats the last pattern.
7. A count works: `3n` goes to the third next match.
8. While the user types the pattern, the cursor previews the first match
   and visible matches are highlighted. `Esc` puts the cursor back where it
   started and clears the preview (D6).
9. A search that passes the last cell wraps to the first and says
   `search hit BOTTOM, continuing at TOP`. Backward says `TOP ... BOTTOM`
   (D5).
10. No match shows `E486: Pattern not found: <pat>` and leaves the cursor.

### Replace

1. `:s/pat/rep/[flags]` replaces in the current cell.
2. `:%s/pat/rep/[flags]` replaces in every cell of the current sheet.
3. `:'<,'>s/pat/rep/[flags]` replaces in the last range selection. `:` typed
   in Range Select mode pre-fills `'<,'>` (D7).
4. The command reports `N substitutions on M cells`.
5. The whole command is one undo step (see Undo, below).

### Keys and commands

| Key or command | Mode | What it does |
|----------------|------|--------------|
| `/` | Navigate | Search forward. |
| `?` | Navigate | Search backward. |
| `n`, `N` | Navigate | Next or previous match, in the direction of the last search. |
| `*`, `#` | Navigate | Search forward or backward for cells like this one. |
| `:s/pat/rep/f` | Command | Replace in the current cell. |
| `:%s/pat/rep/f` | Command | Replace in the sheet. |
| `:'<,'>s/pat/rep/f` | Command | Replace in the last range selection. |
| `:noh`, `:nohlsearch` | Command | Clear the match highlight until the next search. |
| `:set <opt>`, `:set no<opt>`, `:set <opt>=<val>` | Command | Change an option. See Options. |

`/`, `?`, `n`, `N`, `*`, `#` are new bindings in Navigate mode. None are in
use today. They go in `builtin.rs` like the other keys, so `nnoremap` and
`nunmap` work on them. In Range Select mode `/` and `?` are not bound.

## Matching (D3, D4)

- The pattern is a regex in the Rust `regex` crate's syntax. This is close to
  vim's `\v` ("very magic") mode, and not vim's default `magic` mode. The docs
  list the differences. Examples: use `(a|b)`, not `\(a\|b\)`; use `a+`, not
  `a\+`; use `\.` for a literal dot.
- Matching is case-sensitive unless `ignorecase` is set (or `smartcase`, which
  turns it back on when the pattern has an uppercase letter). `\c` and `\C`
  inside a pattern force case-insensitive and case-sensitive. They are removed
  from the pattern before it goes to the regex engine.
- An invalid pattern shows the regex error and does nothing. It must never
  panic.

### What text is matched (D1, D2)

A cell has two strings: the raw contents (`=A1*2`, `0.5`) and the displayed
value (`$12.00`, `50%`, `2026-11-04`). Which one `/`, `?`, `n`, `N`, `*`, `#`
look at is set by the `searchin` option:

- `searchin=value` (default): the displayed value.
- `searchin=raw`: the raw contents, so formulas can be searched (`/SUM\(`).

`:s` always edits raw contents (D8).

### Order and scope (D5)

- Only the current sheet. Search does not cross sheets.
- Row-major: left to right across a row, then the next row down. Only cells
  that have content are visited. The visited area is the sheet's used range.
- `wrapscan` is on by default. With `nowrapscan` a search stops at the end with
  `E385: Search hit BOTTOM without match for: <pat>`.
- A range selection does not limit a search. It limits `:'<,'>s` (D7).

## Highlighting (D6)

- With `hlsearch` (default on), every visible cell that matches is drawn with
  the theme style `search`. The match under the cursor uses `search.current`.
  `:noh` clears it until the next search.
- With `incsearch` (default on), the prompt previews as the user types: the
  cursor jumps to the first match from where the search began, and
  highlighting updates per key. `Enter` keeps the position. `Esc` restores
  the position and the highlight.
- Matches highlight the whole cell, not the matched text. A cell is the
  smallest unit that the sheet draws.
- Two new theme slots: `search` and `search.current`. Built-in themes get a
  value for each. A user theme that does not set them falls back to the
  built-in value, like other slots.

## Replace (D7, D8, D9)

### Scope

| Form | Cells visited |
|------|---------------|
| `:s` | The current cell. |
| `:%s` | Every cell with content in the current sheet. |
| `:'<,'>s` | The cells of the last range selection. |

The `'<,'>` range uses the last range selection. If there is none, it is an
error: `E20: Mark not set`.

### Text edited

`:s` edits raw contents, whatever `searchin` is. `:s/SUM/AVERAGE/` edits
formula text. A number cell is matched on its stored text (`12`, not `$12.00`).
The new text goes into the cell the same way as text typed in CellEdit
mode, so `=A1+1` stays a formula, `12` stays a number, and a cell format or
style does not change. A cell that the pattern does not match is not touched.

Search and replace can therefore disagree: `/12\.00` finds a cell showing
`$12.00`, but `:s/12\.00/x/` does not match its stored `12`. The docs say so.

### Replacement text

Written like vim:

| Text | Meaning |
|------|---------|
| `&` | The whole match. |
| `\1` to `\9` | A capture group. |
| `\&` | A literal `&`. |
| `\\` | A literal backslash. |
| `\/` | A literal delimiter (with `/`). |

Any single character can be the delimiter, as in vim: `:s#a/b#c#`.
There is no `\n` or `\r`, because a cell holds a single line. They are an
error. sheetui turns this into the regex crate's replacement form, with `$`
characters escaped, before it calls the engine.

### Flags

| Flag | Meaning |
|------|---------|
| `g` | Every match in the cell. Without it, only the first match in each cell. |
| `i` | Ignore case. |
| `I` | Match case. |
| `n` | Count the matches and change nothing. Reports `N matches on M cells`. |
| `e` | No error when there is no match. |
| `c` | Confirm each match. See below. |

Flags combine: `:%s/a/b/gc`. An unknown flag is an error that names the flag.

### Confirming with `c`

1. For each match, in row-major order, the cursor goes to its cell, the cell
   is highlighted with `search.current`, and the bottom line shows
   `replace with <rep> (y/n/a/q/l)?`. This is a new mode, `Modality::Confirm`.
2. Keys: `y` replace this match and go on. `n` skip it. `a` replace this and
   all the rest. `l` replace this and stop. `q` or `Esc` stop.
3. With `g`, each match in a cell is asked about in turn. Without it, only the
   first match in each cell.
4. After a `y`, matching goes on after the replaced text. The replacement is
   never matched again, so a replacement that contains the pattern does not
   loop.
5. The prompt's cell is shown even when it is off screen: the viewport scrolls
   to it.
6. When the prompt ends, the cursor goes to the last cell that was changed. If
   nothing was changed, it goes back where it started.

## Undo

A `:s` command is one undo step, in line with the undo design (D6 there). That
includes `:%s` over thousands of cells, and a `c` session however many matches
were confirmed. The step is committed when the command or the confirm prompt
ends. `u` then reverts it all and puts the cursor on the first changed cell.
A search changes no cell, so it is not a step. A `:s` that changes no cell is
not a step either.

## Options (D4, D10)

A new small settings table, set with `:set`, lives for the session.

| Option | Type | Default | Meaning |
|--------|------|---------|---------|
| `searchin` | `value` or `raw` | `value` | Text that search looks at. |
| `wrapscan`, `ws` | bool | on | Wrap at the end of the sheet. |
| `ignorecase`, `ic` | bool | off | Ignore case in patterns. |
| `smartcase`, `scs` | bool | off | With `ignorecase`, match case if the pattern has an uppercase letter. |
| `hlsearch`, `hls` | bool | on | Highlight matches. |
| `incsearch`, `is` | bool | on | Preview while typing. |

Forms: `:set ic`, `:set noic`, `:set ic!` (toggle), `:set searchin=raw`,
`:set searchin?` (show). An unknown option is `E518: Unknown option: <name>`.

Options are not saved. A later change can read them from the keys file or a new
config file. That is out of scope here.

## History (D10)

`/` and `?` have their own history: Up and Down browse earlier patterns that
start with what was typed, as in command mode. It reuses `history.rs`
with a separate store and file: `$XDG_STATE_HOME/sheetui/search-history`
(`~/.local/state/sheetui/search-history` by default), 100 entries. `:s`
commands go in the command history as they do now.

## `*` and `#` (D10)

`*` and `#` search for cells like the current one: the pattern is the cell's
text (displayed or raw, by `searchin`), escaped for the regex engine, and
anchored as `^text$`. So `*` on `Paid` skips `Unpaid`. On an empty cell they do
nothing and say `E348: No string under cursor`. The pattern becomes the last
search pattern, so `n` and `N` continue it. It is added to the search history.

## Technical approach

### Where the code goes

- `src/book/search.rs` (new): the search itself. A function that takes the
  book, the sheet, a compiled matcher, a start address, a direction and
  `wrapscan`, and returns the next match address. It has no UI in it, so it can
  be tested on a book alone, like `sort.rs`. A second function gives all the
  matches in a rectangle, for highlighting.
- `src/book/substitute.rs` (new): `:s`. It parses the command text into a
  scope, pattern, replacement and flags. A pure function runs the
  replacement on one cell's text and returns the new text and the match
  count. A book method applies it to a set of cells inside one undo group.
- `src/ui/search.rs` (new): the search state: last pattern, direction,
  current match, `incsearch` start position, highlight on or off. The
  Search prompt is a text editor like the command line, so it reuses the
  command-line editing code in `cmd.rs` where it can.
- `src/ui/options.rs` (new): the settings table and `:set`.
- `src/ui/mod.rs`: new `Modality::Search` (the `/` and `?` prompt) and
  `Modality::Confirm` (`c` flag prompt).
- `src/ui/keys/builtin.rs`: six new `NavHandler` variants (`SearchForward`,
  `SearchBackward`, `SearchNext`, `SearchPrev`, `SearchWordForward`,
  `SearchWordBackward`), grouped in a new `"search"` help section.
- `src/ui/render/`: draws highlighted cells using the new theme styles.
- `src/ui/theme/`: new slots `search` and `search.current`, in the built-in
  themes and in `docs/themes.md`.

### Compiling the pattern

Compile once per search, not per cell. Handle `\c` and `\C`, then `ignorecase`
and `smartcase`, then build the `regex::Regex` with the case flag. Cache the
compiled form with the last pattern so `n` and highlighting do not
recompile. Regexes have a size limit by default; keep it so a hostile pattern
cannot hang the UI.

### Reading a cell's text

- Displayed value: the same formatted string that rendering uses. Check that
  there is a single function for it (copy-formatted `Y` already reads it) and
  reuse it.
- Raw contents: the same string that CellEdit mode shows when the user presses
  `e`.

### Highlighting cost

`hlsearch` runs the matcher over visible cells only, each frame, with the
compiled regex. That is a screenful of cells. It must not scan the whole
sheet per frame. The cursor-preview and `n` do scan the sheet, once per key.

### Substitution and undo

Use the existing grouping from the undo design: one input event is one action,
and the book counts ironcalc steps inside it. `:s` writes each changed cell with
the same call as a CellEdit commit, so formulas re-parse and recalculation
happens the same way. For a `c` session, the group opens at the first `y` and
closes when the prompt ends.

### Errors

Use vim's messages where they exist (`E486`, `E385`, `E348`, `E20`,
`E518`) so a vim user recognizes them. Everything else is a short plain
message.

## Cases

| Case | Behavior |
|------|----------|
| Empty sheet | `E486: Pattern not found`. |
| Pattern matches only the cell under the cursor | `n` stays there and wraps with the message. |
| `/` with an empty pattern and no earlier search | `E35: No previous regular expression`. |
| Merged cells | The merged cell is one cell, at its top-left address. |
| Array formula | Searched as one cell. `:s` on it is an error: `cannot substitute in an array formula`, as in sort. |
| `:s` produces an invalid formula | The cell shows the formula error, as if typed. Undo reverts it. |
| `:s` on a cell that has a formula and `searchin=value` | Still edits the raw formula (D8). |
| `*` when `searchin=value` on a number cell | Pattern is `^\$12\.00$` for the shown `$12.00`. |
| Pattern with a trailing backslash | Regex error shown. |
| Very large sheet | Search is a single pass, with no allocation per cell beyond the cell's text. |
| Search while a range is selected | Not available: `/` is not bound in Range Select. |

## Tests

- `src/book/search.rs`: forward and backward order, wrap, `nowrapscan`, merged
  cells, empty sheet, `searchin` both ways, `ignorecase`, `smartcase`, `\c`
  and `\C`, invalid pattern.
- `src/book/substitute.rs`: every flag; `&`, `\1`, `\&`, `\\`; other delimiters;
  formulas stay formulas; numbers stay numbers; styles unchanged; `%` and
  `'<,'>` scopes; array formula refused; one undo step; no-op is not a step.
- `c` flag, UI level: each of `y n a l q Esc`, with and without `g`, the
  replaced text is not re-matched, the cursor ends where the design says, one
  undo step.
- UI: `/`, `?`, `n`, `N`, counts, `*`, `#`, `incsearch` preview and `Esc`
  restore, `:noh`, search history, `:set` forms, key remapping of the new keys.
- Render: highlighted cells, the current-match style, theme fallback.
- Help: `{{keys:search}}` section generated from the live tables.

## Alternatives considered

- **Literal substring, no regex.** Easier, but it is not vim, and `:s` without
  groups is weak. Rejected (D3).
- **Translate vim `magic` syntax to the regex crate.** The most faithful, with a
  translator that is a project in itself. Possible later behind an option.
- **Search both displayed and raw at once.** Rejected: searching `5` would
  match `=A5+1`. Chosen instead: pick one with `searchin` (D1, D4).
- **`g/` for raw search.** Rejected: vim's `?` is backward search, and the
  `g` prefix is used for other things (D2).
- **Column-major order, or all sheets.** Row-major on one sheet is Excel's
  default. Others can be options later (D5).
- **Whole-sheet-only replace**, like Excel's "Replace All". Too easy to run by
  mistake; vim's scopes are safer (D7).
- **`:s` that follows `searchin`.** It would turn formulas into their results
  and lose formats. Rejected (D8).

## Non-goals

- Searching all sheets, or only a selection (`/` in Range Select).
- Column-major order.
- Vim's `magic` regex syntax, `\<` `\>` word atoms, and lookbehind with `\@<=`.
- `\n` or `\r` in replacements, and `\=` expression replacement.
- `:g`, `:v` global commands, `:&&`, `:~`, and `&` as a repeat command.
- Saving options between runs.
- Highlighting the matched text inside a cell.

## Decisions

- **D1 — Search both forms of text.** Displayed value and raw contents are
  both searchable.
- **D2 — Keys match vim.** `/` forward, `?` backward, `n` and `N` repeat and
  reverse. Rejected: `g/` for raw search.
- **D3 — Regex, Rust `regex` crate syntax.** `\c` and `\C` supported. Case
  sensitive by default. Differences from vim's syntax are documented.
- **D4 — `:set searchin=value|raw` chooses the text.** Default `value`. A
  per-pattern atom can come later.
- **D5 — Current sheet, row-major, `wrapscan` on.** A selection does not limit
  search. No cross-sheet search.
- **D6 — nvim defaults: `hlsearch` and `incsearch` on.** Theme slots `search`
  and `search.current`. `:noh` clears.
- **D7 — `:s` scopes: cell, `%` sheet, `'<,'>` last selection.** `:` in
  Range Select pre-fills `'<,'>`.
- **D8 — `:s` always edits raw contents** and re-enters the result as if
  typed.
- **D9 — Flags `g i I n e c`, replacement `& \1..\9 \& \\`.** `c` confirms with
  `y n a l q`, in a new `Confirm` mode, as one undo step.
- **D10 — `*` and `#`, search history, and the `:set` options are in v1.** `*`
  anchors the whole cell text. History has its own file.

## Docs to update

- `docs/navigation.md` — `/`, `?`, `n`, `N`, `*`, `#`, in a new `{{keys:search}}`
  section.
- `docs/command.md` — `:s`, `:%s`, `:'<,'>s`, flags, `:noh`, `:set`, and the
  options table.
- `docs/themes.md` — slots `search` and `search.current`.
- `docs/keys.md` — new built-in keys and the `search` help group.
- New `docs/search.md` — regex differences from vim, the value and raw
  distinction, and why `:s` can miss what `/` finds.
- `README.md` — none.
