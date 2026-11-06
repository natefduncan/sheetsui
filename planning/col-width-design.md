# Design: Column Width Control

Status: proposed 2026-11-05 — decisions W1–W15, all agreed in a design
interview. Related: `planning/config-design.md` (the `defaultwidth` option goes
in the config file), `planning/undo-design.md` (column width is already an
undoable change), `planning/freeze-design.md` (the clipping approach in W14).

Tall rows (row heights, wrapped text, `:autofit-rows`) are a separate design.
See "Out of scope".

## User

A person who has just typed or filled a column of data and sees `###` or text
cut off, and wants it to fit without pressing `<C-l>` fifteen times. They know
Excel's double-click on the column border. They also want to set an exact
width, put the same default width in the config file, and undo any of it.

## Today

- A width changes one character at a time, with `<C-h>` and `<C-l>` (`src/ui/mod.rs`,
  `ColNarrower` and `ColWider`). A count repeats it. `<C-h>` stops at 1;
  `<C-l>` has no upper limit.
- `Book::set_col_size` and `get_col_size` convert between characters and
  ironcalc's pixels at 5 pixels a character (`COL_PIXELS`). Width changes go
  through `change()`, so they undo.
- There is no way to set an exact width, to fit a column to its contents, or to
  choose a default width. The default is ironcalc's constant of 90 pixels,
  18 characters, and no workbook can change it.
- `.sui` writes a `col <n> width <chars>` line for every column that differs
  from the default. `.xlsx` saves widths in ironcalc's pixel units.
- `visible_columns_split` (`src/ui/render/viewport.rs`) lays columns out left
  to right. When one column is wider than the terminal, its sliding window can
  drop that column, so the cursor column disappears.

## Decisions

| # | Decision |
|---|----------|
| W1 | This design covers columns only: `:col-width`, `:autofit-cols`, a key for autofit, and a `defaultwidth` option. Row heights and mouse resize are out of scope. |
| W2 | Autofit measures the displayed value: the formatted text (`$12.00`, not `12`) and a formula's result, not its source. Width is measured in terminal cells, so a CJK or emoji character counts as 2. |
| W3 | Autofit measures every row of the column that has content, including rows that are scrolled off screen. The result does not depend on where the sheet is scrolled. There is no scan cap. |
| W4 | The result of autofit is clamped to a minimum of 3 and a maximum of 60 (fixed, not options). A column with nothing to measure is reset to the default width. |
| W5 | Autofit skips merged cells. It counts each other cell's own text in full and does not look at whether it could overflow into an empty neighbor. A column that holds only merged cells has nothing to measure, so W4 resets it. |
| W6 | `:col-width <n> [count]` sets the cursor column, or `count` columns going right, to `n`, a whole number from 1 to 255. `:col-width default [count]` clears the saved width instead. `:autofit-cols [count]` fits the same columns. A missing or bad argument is an error with a usage hint, and an error changes no width at all. |
| W7 | In range select mode both commands apply to every column the selection touches and ignore `count`. `:autofit-cols` then measures only the selected rows. In navigation mode it measures the whole column (W3). |
| W8 | Autofit is `gw` in navigation mode, with a count for the number of columns (`3gw`), and `w` in range select mode. Both are remappable and appear in the `modify` group of the help. `:col-width` has no key. |
| W9 | `defaultwidth` (short `dw`) is a `:set` option, a whole number from 1 to 255, default 18. It is the width of every column that has no width saved in the workbook. It is a display setting, never written to `.sui` or `.xlsx`, and it can go in the config file. |
| W10 | Each command is one undo step, however many columns it changes. `<C-h>` and `<C-l>` are unchanged: one step per key, and `5<C-l>` is one step. |
| W11 | Each command says what it did on the message line, for example `Column C width 20`, `Columns C:G width 20`, `Fitted 5 columns (2 empty, reset to 18)`. |
| W12 | Units do not change. A width is in terminal characters, saved at 5 pixels each. Excel's own scale is 9 pixels a character, so an autofit column can look narrow when the file is opened in Excel. The docs say so. |
| W13 | `:col-width default` (see W6) is the way to reset a column that has content. Autofit resets only a column with nothing to measure. |
| W14 | A column wider than the room on screen is drawn clipped to the room that is left. The saved width does not change. When the cursor is in such a column the message line says it is drawn clipped, as freeze does. |
| W15 | `<C-l>` stops at 255 and `<C-h>` at 1, so all three ways to set a width agree. At the limit the key does nothing and says nothing. |

## Out of scope

- Row heights, wrapped text, `:autofit-rows` and drawing tall rows. The
  renderer treats every row as one terminal line (`viewport.rs`,
  `get_visible_rows`), so tall rows are a rendering change and get their own
  design.
- Mouse resize, hiding columns, and a width for a whole workbook.
- Converting widths between the terminal scale and Excel's scale (W12).
- Saving `defaultwidth` in the workbook, or writing it back to the config.
- A scan cap for autofit on a very large column (W3).

## The commands

```
:col-width 20            the cursor column to 20
:col-width 20 5          five columns, from the cursor going right
:col-width default       clear the saved width of the cursor column
:autofit-cols            fit the cursor column
:autofit-cols 3          fit three columns
```

Errors, each leaving every width unchanged: no argument, `0`, a number over
255, something that is not a whole number, a `count` of 0, and a `count` that
runs past the last column. The usage hint is `Usage: col-width <1-255|default>
[count]`, and `Usage: autofit-cols [count]` for the other.

