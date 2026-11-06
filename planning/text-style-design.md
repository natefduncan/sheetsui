# Design: Text Styling

Status: proposed 2026-11-06 — decisions T1–T14. T1–T5, T7, T9 and T10 were
agreed in a design interview; the rest are my defaults and are marked
"(default)" for review. Related: `planning/number-format-design.md` (the area,
count and undo rules follow it), `planning/col-width-design.md`,
`planning/undo-design.md`, `planning/keys-design.md` (why the keys are Alt).

## User

A person who wants a header row bold, a total underlined, a column centered or
some text in red, the way Excel's Ctrl+B, Ctrl+I, Ctrl+U, the alignment buttons
and the font color picker do. They want it on a cell, a selection, or a whole
column or row, and to undo it.

## Today

- `Book::set_cell_style(&[("font.b", "true")], area)`, `set_col_style` and
  `set_row_style` exist and are how `color-cell` and `:format` work. ironcalc's
  `update_range_style` accepts `font.b`, `font.i`, `font.u`, `font.strike`,
  `font.color`, `alignment.horizontal` and `alignment` (clears alignment).
- `.sui` and `.xlsx` already save and load all of these (`docs/SUI_FORMAT.md`).
- The viewport (`src/ui/render/viewport.rs`, `compute_cell_style`) already draws
  bold and italic, and the font color via `cell_style` in `cell_class.rs`.
- Not drawn: underline, strikethrough and horizontal alignment. A file that
  has them shows plain text.
- No command or key sets any of them. The only visual command is `color-cell`
  (fill).
- Terminals cannot send Ctrl+Shift chords, and `<C-i>` is Tab, so Excel's
  Ctrl+I is out. `<A-h>` (help) and `<A-r>` (range select) are taken.

## Decisions

