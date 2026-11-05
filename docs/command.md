# Command Mode

You enter command mode by typing `:` while in navigation mode. You can then
type a command and hit `Enter` to execute it or `Esc` to cancel.

## Editing and history

* `Left`, `Right` (or `Ctrl-b`, `Ctrl-f`) move the cursor one character.
* `Home`, `End` (or `Ctrl-a`, `Ctrl-e`) move the cursor to the start or end.
* `Backspace` deletes the character before the cursor. `Delete` deletes the
  character at the cursor.
* `Ctrl-k` deletes to the end of the line. `Ctrl-u` deletes to the start.
* `Up` shows the previous command. `Down` shows the next one. If you type
  some text first, `Up` and `Down` show only the commands that start with
  that text. `Down` past the newest command gives back the text you typed.

Every command that you run with `Enter` goes into the history, also a command
that fails, so you can fix a typo with `Up`. A command that you cancel with
`Esc` does not. The history keeps the last 100 commands in
`$XDG_STATE_HOME/sheetui/history` (by default
`~/.local/state/sheetui/history`), so it is still there when you start
sheetui again. Search patterns have a history of their own; see
[Search](./navigation.md#search).

## Commands

The currently supported commands are:

* `write [path]` save the current spreadsheet. If the path is provided it will save it to that path and update the current file path for subsequent saves. If omitted it will save to the path you are currently editing (or `Untitled.sui` for a new empty workbook). The format is auto-detected from the file extension: `.xlsx` uses Excel format, `.csv` writes the first sheet's displayed values, everything else uses the native `.sui` format. `w` is a shorthand alias for this command.
* `insert-rows [number]` Inserts a row into the sheet at your current row. If the number is provided then inserts that many rows. If omitted then just inserts one.
* `insert-cols [number]` Just line `insert-rows` but for columns.
* `freeze` with no arguments pins every row above and every column left of the cursor, counted from row 1 and column A even when the sheet is scrolled. On A1 there is nothing to pin, so nothing changes and a message says so.
* `freeze <rows> [cols]` pin that many rows (and columns) so they stay visible while the rest scrolls. `freeze 0 0` clears it. The freeze is kept per sheet, is not undoable, and is saved in `.xlsx` files only. A line marks the boundary, styled by the `freeze.line` theme slot.
* `unfreeze` clear the freeze on the current sheet. The same as `freeze 0 0`.
* `color-rows [count] <color>` color rows. The count of rows if given specifies how many rows going down to color. 
* `color-cols [count] <color>` color columns. The count of rows if given specifies how many columns going right to color.
* `color-cell <color>` Color the currently selected cells.
* `rename-sheet [idx] <name>` rename a sheet. If the idx is provide then renames that sheet. If omitted then it renames the current sheet.
* `new-sheet [name]` Creates a new sheet. If the name is provided then uses that. If omitted then uses a default sheet name.
* `select-sheet <name>` Select a sheet by name.
* `edit <path>` Open a spreadsheet from the given path. Format is auto-detected from the file extension (`.xlsx` → Excel, `.csv` → CSV, everything else → `.sui`). `e` is a shorthand alias for this command.
* `help [topic]` Display help for a given topic.
* `theme [name]` Switch to a theme. With no name, lists the built-in and user themes and any warnings from the last theme load. See [Themes](./themes.md).
* `export-csv <path>` Export the current sheet to a csv file at `<path>`.
* `quit` Quits the application. `q` is a shorthand alias for this command.
* `system-paste` Paste from the system clipboard
* `sort [skip <N>] [<column> [asc|desc] ...]` Sort rows. See Sorting below.
* `set [option ...]` Change or show an option for this session. See Options below.
* `s/pattern/replacement/[flags]` Replace text in the current cell. See Replacing below.
* `nohlsearch` Stop highlighting the matches of the last search until the next search. `noh` is a shorthand alias for this command.
* `undo` Undo the last change, the same as `u` in navigation mode.
* `redo` Redo the last change that you undid, the same as `r` in navigation mode.
* `nnoremap <keys> <rhs>` map keys in navigation mode. `vnoremap` does the same for range select mode. The right-hand side is keys, or `<Cmd>command<CR>`. See [Key Mappings](./keys.md).
* `nunmap <keys>` remove a navigation mode mapping, or turn off a built-in key. `vunmap` does the same for range select mode.
* `nmap` list your navigation mode mappings and the warnings from loading the keys file. `vmap` lists the range select ones.

Inserting rows at or above the last frozen row (or columns at or left of the last frozen column) grows the freeze by the number inserted, as in Excel. Inserting below or right of the freeze leaves it alone. Undoing an insert does not change the freeze.

## Sorting

`sort` sorts the rows of the last range selection. If there is no range
selection, it sorts all the rows of the sheet that have data.

* `sort` sorts by the column the cursor is on, descending.
* `sort asc` sorts by the column the cursor is on, ascending.
* `sort B` sorts by column B, descending.
* `sort B asc C` sorts by column B ascending. Rows with the same value in B
  then sort by column C, descending.
* `sort skip 1 B` keeps the first row in place (for example a header row) and
  sorts the rest. `skip 3` keeps the first 3 rows.

A direction applies only to the column just before it. `sort B C asc` sorts B
descending and C ascending. A column with no direction sorts descending.
`skip` must come first.

Every cell of a sorted row moves: values, formulas, and cell styles. A range
selection limits which rows sort, not which columns. Row height does not move.

The sort order is: numbers, then text (case is ignored), then `TRUE`/`FALSE`,
then errors. Empty cells are always last. Rows with equal values keep their
order. Formula cells sort by their result.

Formulas follow the same rules as in Excel:

* A formula in a sorted row changes as if you copied it to its new row. So
  `=B2*2` that moves to row 5 becomes `=B5*2`.
* A formula outside the sorted rows that points into them keeps its address.
  After the sort it can show a different value.

You cannot sort rows that contain merged cells or array formulas.

A sort undoes in one step: `u` in navigation mode, or `:undo`.

## Replacing

`s/pattern/replacement/` replaces the first match of `pattern` in the current
cell. `s/pattern/replacement/g` replaces every match in it. The pattern is
the same as for a [search](./navigation.md#search), and `ignorecase`,
`smartcase`, `\c` and `\C` apply to it the same way. An empty pattern, as in
`s//x/`, is the last search pattern. `substitute` is the long name of `s`.

`s` edits what you edit in CellEdit mode, the raw contents, whatever
`searchin` is. `s/SUM/AVERAGE/` changes the formula `=SUM(A1:A3)`. The new
text goes into the cell as if you typed it: a formula stays a formula, a
number stays a number, and the cell's format and style do not change. A
replacement that makes a bad formula shows the formula error, as typing it
would.

So a search and a replacement can see different text. On a cell that stores
`12` and shows `$12.00`, `/12\.00` finds the cell, but `s/12\.00/x/` does not
match it: it matches the stored `12`. Use `set searchin=raw` to search the
text that `s` edits.

In the replacement:

* `&` or `\0` is the whole match. `\1` to `\9` are the groups of the
  pattern: `s/(\w+) (\w+)/\2 \1/` swaps two words.
* `\&` is a literal `&`, `\\` a literal backslash, and `\/` a literal `/`.
* `$` is just a `$`.
* A cell holds one line, so `\n` and `\r` are errors, as is a backslash before
  any other letter.

Any character that is not a letter, a digit, white space, `\`, `"` or `|` can
stand in for `/`: `s#a/b#c#` replaces `a/b` with `c`. The last delimiter can
be left out when there are no flags: `s/a/b`.

Flags:

* `g` replaces every match in the cell, not only the first.
* `i` ignores case and `I` matches case, whatever `ignorecase` and
  `smartcase` say. If both are given, the last one wins. A `\c` or `\C` in
  the pattern still wins over them.
* `n` counts the matches and changes nothing: `3 matches on 2 cells`. Without
  `g` it counts the first match in each cell. It is not an undo step.
* `e` gives no error when nothing matches.
* `c` asks before each match. See below.

Flags combine, as in `s/a/b/ge` or `%s/x//gn`. An unknown flag is an error that names it.

### Confirming with `c`

With `c`, `s` stops at each match, in row-major order. The cursor goes to its
cell, scrolling to it if it is off screen, the match is highlighted, and the
bottom line says `replace with <replacement> (y/n/a/q/l)?`:

* `y` replaces this match and goes on.
* `n` skips it.
* `a` replaces this match and all the rest without asking.
* `l` replaces this match and stops.
* `q` or `Esc` stops.

Other keys do nothing. Without `g` only the first match in each cell is asked
about. With `g` each match in a cell is asked in turn. After a `y` the search
goes on after the replaced text, so a replacement that contains the pattern,
as in `s/a/aa/gc`, does not loop.

The whole session is one undo step, however many matches you confirmed. `u`
reverts all of it. If you confirmed none, there is no step. When it ends the
cursor is on the last cell that changed, or back where it started if none did.
`c` works with `%` and `'<,'>` too. With `n`, `c` is ignored: it only counts.

A range in front of `s` replaces in more than the current cell:

* `%s/pattern/replacement/` replaces in every cell with content in the
  current sheet.
* `'<,'>s/pattern/replacement/` replaces in the cells of the last range
  selection. Typing `:` in range select mode puts `'<,'>` in the prompt for
  you, and running the command ends the selection. The last selection is
  kept after range select mode ends, even with `Esc`. With no selection on
  the current sheet it says `E20: Mark not set`.

Only `s` takes `%`. Other commands typed in range select mode, like `sort`,
already act on the selection, so they ignore the `'<,'>` in front of them.

A merged cell counts as one cell, at its top left. A cell that an array
formula fills is part of the formula's cell.

`s` says how much it did, for example `3 substitutions on 1 cell`. If nothing
matches it says `E486: Pattern not found: <pattern>` and changes nothing. It
cannot edit an array formula, or a cell that one fills: if one matches,
nothing changes at all. A replacement undoes in one step however many cells
it changed, and `u` puts the cursor on the first cell it changed. One that
changes nothing is not a step.

## Options

`set` changes options. They last until you quit and are not saved.

* `set ws` turns an option on. `set nows` turns it off. `set ws!` toggles it.
* `set ws?` shows its value on the message line, for example `nowrapscan`.
* `set name=value` sets an option that takes a value. For such an option,
  `set name` shows its value.
* `set` with no option shows every option.

You can give several at once: `set nows ws?`. They apply in order and stop at
the first error. An unknown option says `E518: Unknown option: <name>`. A form
that does not fit the option, like `set ws=1`, or a value that is not one of
the option's values, says `E474: Invalid argument`.

The options are:

* `wrapscan` (`ws`), on or off, default on. A search that passes the end of
  the sheet goes on from the other end. Off, it stops there and says `E385`.
  See [Navigation](./navigation.md).
* `ignorecase` (`ic`), on or off, default off. A search ignores case: `apple`
  finds `Apple` and `APPLE`.
* `smartcase` (`scs`), on or off, default off. With `ignorecase` on, a pattern
  that has an uppercase letter matches case after all. On its own it does
  nothing. A letter after a backslash, as in `\S`, does not count.
* `searchin`, `value` or `raw`, default `value`. What text a search matches:
  `value` is what the sheet shows, like `$12.00` or a formula's result; `raw`
  is what you edit, like `12` or `=A1*2`. Use `set searchin=raw` to search
  formulas.
* `hlsearch` (`hls`), on or off, default on. The cells that match the last
  search are highlighted. `nohlsearch` hides the highlight until the next
  search; `set hls` shows it again.
* `incsearch` (`is`), on or off, default on. While you type a `/` or `?`
  pattern, the cursor jumps to the first match and the matches are
  highlighted. `Esc` puts the cursor back. See
  [Navigation](./navigation.md#search).

`\c` and `\C` in a pattern beat `ignorecase` and `smartcase`. See
[Navigation](./navigation.md#search).

<aside>Note that in the case of `quit` and `edit` that we do not currently
prompt you if the current spreadsheet has not been saved yet. So your changes
will be discarded if you have not saved first.</aside>

