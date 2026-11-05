# Design: Freeze Panes

Status: proposed 2026-11-05 — decisions F1–F9 agreed in a design interview.
F10–F12 are defaults chosen while writing this up, and were accepted.

## User

A person with a table bigger than one screen who loses the header row, or the
key column, as soon as they scroll. They know Excel's View > Freeze Panes and
vim's `:` commands, and expect a frozen header to stay put while the rest of
the sheet scrolls under it.

## Goal

Excel parity for Freeze Panes, Freeze Top Row and Freeze First Column, with a
vim-style interface. Where Excel and vim differ on a view setting, the
decision says which wins. In every case so far they agreed.

## Flow

1. The user types `:freeze 1`. Row 1 stays at the top of the grid while they
   scroll. A line under it marks the boundary. The message line says
   `Froze rows 1-1, columns none`.
2. `:freeze 0 1` pins column A. `:freeze 2 1` pins rows 1-2 and column A.
3. Bare `:freeze` pins everything above and to the left of the cursor.
4. `:unfreeze` (or `:freeze 0 0`) clears it.
5. `:w book.xlsx` saves the freeze. Reopening in sheetui or Excel shows it.

## Decisions

| # | Decision |
|---|----------|
| F1 | `:freeze` pins above and left of the cursor. `:freeze <rows> [cols]` takes explicit counts, and `cols` defaults to 0. `:unfreeze` clears. These cover Excel's three menu items. No default key. A user can map one with `nnoremap`. |
| F2 | The freeze is saved in `.xlsx` only. `.sui` and `.csv` do not carry it, and the docs say so. Nothing is added to the `.sui` format. |
| F3 | A freeze is not undoable (Excel and vim both keep view settings out of undo). It lives on `Book` and is applied to the model at save time. See "Keeping it out of undo". |
| F4 | A freeze marks the workbook dirty, so `:q` warns about it. |
| F5 | If the frozen area leaves no room, the display clips it so at least one row and one column still scroll. The stored count is not changed, and a message says the freeze was clipped. |
| F6 | The boundary is a border line, drawn with a new theme slot `FreezeLine`, under the frozen rows and to the right of the frozen columns. It uses one screen row and one column, and only while a freeze is set. |
| F7 | Bare `:freeze` counts from row 1 and column A, not from the top of the screen. The message line reports what it pinned. |
| F8 | Movement follows Excel. See "Navigation". |
| F9 | `insert-rows` and `insert-cols` follow Excel. See "Inserting rows and columns". |
| F10 | Default: bare `:freeze` with the cursor on A1 pins nothing, changes nothing, and says so. |
| F11 | Default: each sheet has its own freeze. Counts that are negative, not whole numbers, or past the last row or column are errors that leave the freeze alone. |
| F12 | Default: `:freeze 0 0` and `:unfreeze` mean the same thing. |

## Navigation (F8)

The grid has up to four parts: the pinned corner, the pinned rows (columns
scroll), the pinned columns (rows scroll), and the scrolling area.

- The cursor moves into frozen cells freely. Moving through them never
  scrolls.
- In the scrolling area, scrolling works as it does today. The first visible
  scrolling row or column is never above or left of the freeze boundary, so a
  row cannot show twice.
- A jump to a frozen cell (`gg`, a search match, `:s` result) does not scroll.
  A jump to any other cell scrolls the least that reveals it.
- Page up and down move by the height of the scrolling area.
- Range select works across the boundary, and the highlight draws in both
  areas.

## Inserting rows and columns (F9)

With `n` frozen rows, inserting `k` rows at row `r`:

- If `r <= n` (inside the frozen area), the count becomes `n + k`.
- If `r > n`, nothing changes. Inserting at row `n + 1`, the first scrolling
  row, lands below the freeze.

Columns follow the same rule. There is no delete command today. When one is
added it should do the reverse: removing rows inside the frozen area shrinks
the count. That is a follow-up and is not part of this work.

## Too big for the screen (F5)

`:freeze 30` on a 24-row terminal pins more than fits. Excel would let the
frozen area fill the window and strand the cursor. Here the renderer shrinks
the frozen part until one scrolling row and one scrolling column remain, and
shows `Freeze clipped to fit the screen`. The stored count stays as typed, so
it saves to `.xlsx` unchanged and shows in full when the terminal grows.

## Keeping it out of undo (F3, F4)

`UserModel` has no public mutable access to its `Model`. Its
`set_frozen_rows_count` and `set_frozen_columns_count` push an ironcalc undo
step. `src/book/undo.rs` keeps its own history that counts ironcalc steps per
action. A step it did not count would put the two stacks out of step, and `u`
could then undo the freeze instead of the last edit.

