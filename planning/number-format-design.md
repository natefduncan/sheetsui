# Design: Number Format Control

Status: proposed 2026-11-05 — decisions N1–N13, all agreed in a design
interview. Related: `planning/col-width-design.md` (the column and range select
rules in N8 follow it, and N12 leaves widths to autofit), `planning/undo-design.md`,
`planning/keys-design.md` (why the keys in N6 are Alt and not Ctrl+Shift).

## User

A person who has typed or filled a column of numbers and wants them to read as
currency, percent, a date, or a fixed number of decimals, the way Excel's
Ctrl+Shift+4, 5 and 3 do. They want to format a cell, a selection, or a whole
column, to see what format a cell has, and to undo any of it.

## Today

- The only way to get a format is to type it: `$12` makes currency, `5%`
  percent, a date a date. ironcalc infers the format from what you type.
  `docs/edit.md` says the type of a cell can't be changed afterward.
- `Book::set_cell_style(&[("num_fmt", code)], area)` already exists and is how
  `color-cell` sets a fill. `set_col_style` and `set_row_style` do the same over
  a whole column or row.
- ironcalc stores the `num_fmt` string without checking it
  (`user_model/common.rs`, the `"num_fmt"` arm). A bad code is saved and the
  cell then renders as an error.
- `.sui` and `.xlsx` already save `num_fmt` for every cell. The renderer already
  reads it to color date and number cells. Fill copies formats.
- There is no command to set a format and none to see one. Terminals cannot
  send Ctrl+Shift on a character, so Excel's own chords are impossible
  (`docs/keys.md`, "Key notation").

## Decisions

| # | Decision |
|---|----------|
| N1 | `:format <fmt>` formats the cursor cell, or the selection in range select mode. `:format-cols [count] <fmt>` and `:format-rows [count] <fmt>` format whole columns and rows, going right or down from the cursor. They mirror `color-cell`, `color-cols` and `color-rows`. |
| N2 | `<fmt>` is a preset name or a raw Excel format code. The presets are `general` (`General`), `number` (`0.00`), `integer` (`0`), `thousands` (`#,##0`), `currency` (`$#,##0.00`), `percent` (`0.00%`), `date` (`yyyy-mm-dd`), `time` (`hh:mm:ss`) and `scientific` (`0.00E+00`). Anything else is a raw code. |
| N3 | A raw code is checked before it is applied, by formatting a sample number with it and refusing if the formatter reports an error. A bad code is the error `Invalid number format: <code>` and changes nothing, including later cells of a range. |
| N4 | `:format` with no argument shows the cursor cell's format on the message line, such as `Format: 0.00%`, or `Format: General`. It changes nothing. |
| N5 | A format applies to every cell in the area, including empty and text cells, as in Excel. A number typed later into a formatted empty cell shows with that format. Text cells show no visible change. |
| N6 | The preset keys are Excel's digit with Alt in place of Ctrl+Shift, in navigation and range select mode: `<A-1>` number, `<A-2>` time, `<A-3>` date, `<A-4>` currency, `<A-5>` percent, `<A-6>` scientific, `<A-0>` general. In navigation mode they format the cursor cell, in range select the selection. They are remappable and show in the help. |
| N7 | `:decimals <+n\|-n\|n>` adds, removes or sets decimal places: `+1` is Excel's Increase Decimal, `-1` Decrease, `3` sets exactly 3. The range is 0 to 15. It works on a plain number format (digits, `0`, `#`, `,`, `.`, with an optional `$` or `%`) and on `General`, which counts as `0` as a base, so `+1` gives `0.0` and `-1` leaves it alone. Any other format (dates, times, scientific, several sections, quoted text) is the error `Cannot change decimals of <code>`. An error changes nothing in the whole area. The keys are `<A-.>` for one more and `<A-,>` for one fewer, in both modes. |
| N8 | In range select mode `:format-cols` and `:format-rows` apply to every column or row the selection touches and ignore `count`, as `:col-width` does. Plain `:format` and `:decimals` apply to exactly the selected cells. |
| N9 | Typed-input inference is not changed, and neither is copy and paste. Fill already copies formats. What typing `5%` into a currency cell does is checked while building and recorded in the docs, not changed here. |
| N10 | Each command and key is one undo step, however many cells it changes, and says what it did: `Format currency: 12 cells`, `Format 0.000: column C`, `Decimals +1: 12 cells`. |
| N11 | The format is everything after the command, or after the count, so a raw code may contain spaces and quotes: `:format 0.00 "USD"`. Preset names are not case sensitive. For columns and rows the count comes first and only when a format follows it: `:format-cols 3 percent`. A bare `:format-cols 0` is the format `0`, and `:format-cols 3 0` is three columns formatted as `0`. |
| N12 | Format commands and keys never change a column's width. A number that does not fit shows `###`, and `gw` (autofit) is the fix. |
| N13 | `:format-cols` and `:format-rows` override the format of cells in the column or row that already have their own, as Excel does. |

## Out of scope

- User-defined presets, preset names in the config file, and a list of
  recently used codes.
- A `text` preset (`@`), and changing the type of a cell. Typing still decides
  whether a cell holds a number or text.
- A format painter, and changes to copy and paste (N9).
- Color conditions in codes (`[Red]`) as presets. A raw code may contain them
  if the formatter accepts them.
