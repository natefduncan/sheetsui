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
* `delete-rows [number]` Deletes the row at your current row and moves the rows below it up. If the number is provided then deletes that many rows, counting down from the cursor row. If omitted then deletes one. In range select mode it deletes every row the selection touches and ignores the number. `dd` does the same in navigation mode.
* `delete-cols [number]` Just like `delete-rows` but for columns, moving the columns to the right left. `dc` does the same in navigation mode.
* `fill-down [count]` copies the cell above onto the cursor cell, moving formula references and copying formats. With a count it fills that many cells down from the cursor, all from the cell above the cursor. `<C-d>` does the same in navigation mode. See [Filling](#filling).
* `fill-right [count]` Just like `fill-down` but copies the cell to the left, to the right. `<C-r>` does the same in navigation mode.
* `fill-series <up|down|left|right> <count>` takes the selection, or the cursor cell, as seeds and extends it by count cells in that direction, continuing a series. See [Filling](#filling).
* `col-width default [count]` clears the saved width of the cursor column, or `count` columns going right from it, so they follow the `defaultwidth` option. It is one undo step.
* `col-width <1-255> [count]` sets the cursor column, or `count` columns going right from it, to that many characters. It is one undo step. A missing or bad width, a count of 0, or a count that runs past the last column is an error, and no width changes.
* `autofit-cols [count]` fits the cursor column, or `count` columns going right from it, to the widest displayed value in each (3 to 60 characters, wide characters count as 2, merged cells are skipped). A column with nothing to measure is reset to the `defaultwidth` option's value. It is one undo step.
* `format [<preset|code>]` sets the number format of the cursor cell, or of the selected cells in range select mode. The format is a preset name (`general`, `number`, `integer`, `thousands`, `currency`, `percent`, `date`, `time`, `scientific`, not case sensitive) or a raw Excel format code, which is everything after the command, so `format 0.00 "USD"` works. A raw code is checked first; a bad one is the error `Invalid number format: <code>` and nothing changes. It applies to empty and text cells too, never changes a column width, and is one undo step. With no argument it shows the cursor cell's format.
* `format-cols [count] <preset|code>` and `format-rows [count] <preset|code>` set the number format of the cursor column or row, or `count` going right or down, over every cell in it, replacing formats cells already have. The count comes first and only when a format follows it: `format-cols 3 percent` is three columns, a lone `format-cols 0` is the format `0`, and `format-cols 3 0` is three columns formatted `0`. In range select mode they apply to every column or row the selection touches and ignore `count`. A missing format, a bad code, a count of 0 or one past the last column or row is an error and nothing changes. It is one undo step and never changes a column width.
* `decimals <+n|-n|n>` adds, removes or sets the decimal places of the cursor cell, or of the selected cells in range select mode: `+1` is Excel's Increase Decimal, `-1` Decrease, `3` sets exactly 3 (0 to 15). The new format is worked out once from the cursor cell (the first selected cell in range select mode) and applied to every cell, so a mixed selection gets one format. It works on plain number formats (digits, `0`, `#`, `,`, `.`, with an optional `$` or `%`) and on `General`, which counts as `0`, so `+1` gives `0.0` and `-1` leaves it alone. Any other format (dates, times, scientific, several `;` sections, quoted text) is the error `Cannot change decimals of <code>`, and nothing changes. It is one undo step.
* `style <attr>...` changes the text style of the cursor cell, or of the selected cells in range select mode. The words are `bold`, `italic`, `underline` and `strike`, which toggle, and `nobold`, `noitalic`, `nounderline` and `nostrike`, which clear; several can be given, as in `style bold underline`. The words `left`, `center`, `right` and `general` set the horizontal alignment (`general` clears it: text left, numbers right), `color=<color>` sets the font color (the colors of `color-cell`) and `nocolor` clears it; at most one alignment and one color per command. A toggle is decided once, from the cursor cell (the first selected cell in range select mode): if it has the attribute the whole area loses it, otherwise the whole area gets it, so a mixed selection ends up uniform. It applies to empty and text cells too, never changes a column width, and is one undo step. An unknown word is the error `Unknown style: <word>`, a word given twice is `Repeated style: <word>`, and nothing changes. `bold`, `italic`, `underline` and `strike` are also commands of their own that toggle that one attribute.
* `style-cols [count] <attr>...` and `style-rows [count] <attr>...` apply `style` attributes over every cell of the cursor column or row, or `count` going right or down. The count comes first and only when an attribute follows: `style-cols 3 bold` is three columns. A toggle is decided by the cursor cell. In range select mode they apply to every column or row the selection touches, decided by the first selected cell, and ignore `count`. Cells keep their other style, such as a fill or number format. A count of 0 or one past the last column or row is an error and nothing changes. It is one undo step and says what it did, as in `Bold: column C`.
* `align <left|center|right|general>` is `style` with that alignment, and `font-color <color>` is `style color=<color>`. Both take no count; use `style-cols` or `style-rows` for whole columns and rows. They say what they did, as in `Align center: 4 cells`.
* `clear-style` puts everything visual on the cursor cell, or the selected cells in range select mode, back to the default: bold, italic, underline, strikethrough, font color, fill, alignment and the number format. Cell values and formulas are not touched. `clear-style-cols [count]` and `clear-style-rows [count]` do the same over whole columns or rows, with the count and range select rules of `style-cols`. It is one undo step and says what it did, as in `Cleared style: 12 cells` or `Cleared style: columns A:B`.
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
* `config` list every warning from loading the config file, the keys file and the theme at startup, with its file and line, or say there are none. See [The Config File](./config.md).
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

Inserting rows at or above the last frozen row (or columns at or left of the last frozen column) grows the freeze by the number inserted. Inserting below or right of the freeze leaves it alone. Undoing an insert does not change the freeze.

## Number formats

`:format`, `:format-cols`, `:format-rows` and `:decimals` control how a number is shown. They change the display format only: typing still decides whether a cell holds a number or text. The format is saved in `.sui` and `.xlsx` files.

| Preset | Code | Key |
|--------|------|-----|
| `general` | `General` | `<A-0>` |
| `number` | `0.00` | `<A-1>` |
| `integer` | `0` | |
| `thousands` | `#,##0` | |
| `currency` | `$#,##0.00` | `<A-4>` |
| `percent` | `0.00%` | `<A-5>` |
| `date` | `yyyy-mm-dd` | `<A-3>` |
| `time` | `hh:mm:ss` | `<A-2>` |
| `scientific` | `0.00E+00` | `<A-6>` |

The keys work in navigation and range select mode, and `<A-.>` and `<A-,>` add and remove a decimal place. See [Key notation](./keys.md#key-notation) for why they use Alt.

Anything that is not a preset name is a raw Excel format code. It is everything after the command (or after the count), so spaces and quotes are part of it:

```
:format
:format currency
:format 0.00 "USD"
:format-cols percent
:format-cols 3 percent
:format-rows 2 0.000
:decimals +1
:decimals -1
:decimals 3
```

* A raw code is checked before it is used, by formatting a sample number with it. A bad code is the error `Invalid number format: <code>`, and nothing changes, including later cells of a range.
* A format applies to every cell in the area, including empty and text cells. A number typed later into a formatted empty cell shows with that format. Text shows no change.
* `format-cols` and `format-rows` replace the format of cells in the column or row that already have their own.
* A format never changes a column width. A number that does not fit shows `###`: use `gw` to fit the column.
* Every command is one undo step and says what it did, for example `Format currency: 12 cells`, `Format 0.000: column C` or `Decimals +1: 12 cells`.
* `decimals` reads the cursor cell's format (the first selected cell in range select mode) and applies the one result to the whole area. Only the first section of the code, before any `;`, is read. A code with no `.` gets one when you add decimals, and `-1` on a format with no decimals leaves it alone.

**Typing into a formatted cell.** Typing a plain number keeps the cell's format. Typing a value that carries its own format, such as `5%` or `$12`, gives the cell the format that value implies and drops the one you set (`5%` in a currency cell becomes `#,##0%`). These commands do not change that, or copy and paste. Fill copies formats.

## Column widths

A width is a number of terminal characters, from 1 to 255. `:col-width`, `:autofit-cols`, `<C-l>` and `<C-h>` all keep to that range. `<C-l>` stops at 255 and `<C-h>` at 1; at the limit the key does nothing and says nothing.

* `:col-width 24` sets the cursor column to 24 characters, and `:col-width 24 3` sets three columns going right from it.
* `:col-width default` clears the cursor column's width, and `:autofit-cols 2` fits two columns.

* A column with no width of its own shows the `defaultwidth` option (18 unless you change it). `:col-width default` is the way to reset a column that has content.
* `:autofit-cols` clamps the result to 3 to 60 characters. A column with nothing to measure is reset to the `defaultwidth` value rather than shrunk to 3.
* Each command is one undo step, however many columns it changes, and says what it did on the message line, such as `Column C width 20`. `5<C-l>` is one undo step too.
* A column wider than the room on screen is drawn clipped to the room that is left. Its saved width does not change, and the message line says it is drawn clipped while the cursor is in it.
* **Units.** Widths are in terminal characters and are saved at 5 pixels each in `.xlsx` files. Excel's own scale is 9 pixels a character, so a column fitted here can look narrow when the file is opened in Excel.
* A column set to exactly 18 counts as having no width of its own, and follows `defaultwidth`. Use `:col-width 19`, or change the option, instead.

## Deleting rows and columns

`:delete-rows` and `:delete-cols` ask for no confirmation. `u` undoes a delete in one step, and puts the cursor back where it was. A count of 0, a count that is not a whole number, or a count past the end of the sheet is an error and changes nothing. So is a delete that cuts through part of an array formula. A delete that covers the whole array formula removes it.

* The cursor stays at the first deleted row (or column). If a range selection deletes rows above the cursor, the cursor moves up by that many so it still points at the same data.
* Deleting rows at or above the last frozen row (or columns at or left of the last frozen column) shrinks the freeze by the number of deleted rows (or columns) that were inside it. Deleting below or right of the freeze leaves it alone. Undoing a delete does not restore the freeze, the same as for an insert. This rule is sheetui's own and is not claimed to match Excel.
* Formulas that point below the deleted rows are rewritten to follow the cells. A reference to a deleted cell becomes `#REF!`.
* A range reference does not shrink the way it does in Excel. If you delete an end of the range, such as row 1 of `=SUM(A1:A10)`, the formula becomes `=SUM(#REF!:A9)` and shows `#VALUE!`. Deleting row 10 gives `=SUM(A1:#REF!)`. Fix those formulas by hand.
* Merged cells are not moved by a delete, and they are not moved by an insert either. Check them after you insert or delete rows or columns.

## Filling

`:fill-down` and `:fill-right` (`<C-d>` and `<C-r>`) copy and never make a series. The source is the cell above (down) or to the left (right). A single number, `Item 1` or `Jan` is copied, not continued. Formulas get their references moved, and formats are copied with the values. A fill is one undo step, asks no confirmation, and overwrites what is in the target.

* With no selection, the cursor cell is filled and the cursor does not move. `:fill-down 5` and `5<C-d>` fill the cursor cell and the four below it, all from the cell above the cursor.
* In range select mode the top row (fill down) or left column (fill right) of the selection is the source, and each column (or row) is filled from its own cell. A count is ignored. A selection one row tall (or one column wide) uses the row above (or the column to the left) as its source. Range select mode stays active, so `<C-d>` then `<C-r>` fills a block from its top left cell. `x` in range select stamps the start cell over the selection and does not copy formats; `<C-d>` then `<C-r>` gives the same values and formulas, with formats copied.
* With the `'<,'>` prefix that the prompt adds in range select mode, the commands use the selection and ignore the count.
* Nothing to copy from (row 1 for down, column A for right), a fill past the last row or column, or a fill that cuts through part of an array formula shows a message and changes nothing. Merged cells are not handled.

`:fill-series <direction> <count>` needs both arguments, and a missing or bad one is an error with a usage hint. The seeds are never overwritten. Each seed column (or row) is read for:

* two or more numbers with a constant step: seeds `1, 2` with `:fill-series down 3` give `3, 4, 5`;
* text ending in a number: `Item 1, Item 2` gives `Item 3, Item 4`;
* month and day names, keeping their case.

Anything else is copied, and a block of several seeds repeats. A single number seed gives `1, 1, 1`. `up` and `left` exist only for series.

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

`set` changes options. They last until you quit and `set` never saves them. A `set` line in the
[config file](./config.md) sets the start value for every session.

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

* `defaultwidth` (`dw`), a whole number from 1 to 255, default 18. The width of
  every column that has no width of its own, and it changes them at once. It is
  a display setting: no `.sui` or `.xlsx` file saves it, so a file read with another
  `defaultwidth` shows its unset columns at that one. ironcalc cannot tell a
  column set to exactly 18 from one with no width, so a column at exactly 18
  counts as unset and follows `defaultwidth`.

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

