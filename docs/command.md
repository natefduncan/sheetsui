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
sheetui again.

## Commands

The currently supported commands are:

* `write [path]` save the current spreadsheet. If the path is provided it will save it to that path and update the current file path for subsequent saves. If omitted it will save to the path you are currently editing (or `Untitled.sui` for a new empty workbook). The format is auto-detected from the file extension: `.xlsx` uses Excel format, `.csv` writes the first sheet's displayed values, everything else uses the native `.sui` format. `w` is a shorthand alias for this command.
* `insert-rows [number]` Inserts a row into the sheet at your current row. If the number is provided then inserts that many rows. If omitted then just inserts one.
* `insert-cols [number]` Just line `insert-rows` but for columns.
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
* `undo` Undo the last change, the same as `u` in navigation mode.
* `redo` Redo the last change that you undid, the same as `r` in navigation mode.
* `nnoremap <keys> <rhs>` map keys in navigation mode. `vnoremap` does the same for range select mode. The right-hand side is keys, or `<Cmd>command<CR>`. See [Key Mappings](./keys.md).
* `nunmap <keys>` remove a navigation mode mapping, or turn off a built-in key. `vunmap` does the same for range select mode.
* `nmap` list your navigation mode mappings and the warnings from loading the keys file. `vmap` lists the range select ones.

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

`\c` and `\C` in a pattern beat `ignorecase` and `smartcase`. See
[Navigation](./navigation.md#search).

<aside>Note that in the case of `quit` and `edit` that we do not currently
prompt you if the current spreadsheet has not been saved yet. So your changes
will be discarded if you have not saved first.</aside>