| # | Decision |
|---|----------|
| T1 | Scope: bold, italic, underline, strikethrough, horizontal alignment, font color, and a clear command. Borders, font size and name, wrap text, vertical alignment and merge are out of scope. |
| T2 | `:style <attr>...` styles the cursor cell, or the selection in range select mode. `:style-cols [count] <attr>...` and `:style-rows [count] <attr>...` style whole columns and rows going right or down from the cursor, over every cell as `:format-cols` does. In range select mode they apply to every column or row the selection touches and ignore `count`. |
| T3 | An `<attr>` is a word: `bold`, `italic`, `underline`, `strike` (toggles), `nobold`, `noitalic`, `nounderline`, `nostrike` (clear), `left`, `center`, `right`, `general` (alignment), `color=<color>` and `nocolor`. Several may be given: `:style bold underline color=red`. `<color>` uses the parser of `color-cell`. An unknown word is the error `Unknown style: <word>`, and nothing changes. |
| T4 | A toggle (`bold`, `italic`, `underline`, `strike`) is decided once, from the cursor cell in navigation mode and the first selected cell in range select mode, and applied to the whole area. If that cell has the attribute the area loses it, otherwise the area gets it. This holds for `style-cols` and `style-rows` too: the cursor cell decides. A mixed selection ends up uniform. |
| T5 | Short aliases: `:bold`, `:italic`, `:underline`, `:strike` are `:style` with that one toggle. `:align left\|center\|right\|general` is `:style` with that alignment. `:font-color <color>` is `:style color=<color>`. They take no count and no `-cols` form; use `:style-cols`. |
| T6 | (default) Alignment `general` clears the horizontal alignment, so text goes left and numbers right as today. `left`, `center` and `right` set it. Alignment is horizontal only. |
| T7 | `:clear-style` clears everything visual on the area, including the number format: font bold, italic, underline, strike, font color, fill, alignment and number format go back to the default, as Excel's Clear Formats does. It does not touch cell values. It has `-cols` and `-rows` forms with the same count rules as T2: `:clear-style-cols [count]`, `:clear-style-rows [count]`. |
| T8 | (default) Applying a style covers every cell in the area, including empty cells, as `:format` does (number-format N5). Whole columns and rows replace the matching attribute of cells that already have their own style and leave the other attributes alone, so `:style-cols bold` does not drop a cell's fill. The first step pins how ironcalc does this (see Facts to check). |
| T9 | Keys, in navigation and range select mode: `<A-b>` bold, `<A-i>` italic, `<A-u>` underline, `<A-s>` strike (all toggles by T4). They are remappable and show in the help. |
| T10 | Alignment keys exist too: `<A-l>` left, `<A-e>` center, `<A-;>` right (Excel's Ctrl+R is taken by fill right). A key sets the alignment and does not toggle it. There is no key for `general`; use `:align general`. |
| T11 | (default) Each command and key is one undo step however many cells it changes, and says what it did: `Bold: 12 cells`, `Not bold: column C`, `Style bold underline: 3 columns`, `Align center: 4 cells`, `Cleared style: 12 cells`. |
| T12 | (default) Rendering: underline and strikethrough are drawn with ratatui's `UNDERLINED` and `CROSSED_OUT` modifiers beside the existing bold and italic. Horizontal alignment is drawn by aligning the cell text in its column. These modifiers go on the cell style so the theme's cursor, selection and search slots still patch on top. |
| T13 | (default) A style never changes a column width or the displayed text. `gw` still fits to the widest displayed value, and a bold value is not measured wider. |
| T14 | (default) Typing a value into a styled cell keeps its style. Copy and paste, and fill, already carry the style with them; the first step checks that and the docs record what they do. |

## Out of scope

- Borders, font size and name, wrap text, vertical alignment, merge.
- A format painter and styles saved under a name.
- Underline variants (double) and strikethrough or underline colors.
- Changing the theme's own cell slots. User styles sit on top of the theme.
- Terminals that cannot draw underline or strikethrough: the attribute is
  saved and shown in the cell's style, and the terminal does what it does.

## The commands

```
:style bold                    toggle bold on the cursor cell or selection
:style bold underline          toggle both, each decided by the cursor cell
:style color=red nobold        font color red, bold off
:style-cols italic             italic over the cursor column
:style-cols 3 center           three columns, centered
:style-rows 2 bold             two rows, going down
:bold  :italic  :underline  :strike
:align right
:font-color blue
:clear-style
:clear-style-cols 2
```

Errors, each leaving everything unchanged: no attribute, an unknown word (T3),
a bad color, a `count` of 0 or one past the last column or row, and a
non-numeric count. Usage hints: `Usage: style <attr>...`,
`Usage: style-cols [count] <attr>...` (and rows), `Usage: align
<left|center|right|general>`. A color that is repeated or two alignments in one
command is an error rather than a last-one-wins.

## Parsing

`style`, `style-cols` and `style-rows` parse the attribute words once into a
list of `(path, value)` pairs for `set_cell_style`, after the toggle state is
resolved against the deciding cell (T4). The attribute parser is one function
in `cmd.rs` with its own unit tests, so the aliases (T5) are thin wrappers and
the keys build the same list without parsing.

`style-cols [count]` follows the count rule of `format-cols` (number-format
N11): a count comes first and only when an attribute follows, so
`:style-cols 3 bold` is three columns. No attribute word is a number, so there
is no ambiguity and no raw form.

## Book layer

- A `StyleChange` made of the four optional toggles, an alignment, a color and
  a clear flag, applied by one `Book::apply_style(area, change)` that resolves
  toggles against the deciding cell and calls the existing `set_cell_style`,
  `set_col_style` or `set_row_style` inside one `as_action`.
- `Book::cell_has_style(addr, attr)` for T4, from `get_cell_style`.
- `clear_style` writes the default for each path, including `num_fmt` =
  `General`; whole column and row forms use `set_col_style` and `set_row_style`.

## Keys

| Mode | Keys | Does |
|------|------|------|
| Navigation, range select | `<A-b>`, `<A-i>`, `<A-u>`, `<A-s>` | toggle bold, italic, underline, strike |
| Navigation, range select | `<A-l>`, `<A-e>`, `<A-;>` | align left, center, right |

Check the keys are not taken: `<A-h>` and `<A-r>` are, and `<A-1>` to `<A-6>`,
`<A-0>`, `<A-.>` and `<A-,>` are the number format keys. The key tables, the
handler enums and the help text get new rows in both modes.

## Undo and messages

One `change()` group per command or key, as fill and format do, so one `u`
undoes it. Messages are in T11. A refused command shows its error and changes
nothing, including at a later cell of a range.

## Testing

Book layer, with no terminal:

- Each attribute sets the matching ironcalc path on a cell, a range, a whole
  column and a whole row, and round-trips through `.sui` and `.xlsx`.
- Toggle (T4): cursor cell off → area on; cursor cell on → area off; a mixed
  selection ends uniform and the deciding cell is the first, not any other.
- `style-cols` does not drop an existing fill or number format (T8).
- Empty and text cells accept a style, and a value typed into a styled empty
  cell keeps it (T14).
- Several attributes in one command are one undo step; an error at a later
  attribute leaves everything unchanged.
- `clear-style` resets every path including `num_fmt` and leaves values alone.
- A style never changes a column width (T13).

UI layer:

- `:style`, `:style-cols`, `:style-rows`, the aliases and `:clear-style` and
  its `-cols` and `-rows` forms do what they say, with the message line text
  (T11) and the usage errors, including an unknown word and a repeated color.
- The keys work in both modes, and can be remapped and unmapped.
- Count rules: `style-cols 3 bold`, a count of 0, a count past the last column.
- Range select: `style-cols` applies to the touched columns and ignores the
  count; `:style` and `:bold` apply to exactly the selection.
- Rendering: a bold, italic, underlined and struck cell carry the matching
  modifiers; a centered and a right aligned cell place their text; the cursor,
  selection and search slots patch on top and keep the modifiers.

## Documentation

- `docs/command.md`: `style`, `style-cols`, `style-rows`, the aliases and
  `clear-style` in the command list, and a "Text styles" section with the
  attribute table, the toggle rule, clear-style and what copy, fill and typing
  do (T14).
- `docs/navigation.md` and `docs/visual.md`: the style keys; the key lists in
  the help follow from the tables.
- `docs/keys.md`: add the style keys to the note about Alt.
- `docs/edit.md`: point to `:style` where it says how to change how a cell
  looks, if it does.

## Facts to check while building

None of these has been checked by running anything:

- How ironcalc applies a full-column or full-row `update_range_style` over
  cells that already have a style: whether it merges one path or replaces the
  whole style (T8). The test pins the answer; if it replaces, `style-cols`
  must read each cell and write the whole style.
- Whether `update_range_style` with `font.u` and `font.strike` accepts
  `"true"` and `"false"` as `font.b` does, and what `alignment.horizontal`
  `"general"` stores.
- Whether `alignment` with an empty string clears the alignment only, or the
  whole alignment struct including wrap and vertical (relevant to T6 and T7).
- Whether the table `Cell` widget in ratatui 0.30 supports per-cell
  alignment, or the text has to be padded to the column width by us. If we pad,
  wide characters count as 2, as autofit does.
- Whether common terminals report `<A-b>`, `<A-i>`, `<A-u>`, `<A-s>`, `<A-l>`
  `<A-e>` and `<A-;>` (macOS Terminal.app treats Option as a character unless
  configured; see `docs/keys.md`).
- What typing, copy, paste and fill do to the style of the target (T14).

## Decided questions

- Right alignment is `<A-;>` (T10).
- A toggle that turns a style off says so: `Not bold: 12 cells` (T11).
- Clear-style has no key; it is a command only. A user can map one.

## Implementation steps

1. Render underline, strikethrough and alignment from a file's styles, and
   check the facts above. This is visible with a `.xlsx` that has them and
   needs no commands.
2. The attribute parser, the `StyleChange` and `Book::apply_style`; `:style`
   and the single-word aliases on a cell and a selection (T2–T5, T11).
3. `:style-cols` and `:style-rows`, and the range select rules (T2, T8).
4. `:align`, `:font-color` and the alignment keys (T5, T6, T10).
5. `:clear-style` with its `-cols` and `-rows` forms (T7).
6. The toggle keys (T9).
7. Docs.

Each step builds and passes the tests on its own, and each is one commit.
