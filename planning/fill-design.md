# Design: Fill Down, Fill Right and Series

Status: proposed 2026-11-05 — decisions L1–L8 agreed in a design interview.
L9–L11 are defaults chosen while writing this up, and were accepted.

## User

A person who writes a formula or a first value in one cell and needs it in
the cells below or beside it. They know Excel's `Ctrl-D`, `Ctrl-R` and the
fill handle, and expect `1, 2` to continue as `3, 4`.

## Today

In range select mode `x` copies the cell where the selection started over
the whole selection, using ironcalc's `extend_to` to rewrite formula
references. It copies values and formulas only, not formats. It uses the one
start cell even for a selection several columns wide, and it never continues
a series. There is no key or command for filling without entering range
select mode first.

## Goal

Excel parity for `Ctrl-D`, `Ctrl-R` and the fill handle, with the vim style
interface sheetui already has: numeric prefixes, `:` commands, and keys that
can all be remapped.

## Flow

1. The cursor is on a cell below a formula. `<C-d>` copies the formula down
   onto the cursor cell, with its references moved. `5<C-d>` fills the cursor
   cell and the four below it from the cell above.
2. `<C-r>` does the same to the right.
3. In range select mode, `<C-d>` fills each column of the selection down from
   its top cell. `<C-r>` fills each row right from its left cell. The mode
   stays active, so `<C-d>` then `<C-r>` fills a whole block from its top
   left cell.
4. `:fill-series down 10` takes the cursor cell, or the selection, as seeds
   and extends it ten rows. Seeds `1, 2` become `3, 4, ...`.

## Decisions

