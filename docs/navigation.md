# Navigation Mode

The interface will start out in navigation mode. You can navigate around the
table and between the sheets using the following keybinds. Keys are written in
vim notation: `<C-n>` is Ctrl-n, `<A-h>` is Alt-h, `<CR>` is Enter, and a
capital letter is the shifted key.

## Cell Navigation

{{keys:cell-navigation}}

## Sheet Navigation

{{keys:sheet-navigation}}

Sheet navigation moving will loop around when you reach the ends.

## Numeric prefixes

You can prefix each of the keybinds above with a numeric prefix to do them that
many times. So typing `123h` will move to the left 123 times. Hitting `Esc`
will clear the numeric prefix if you want to cancel it.

**Modifying the Sheet or Cells**

{{keys:modify}}

## Undo and Redo

{{keys:undo}}

A numeric prefix undoes or redoes that many changes: `3u` undoes three.

Everything that one key or one command changes undoes in one step: a cell
edit, a paste, a sort, `:insert-rows 5`, or `5 Ctrl-l`. `o` and `O` are two
steps: the new row, then the edit you type in it. Moving the cursor, selecting,
switching sheets, and saving are not changes.

When you undo back to the point where you last saved, the sheet counts as not
changed, so `q` quits with no prompt. The history starts empty each time you
open a file, and it is not saved when you quit.

## Search

{{keys:search}}

For an overview of search and replace, see [Search and Replace](./search.md).

Type `/`, then a pattern, then `Enter`. The cursor goes to the next cell after
it whose displayed value matches the pattern: left to right across a row, then
the next row down. `?` does the same backward: right to left, then the row
above. Only the current sheet is searched, and only cells that have something
in them.

`n` goes to the next match in the direction of the last search, so after `?`
it keeps going backward. `N` goes the other way, and does not change which
way `n` goes. Both take a count: `3n` goes to the third match.

`*` searches forward for cells like the one under the cursor, and `#`
searches backward. The pattern is the cell's whole text, with every character
that means something in a pattern escaped, so `*` on `Paid` skips `Unpaid`,
and `*` on `a.b` skips `axb`. The text is the shown value, or the raw contents
with `:set searchin=raw`: `*` on a cell showing `$12.00` finds the other cells
that show `$12.00`, and with `searchin=raw` the cells that store `12`, or the
same formula. `ignorecase` and `smartcase` apply as for `/`. The pattern is the
last search afterwards, so `n` and `N` go on with it. Both take a count. On an
empty cell they say `E348: No string under cursor`.

In the `/` and `?` prompts, `Up` shows the previous pattern and `Down` the
next one. If you type some text first, `Up` and `Down` show only the patterns
that start with that text. `Down` past the newest pattern gives back the text
you typed. With `incsearch` the cursor follows the pattern shown. Every
pattern that you search for with `Enter` goes into the history, also one that
does not parse, and so does the pattern that `*` or `#` builds. A search that
you cancel with `Esc` does not. `/` and `?` share one history, apart from the
command history: the last 100 patterns are kept in
`$XDG_STATE_HOME/sheetui/search-history` (by default
`~/.local/state/sheetui/search-history`), so they are still there when you
start sheetui again. If that file cannot be read or written, the history is
kept until you quit.

`/` or `?` with no pattern, then `Enter`, searches for the last pattern again,
in the new direction. With no last pattern it says
`E35: No previous regular expression`, as do `n` and `N`.

A search that passes the last cell goes on from the first one and says
`search hit BOTTOM, continuing at TOP`; backward, it says
`search hit TOP, continuing at BOTTOM`. With `:set nowrapscan` a search stops
at the end of the sheet instead and says
`E385: Search hit BOTTOM without match for: <pattern>` (or `TOP` going
backward). See [Options](./command.md#options). A pattern with no match says
`E486: Pattern not found` and leaves the cursor where it is. `Esc` closes the
prompt without searching.

While you type, the cursor jumps to the first match of what is in the prompt so
far, counted from where you opened it, and the matches are highlighted. With
nothing to match yet, it stays where it started. `Enter` keeps the cell the
preview found. `Esc` puts the cursor, the scroll position, and the highlight
back as they were. A pattern that is not valid yet, like `a(` on the way to
`a(b|c)`, says nothing and leaves the preview as it was; only `Enter` reports
the error. `:set noincsearch` turns the preview off, so the cursor moves only
on `Enter`. With `nohlsearch` the preview moves the cursor but highlights
nothing. See [Options](./command.md#options).

The pattern is a regular expression in the syntax of Rust's `regex` crate. It
is close to vim's "very magic" `\v` mode: write `(a|b)`, not `\(a\|b\)`, `a+`,
not `a\+`, and `\.` for a literal dot. A pattern that does not parse shows why
on the message line.

A search matches case unless you `:set ignorecase`. With `:set smartcase` as
well, a pattern with an uppercase letter in it matches case again: `apple`
finds `APPLE`, but `Apple` does not. `\c` anywhere in a pattern ignores case
and `\C` matches case, whatever the options say; if a pattern has both, `\c`
wins. They are not part of what is matched: `/apple\c` searches for `apple`.
Changing an option after a search changes what `n` and `N` find. See
[Options](./command.md#options).

A cell has two kinds of text: its value as the sheet shows it (`$12.00`,
`50%`, the result of a formula) and its raw contents as `e` shows them for
editing (`12`, `0.5`, `=A1*2`). A search matches the shown value. With
`:set searchin=raw` it matches the raw contents instead, so `/SUM\(` finds the
cells whose formula calls `SUM`, and `/^12$` finds the cell showing `$12.00`.
A date shows the date in both. `:set searchin=value` goes back. Changing it
after a search changes what `n` and `N` find. See
[Options](./command.md#options).

While a search is in effect, every visible cell that matches it is drawn
highlighted, and the match under the cursor in a second color. The whole cell
is highlighted, not just the matching text. `:noh` hides the highlight until
the next search, or `n` or `N`. `:set nohlsearch` turns highlighting off. The
colors are the theme's `search` and `search.current` slots; see
[Themes](./themes.md).

A merged cell is searched once, as its top-left cell. An array formula is
searched once, as the cell that holds the formula.

## Other Keybindings

{{keys:other}}

Range selections made from navigation mode will be available to paste into a Cell Edit.

<aside>Note that for `q` this will not currently prompt you if the sheet is not
saved.</aside>

Note also that copy paste works with the system clipboard.