So the freeze is kept out of ironcalc until it is saved:

- `Book` holds the frozen row and column count for each sheet.
- On load of an `.xlsx`, `Book` reads each sheet's counts with
  `get_frozen_rows_count` and `get_frozen_columns_count`. ironcalc already
  imports the `<pane state="frozen">` element.
- On save, `Book` round-trips the model with `Model::to_bytes` and
  `Model::from_bytes`, calls `set_frozen_rows` and `set_frozen_columns` on the
  copy, and exports that copy. Both save paths (`Book::save` and the writer
  path near `src/book/mod.rs:442`) use one helper.
- `:freeze` and `:unfreeze` set `Book.dirty = true`. Nothing is pushed to the
  undo history, so `u` never touches the freeze.

This depends on the byte round-trip losing nothing. Stage 0 checks that
first. If it fails, fall back to making the freeze an ordinary undoable
action through the `UserModel` setters (a one-step action in `undo.rs`). F3
then reads "undoable" and the docs say so.

## Viewport changes

`get_visible_rows` and `get_visible_columns` in
`src/ui/render/viewport.rs` build one sliding window from `prev_corner`. The
header row and the row-number column are built from that one range. They
change as follows:

1. Work out `frozen_rows` and `frozen_cols`, then clip them (F5) using the
   height and width of the grid.
2. Run the existing window logic over the remaining space, starting at
   `max(prev_corner, frozen)`. `prev_corner` now means the first visible
   scrolling row and column.
3. Return the frozen indices followed by the window's indices. The row
   numbers, the column header and the cell styles then work on the combined
   list. Today the header uses `prev_corner.col..=end_idx`, which would show
   hidden columns, so it must use the list instead.
4. Insert the border (F6) between the two groups. The table gets an extra
   row or a styled cell edge for it. Search highlighting, range select and
   `cell_class` already work by index, so they need no change.

## Cases

| Case | Result |
|------|--------|
| `:freeze` on A1 | Message, no change (F10) |
| `:freeze` on C5 | Rows 1-4, columns A-B pinned; message says so |
| `:freeze 1`, scroll to row 500 | Row 1 stays; 500 shows below it |
| `:freeze 1`, `gg` while scrolled | Cursor to A1, no scroll |
| `:freeze 30`, 24-row terminal | Clipped, message; stored count 30 |
| `:freeze -1`, `:freeze 1.5`, `:freeze x` | Error, freeze unchanged |
| `:freeze 5` then `:unfreeze` | Cleared; dirty set both times |
| Insert 2 rows at row 2 with 3 frozen | 5 frozen |
| Insert at row 4 with 3 frozen | 3 frozen |
| Switch sheets | Each sheet keeps its own freeze |
| Save `.xlsx`, open in Excel | Panes frozen at the same cell |
| Open an Excel file with frozen panes | Shown frozen on load |
| Save as `.sui` or `.csv` | Freeze not saved; docs say so |
| Undo after `:freeze` | Undoes the last edit, not the freeze |

## Tests

- **Round trip (stage 0):** a workbook with styles, column widths, formulas,
  colors and two sheets survives `to_bytes`/`from_bytes` with every cell
  value, style and width equal.
- **Book:** counts set and read per sheet, dirty set, `.xlsx` save then load
  keeps the freeze, an Excel-made file with panes loads frozen, the insert
  rule in both directions, undo history untouched.
- **Command:** each form in the Cases table, including every error.
- **Viewport:** visible lists with and without a freeze, scrolled and not,
  clipped, with the cursor in each of the four parts, page up and down.
- **Render:** border placement, theme slot lookup, snapshot of a frozen grid.

## Plan

Work goes on a long-lived `freeze` branch, one commit per stage. Each stage
builds, passes `cargo test` with zero warnings, and has one issue on
`natefduncan/sheetsui`.

0. **Round-trip check.** Test and spike only. Go or no-go for F3. A no-go
   switches to the undoable fallback.
1. **Book state.** Per-sheet counts on `Book`, load, save helper, dirty, and
   the insert rule. No UI.
2. **Commands.** `:freeze` and `:unfreeze`, validation, message line,
   `help` entry.
3. **Viewport.** Frozen rows and columns, the combined headers, the clip,
   navigation and range select.
4. **Boundary line.** `FreezeLine` slot, border drawing, a value in each of
   the five built-in themes.
5. **Docs.** `docs/command.md`, `docs/navigation.md`, `docs/themes.md`, and a
   note in `docs/file-formats.md` that only `.xlsx` carries the freeze.

## Open items

- Confirm `set_message` (`src/ui/mod.rs:1922`) is the right place for the
  reports in F5 and F7.
- Delete rows and columns, once they exist, must shrink the count (see F9).