| # | Decision |
|---|----------|
| L1 | Scope is copy fill and series fill. `x` in range select is unchanged. |
| L2 | Two verbs. Copy fill never makes a series (as Excel's `Ctrl-D`). Series fill is a separate command. |
| L3 | `<C-d>` is fill down and `<C-r>` is fill right, in navigation and range select mode. The navigation mode "enter range select" binding that was `<C-r>` moves to `<A-r>`. All of them can be remapped or unmapped like any built-in key. Edit mode's `<C-r>` is unchanged. |
| L4 | Copy fill follows "Copy fill" below, including the numeric prefix. |
| L5 | Commands: `:fill-down [count]`, `:fill-right [count]` and `:fill-series <down\|up\|left\|right> <count>`. See "Commands". |
| L6 | Series fill takes the selection (or the cursor cell) as the seeds and extends it by `count`. ironcalc detects the series. Seeds are never overwritten. Up and left exist only for series. `count` is required. |
| L7 | `x` in range select is kept as it is. The docs say `<C-d>` then `<C-r>` gives the same result with formats copied. |
| L8 | A fill is one undo step, asks no confirmation, overwrites what is in the target, copies formats with values, and marks the workbook dirty. |
| L9 | Default: a fill with nothing to copy from (row 1 for down, column A for right), or past the last row or column, shows a message and changes nothing. So does a fill that ironcalc refuses (a partly covered array formula). No undo step, no dirty flag. |
| L10 | Default: the fill commands do not touch the freeze, search or sort state. |
| L11 | Default: merged cells are not handled by fill. It is a known limitation, as for delete. |

## Copy fill (L3, L4)

The source is always the cell above (fill down) or to the left (fill right).

- **No selection, no prefix:** copy the cell above onto the cursor cell. The
  cursor does not move.
- **No selection, prefix `n`:** the cell above the cursor fills down over the
  cursor cell and the `n - 1` below it. The cursor does not move.
- **A range selected:** the top row (fill down) or left column (fill right)
  is the source, and the rest of the selection is filled. A prefix is
  ignored. A selection one row tall (for fill down) or one column wide (for
  fill right) uses the row above (or column left) as its source.
- **Range select mode stays active** after a fill, with the same selection.
  Excel does the same. It lets `<C-d>` then `<C-r>`, in either order, fill a
  block from its top left cell.
- **Nothing to copy from:** see L9.

Every source column (or row) is filled from its own cell, so a selection
several columns wide copies each column down from its own top cell.

## Commands (L5)

- `:fill-down [count]` and `:fill-right [count]` act as the keys do. With a
  `'<,'>` prefix, which the prompt adds in range select mode, they use the
  selection and ignore `count`. Without one they use the cursor and `count`
  (default 1), so `:fill-down 5` is `5<C-d>`.
- `:fill-series <direction> <count>` takes the selection as seeds, or the
  cursor cell. A missing or bad direction or count is an error with a usage
  hint, as `insert-rows` shows for a bad argument.

## Series (L6)

ironcalc's fill reads each seed column (or row) and looks for:

- two or more numbers with a constant step (`1, 2` gives `3, 4`; `10, 8`
  gives `6, 4`);
- text ending in a number (`Item 1, Item 2`);
- month and day names, keeping their case.

A single seed number, or anything else it does not recognize, is copied, and
a block of several seeds repeats. Formulas get their references moved for
each target, and formats are copied. This is the same behavior Excel's fill
handle has when you drag from one cell versus two.

## Technical approach

### Book

`Book` gets `fill_down`, `fill_right` and `fill_series` next to
`extend_to`. Each calls ironcalc's `auto_fill_rows` or `auto_fill_columns`
through `Book::change`, which records exactly one undo step, so the action
counts one step. The method also sets `dirty` and re-evaluates. On an error
from ironcalc nothing else happens (L9).

ironcalc's functions take a source area and a target row or column beyond it.
Copy fill passes a source one row tall (or one column wide), so the seed
count is one and no series is detected. This is the thing to check first, in
a test: a single seed that looks like it could start a series, such as a
number, `Item 1`, or `Jan`, must copy and not continue. If it does continue,
copy fill falls back to `extend_to` plus a style copy, and the series fill
still uses ironcalc.

### Keys

`FillDown` and `FillRight` are new handlers in both key tables, bound to
`<C-d>` and `<C-r>`. The navigation `EnterRangeSelect` moves from `<C-r>` to
`<A-r>`. The handlers appear in the generated key lists, so the help updates
itself. In range select they do not call `exit_range_select_mode`.

### Commands

`src/ui/cmd.rs` gets three parsers beside the insert and delete ones, with
`Cmd::FillDown`, `Cmd::FillRight` and `Cmd::FillSeries`. They read the range
prefix as `sort` and `s` do.

## Cases

| Case | Result |
|------|--------|
| `<C-d>` on B3, B2 is `=A2+1` | B3 becomes `=A3+1` |
| `5<C-d>` on B3 | B3-B7 filled from B2; cursor stays on B3 |
| `<C-d>` on row 1 | Message; nothing changes |
| `<C-r>` on column A | Message; nothing changes |
| Select B2:D5, `<C-d>` | B3:D5 filled from B2, C2, D2; still in range select |
| Select B2:D5, `<C-d>` then `<C-r>` | Whole block is B2 stamped, formats included |
| Select one row, `<C-d>` | Row above fills it |
| `:fill-down 5` | Same as `5<C-d>` |
| `:'<,'>fill-down 5` | Selection used; the 5 is ignored |
| Seeds `1, 2`, `:fill-series down 3` | `3, 4, 5` below them |
| Seeds `1`, `:fill-series down 3` | `1, 1, 1` below it |
| Seeds `Item 1, Item 2` | `Item 3, Item 4, ...` |
| `:fill-series down` | Error with a usage hint |
| Fill past the last row | Error; nothing changes |
| Partly covered array formula in the target | Error; nothing changes |
| `u` after any fill | Undoes the whole fill, one step |
| `<A-r>` in navigation mode | Enters range select, as `<C-r>` did |
| Edit mode `<C-r>` | Unchanged |
| `x` in range select | Unchanged: stamps the start cell |

## Tests

- **Copy source (first):** a single seed that could start a series copies.
- **Book:** each fill, formats copied, one undo step, dirty flag, no change
  on an error, array formula overlap, row 1 and column A, past the end.
- **Series:** numbers up and down, suffixed text, month and day names with
  case kept, single seed copying, repeated seed blocks, up and left.
- **Command:** each form and each error, range prefix, count ignored under a
  selection, `:fill-series` usage hints.
- **Keys:** `<C-d>`, `<C-r>` and `5<C-d>` in both modes, range select stays
  active, `<A-r>` enters range select, `<C-r>` no longer does in navigation
  mode, remapping, and the generated help list.
- **Regression:** `x` still stamps the start cell.

## Plan

Work goes on a long-lived `fill` branch, one commit per issue.

1. **Fill down at the cursor.** `:fill-down [count]` and `<C-d>`, including
   the check that a single seed copies. Go or no-go for reusing ironcalc.
2. **Fill right at the cursor.** `:fill-right [count]` and `<C-r>`, and move
   navigation range select to `<A-r>`.
3. **Fill a selection.** `<C-d>` and `<C-r>` in range select, the `'<,'>`
   commands, range select staying active.
4. **Series.** `:fill-series`.
5. **Docs.** `docs/command.md`, `docs/navigation.md`, `docs/visual.md`
   (including the `<C-r>` change), and a note on `x` versus `<C-d>`.

## Open items

- The `<C-r>` move is a visible change for anyone used to it. The docs and
  the generated key list show the new key.
- After fill, column width auto-fit (`:width`) is next. `<C-h>` and `<C-l>`
  already change a column's width.
