# Design: Delete Rows and Columns

Status: proposed 2026-11-05 — decisions R1–R5 agreed in a design interview.
R6–R9 are defaults chosen while writing this up, and were accepted.

## User

A person who loaded real data and needs to remove rows or columns from it.
Today they can insert but not delete, so they clear cells by hand and live
with the blank rows. They know vim's `dd` and Excel's Delete Row.

## Goal

The mirror of `insert-rows` and `insert-cols`: remove whole rows or columns,
shift what is below or to the right, and undo in one step.

## Flow

1. The user puts the cursor on a row and types `dd`. The row goes, the rows
   below move up, and the cursor stays on the same row number, now showing
   what was the next row.
2. `3dd` deletes three rows. `:delete-rows 3` does the same.
3. `dc` and `:delete-cols` do this for columns.
4. In range select mode, `d` deletes the rows the selection touches and `D`
   deletes its columns. Then the sheet returns to navigation mode.
5. `u` puts everything back, including the cursor position.

## Decisions

| # | Decision |
|---|----------|
| R1 | `:delete-rows [count]` deletes `count` rows from the cursor row down, default 1. `:delete-cols [count]` does the same for columns. In range select mode the commands delete the rows or columns the selection touches and ignore any count. |
| R2 | Navigation mode gets `dd` (delete row) and `dc` (delete column), with count prefixes: `3dd`. They appear in the generated help list. Users can remap or unmap them as any built-in key. |
| R3 | No confirmation prompt. `u` undoes a delete in one step. |
| R4 | The freeze shrinks by the number of deleted rows (or columns) that were inside it. See "Freeze". |
| R5 | The cursor follows three rules. See "Cursor". |
| R6 | Default: in range select mode `d` deletes the selected rows and `D` the selected columns. The count is ignored, and the mode returns to navigation. |
| R7 | Default: a delete that ironcalc refuses (it partly overlaps an array formula) shows the error and changes nothing: no undo step, no dirty flag, no cursor move. A count of 0, a count that is not a whole number, or a count past the end of the sheet are errors the same way. |
| R8 | Default: merged cell ranges are not moved by a delete. ironcalc does not move them on insert either. This is a known limitation, documented, and out of scope. |
| R9 | Default: deleting does not touch the search, the highlights or the sort. Highlights are rebuilt on the next draw. |

## Freeze (R4)

With `n` frozen rows, deleting `k` rows from row `r`:

- Entirely below the freeze (`r > n`): the count stays `n`.
- Entirely inside it (`r + k - 1 <= n`): the count becomes `n - k`.
- Spanning the boundary: only the deleted rows inside the freeze count. With 3
  frozen, deleting rows 2-5 removes rows 2-3 from the freeze, so the count is
  1.
- Every frozen row deleted: the count is 0 and the freeze clears.

Columns follow the same rule. One helper takes the first deleted index, the
count and the frozen count, and returns the new count, so rows, columns and
tests share it. A delete marks the workbook dirty. Undoing a delete does not
restore the freeze count, because the count lives on `Book` outside the undo
history. The same is true for undoing an insert. The docs say so.

This rule is sheetui's own. It has not been checked against Excel, and the
docs do not claim it matches.

## Cursor (R5)

- Delete the rows the cursor is in: the cursor stays at the first deleted
  index.
- Delete rows above the cursor (a range selection above it): the cursor moves
  up by the number of deleted rows before it, so it keeps pointing at the
  same data. This is the mirror of insert.
- Delete the last row that has data: the cursor stays at the same index and
  now sits on an empty row. No clamping is needed.

Undo restores the cursor to where it was before the delete. The undo history
already records that location for each action.

Columns follow the same rules.

## Formulas

ironcalc rewrites formulas when rows or columns are deleted. A reference to a
cell below the deleted rows shifts up. A single-cell reference to a deleted
row becomes `#REF!`. Conditional format ranges are shifted the same way.

What is not yet known is what happens to a range such as `SUM(A1:A10)` when
row 1 or row 10 is deleted. Excel shrinks it. ironcalc may produce `#REF!`.
A test records the actual behavior. If it differs from Excel, the docs say
so and we do not work around it.

