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

Type `/`, then a pattern, then `Enter`. The cursor goes to the next cell after
it whose displayed value matches the pattern: left to right across a row, then
the next row down. `?` does the same backward: right to left, then the row
above. Only the current sheet is searched, and only cells that have something
in them.

`n` goes to the next match in the direction of the last search, so after `?`
it keeps going backward. `N` goes the other way, and does not change which
way `n` goes. Both take a count: `3n` goes to the third match.

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

A merged cell is searched once, as its top-left cell. An array formula is
searched once, as the cell that holds the formula.

## Other Keybindings

{{keys:other}}

Range selections made from navigation mode will be available to paste into a Cell Edit.

<aside>Note that for `q` this will not currently prompt you if the sheet is not
saved.</aside>

Note also that copy paste works with the system clipboard.