- Locale: formats use ironcalc's locale and are not translated.
- Changing the rule by which `$12` and `5%` infer a format (N9).

## The commands

```
:format                       show the cursor cell's format
:format currency              the cursor cell, or the selection
:format 0.00 "USD"            a raw code; spaces and quotes are part of it
:format-cols percent          the cursor column
:format-cols 3 percent        three columns, going right
:format-rows 2 0.000          two rows, going down
:decimals +1                  one more decimal place
:decimals -1                  one fewer
:decimals 3                   exactly three
```

Errors, each leaving everything unchanged: no format for `format-cols` or
`format-rows`, a bad raw code (N3), a `count` of 0 or one past the last column
or row, a `:decimals` argument that is not `+n`, `-n` or a whole number from 0
to 15, and `:decimals` on a format it cannot change (N7). The usage hints are
`Usage: format [<preset|code>]`, `Usage: format-cols [count] <preset|code>`
(and the same for rows), and `Usage: decimals <+n|-n|n>`.

## Checking a code

The check formats a sample number (for example 1234.5) with the code and
looks at the formatter's result. If the formatter's error path returns, the
code is refused. The check is in the book layer, next to the code that sets
the style, so it is tested without a terminal. It runs once per command, not
per cell.

## Decimals

`:decimals` reads the cursor cell's format in navigation mode, and the first
cell of the selection in range select mode, and computes one new code that it
applies to the whole area. So a mixed selection gets one format, as in Excel.

The change is made on the first section of the code only, before any `;`, and
only on the run of `0` and `#` after the `.`. A code with no `.` gets one when
`+n` is asked for. `General` is read as `0`. The cases that are refused (N7)
are listed there; the code is parsed once, so "plain number format" has one
definition in one function with its own tests.

## Keys

| Mode | Keys | Does |
|------|------|------|
| Navigation, range select | `<A-1>` … `<A-6>`, `<A-0>` | apply the preset (N6) |
| Navigation, range select | `<A-.>`, `<A-,>` | one more, one fewer decimal |

`<A-digit>` is a different key from the digit that starts a numeric prefix,
and `<A-.>` and `<A-,>` are free. Check that the terminals the project cares
about report Alt with these characters; the keys are remappable if one does
not. The key tables get new rows, help text and handlers in both modes.

## Undo and messages

A command changes its area in one `change()` group, as fill does, so one `u`
undoes it. The message line text is in N10. A refused command shows its error
and changes nothing.

## Testing

Book layer, with no terminal:

- Each preset sets the code in the table, and a raw code passes through.
- The check refuses bad codes and accepts good ones, including a code with a
  space and quotes.
- A format applies to an empty cell, a text cell and a number cell, and a
  number typed later into the empty cell shows with it.
- `:format-cols` and `:format-rows` override existing cell formats (N13) and a
  range spanning the sheet is not slow.
- Decimals: `General` +1 and -1, `0.00` +1 and -1, `#,##0` +2, `$#,##0.00` -2,
  `0.00%` +1, a set to 0 and to 15, and refusals for a date, a time,
  scientific, a code with `;` sections and a quoted literal.
- A mixed selection gets one format from the first cell.
- Undo and redo are one step for a multi-cell command, and an error leaves
  everything unchanged, including a failure at a later cell.
- `.sui` and `.xlsx` round trip every preset code.

UI layer:

- `:format`, `:format-cols`, `:format-rows` and `:decimals` do what they say,
  with the message line text (N10) and the usage errors.
- `:format` with no argument shows the cell's format.
- The preset keys and `<A-.>`, `<A-,>` work in both modes, and can be
  remapped or unmapped.
- Count rules in N11: `format-cols 3 percent`, `format-cols 0`,
  `format-cols 3 0`.
- Range select: `:format-cols` applies to the touched columns and ignores the
  count; `:format` and `:decimals` apply to the selected cells only.
- A format never changes a column width (N12).

## Documentation

- `docs/command.md`: `format`, `format-cols`, `format-rows` and `decimals` in
  the command list, and a "Number formats" section with the presets table, raw
  codes, the check, the decimals rules and what was found about typing (N9).
- `docs/navigation.md` and `docs/visual.md`: the preset keys and the decimals
  keys; the key lists in the help follow from the tables.
- `docs/edit.md`: update the aside that says cell types cannot be changed to
  point to `:format` for the display format.
- `docs/keys.md`: say that Excel's Ctrl+Shift chords cannot be sent by a
  terminal, which is why the preset keys use Alt.

## Facts to check while building

These came from reading the code and the library. None of them has been
checked by running anything:

- How ironcalc applies a full-column or full-row range over cells that already
  have a style (N13). The test pins the answer.
- What typing `5%` into a cell formatted as currency does to its format (N9).
- That the formatter's error path is reachable from outside the library and
  reports a bad code (N3). If it is not, the check has to parse the code
  itself, and the design should be updated.
- That Alt with a digit, `.` and `,` is reported by common terminals (N6).

## Implementation steps

1. The format check, the preset table, `:format` with a preset or a raw code
   on a cell, and `:format` with no argument (N1 to N5, N10, N11).
2. Range select and whole columns and rows: `:format-cols`, `:format-rows`,
   and the selection rules (N1, N8, N13).
3. `:decimals` with its parser and refusals (N7).
4. The keys (N6, N7).
5. Docs.

Each step builds and passes the tests on its own, and each is one commit.