## Autofit

For each column to fit:

1. Take the cells to measure: every row (navigation mode) or the selected rows
   (range select), without merged cells (W5).
2. Measure each cell's displayed text in terminal cells (W2). A cell with a
   newline is measured by its longest line, because the grid draws one line.
3. If nothing was measured, the column is reset to `defaultwidth` (W4).
4. Otherwise the width is the longest measurement, clamped to 3 to 60.

`unicode-width` is already in the dependency tree through ratatui. Add it as a
direct dependency, at the version ratatui uses, so the measurement and the
drawing agree.

The measurement goes in the book layer next to the other column code, taking a
column and a row range, so it is tested without a terminal.

## The default width

`defaultwidth` is a number option, and the options table has only on/off and
choice kinds, so it needs a `Kind::Number` with a minimum and a maximum.
`set defaultwidth=24` and `set dw=24` set it, `set dw?` shows it, and a value
that is not a whole number from 1 to 255 is `E474`, as other options are.

ironcalc reports its own constant for any column with no saved width, so the
option cannot change that number. The book layer wraps it: if the column has
no entry in the sheet's own list of column settings, it reports
`defaultwidth`. If the model does not let us tell "no entry" from "set to 18",
the fallback is that a column at exactly 18 counts as unset, and the design
doc for the change notes which one was used.

**Used: the fallback.** ironcalc sets `custom_width` to `width != 90 pixels`, so
a column set to exactly 18 is stored the same as one with no entry. A column at
exactly 18 counts as unset and follows `defaultwidth`: with `dw=24`,
`:col-width 18` shows 24. Use `:col-width 19` or change the option instead.

`.sui` and `.xlsx` writers compare against `defaultwidth` so that a column
following the option writes no line. Because the option is not saved, a file
written with `dw=24` and read with `dw=18` shows those columns at 18. This is
what "a display setting" means, and the docs say so.

## Keys

| Mode | Key | Does |
|------|-----|------|
| Navigation | `gw` | autofit the cursor column; `3gw` fits three |
| Range select | `w` | autofit the columns the selection touches |

`gw` and `w` are free today (`gg` is the only `g` chord). They are new rows in
the key tables, with help text and a handler in each mode, and a user can
remap or `nunmap` them like any built-in.

## Wide columns

`visible_columns_split` is the place to fix (W14). A column's drawn width is
the smaller of its width and the room on screen after the row-number column
and any frozen columns. The frozen-column clipping already works this way and
its code is the model to follow. The same fix is used when the column is the
cursor column and when it is only on the way.

## Undo, freeze and merged cells

- A command changes several columns in one `change()` group, as fill does, so
  `u` undoes the whole command (W10).
- Frozen columns are measured and set like any other. Clipping them is the
  freeze code's job and is unchanged.
- Inserting and deleting columns already move widths with their columns,
  because ironcalc rewrites its column list. A test pins this.
- Merged cells are skipped in autofit only (W5). `:col-width` sets the width
  of every column it names whatever is merged.

## Testing

Book layer, with no terminal:

- Autofit width for text, numbers with a format, a formula result, a date, an
  empty cell, a multi-line cell, and a CJK character.
- The clamps: a 100-character cell gives 60; a 1-character cell gives 3.
- An empty column resets to the default; a column of merged cells does too.
- Rows that are off screen are counted; in range select only the selected rows.
- `:col-width` limits (0, 1, 255, 256, `default`) and that an error leaves
  every width unchanged, including a late failure at the `count`.
- Undo and redo are one step for a multi-column command.
- `.sui` and `.xlsx` round trip widths, and a column at the default writes
  nothing.
- Insert and delete columns carry the widths.

UI layer:

- `:col-width`, `:autofit-cols`, `gw`, `3gw` and `w` in range select do what
  they say, with the message line text (W11).
- A 255-wide column on a narrow terminal still shows the cursor column, with
  the clipping message (W14), and a frozen column next to it still fits.
- `<C-l>` stops at 255 and `<C-h>` at 1.
- `set dw=24`, `set dw?`, bad values, and `set dw` in the config file.
- A remapped and an unmapped `gw`.

## Documentation

- `docs/command.md`: `col-width` and `autofit-cols` in the command list; a
  short "Column widths" section with the limits, the clamp, the empty-column
  reset and the units note (W12); `defaultwidth` in the options list.
- `docs/navigation.md` and `docs/visual.md`: `gw` and `w`, and the key list
  groups.
- `docs/config.md`: `set defaultwidth=24` as an example line.
- The help pages that list keys are generated from the tables, so they follow.

## Facts to check while building

These came from reading the code and the library, and from memory of Excel.
They are not yet checked by running anything:

- Whether the model's column list can tell an unset column from one set to the
  default (W9).
- That autofit on an empty column resets to the default in Excel, and that the
  ribbon's autofit with a selection fits the selected cells only (W4, W7).
  Both are from memory of Excel's behavior, not checked.

## Implementation steps

1. Column measurement in the book layer, with its tests: displayed text,
   terminal width, merged cells, clamps, row ranges.
2. `:col-width` with `default`, the limits, atomic errors, one undo step and
   the message.
3. `:autofit-cols` and the keys `gw` and `w`, including range select.
4. `Kind::Number` and `defaultwidth`, with the unset-column rule and the file
   writers following it.
5. Clip a wide column in the layout, and the `<C-l>` and `<C-h>` limits.
6. Docs.

Each step builds and passes the tests on its own, and each is one commit.