An array formula that sits entirely inside the deleted rows is deleted with
them. One that only partly overlaps the deleted rows makes ironcalc refuse
the delete (R7).

## Technical approach

### Book

`Book` gets `delete_rows(row_idx, count)` and `delete_columns(col_idx, count)`
next to `insert_rows` and `insert_columns`. Insert loops one row at a time
inside `as_action`. Delete makes a single ironcalc call, which records one
undo step for the whole delete, so the action counts exactly one step.

The method moves the cursor by R5, applies the freeze helper (R4), sets
`dirty`, and re-evaluates. If ironcalc returns an error, none of that
happens (R7).

### Commands and keys

`src/ui/cmd.rs` gets two parsers next to `try_consume_insert_row` and
`try_consume_insert_column`, with the same count handling and error text.
`Cmd::DeleteRows` and `Cmd::DeleteColumns` run in `src/ui/mod.rs` beside
`InsertRows` and `InsertColumns`. In range select mode they read the
selection instead of the cursor and the count.

The built-in key table gets `dd` and `dc` in the `modify` group, next to `o`
and `O`, and `d` and `D` in the range-select table. `d` becomes a prefix key
like `g`, so a lone `d` waits for the second key, using the existing prefix
timeout.

## Cases

| Case | Result |
|------|--------|
| `dd` on row 5 | Row 5 gone; rows below move up; cursor on row 5 |
| `3dd` on row 5 | Rows 5-7 gone; cursor on row 5 |
| `:delete-rows 0` | Error; nothing changes |
| `:delete-rows` past the last row | Error; nothing changes |
| `dc` on column C | Column C gone; columns right move left; cursor on column C |
| Range select rows 3-6, `d` | Rows 3-6 gone; navigation mode; cursor on row 3 |
| Cursor on row 10, rows 3-6 deleted by range | Cursor on row 6, same data |
| Delete a row that partly overlaps an array formula | Error; nothing changes |
| Delete rows entirely holding an array formula | Formula deleted with them |
| `=A5` and row 5 deleted | `#REF!` |
| `=A9` and row 5 deleted | `=A8` |
| 3 frozen rows, delete rows 2-5 | 1 frozen |
| 3 frozen rows, delete row 6 | 3 frozen |
| `u` after a delete | Rows, formulas and cursor restored; freeze count unchanged |
| Delete in a merged range | Merged range not moved; known limitation |
| `u` after `dd` then `u` again | Undoes the edit before it |

## Tests

- **Book:** delete rows and columns, count and bounds errors, one undo step
  per delete, cursor rules, dirty flag, no change on error, array formula
  overlap and containment, formula rewrite for cells and for ranges (the
  characterization test), conditional format shift.
- **Freeze:** the shrink helper over a table of first index, count and frozen
  count, for rows and columns, including spanning and full deletion.
- **Command:** each form, each error, range select ignoring the count.
- **Keys:** `dd`, `dc`, `3dd`, `d` and `D` in range select, the prefix wait,
  remapping, and the generated help list.
- **Undo:** one `u` after a delete, a count prefix, and undo back to the
  save point clears the dirty flag.

## Plan

Work goes on a long-lived `delete` branch, one commit per issue, as for
freeze. Each issue builds, passes `cargo test` with zero warnings, and closes
with `Closes #N`.

1. **Delete rows.** `:delete-rows [count]` end to end: `Book` method, single
   undo step, cursor rules, dirty flag, errors, and the formula rewrite tests.
2. **Delete columns.** `:delete-cols [count]`, same rules.
3. **Freeze shrinks.** The helper and its hook in both deletes.
4. **Keys.** `dd`, `dc`, `d` and `D` in range select, and the generated help.
5. **Docs.** `docs/command.md`, `docs/navigation.md`, the freeze note about
   undo, and the merged cell limitation.

## Open items

- Confirm that ironcalc does not move merged ranges on delete (R8), by a test
  that records what it does.
- Fill down and fill right come next, and will want the same range-select
  keys. Check `d` and `D` do not clash with them.
