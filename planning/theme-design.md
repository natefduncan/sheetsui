# Design: Themes

Status: accepted 2026-11-03 — decisions D1–D18. Implementation plan:
`planning/theme-plan.md`.

## User

A person who runs sheetui in a terminal whose colors do not suit the built-in
look: a light terminal, a transparent background, or a palette they already
like. They want to pick a theme once and have it stay, and to write their own.

## Today

Colors are hardcoded:

- `src/ui/render/viewport.rs`: column headers (even and odd), default cell
  fg and bg (`Rgb(35,33,54)`, `White`), range selection, cursor cell.
- `src/ui/render/dialog.rs`: dialog background (`on_black`).
- `src/ui/render/markdown.rs`: link color (`Blue`) and bold/italic in help.

Borders, tabs, status and command lines, and the edit box use the terminal's
own colors. A cell's fill and font color from the workbook override the
default cell colors.

The look is fixed dark. A light terminal gets the same dark cells.

## Decisions

| # | Decision |
|---|----------|
| D1 | A theme is a line-based text file in the style of `.sui`. No new dependency. |
| D2 | Version 1 has 24 slots (below), the full set rather than only what is hardcoded today. |
| D3 | A color the workbook sets on a cell wins over the theme. Theme type colors apply only where the file sets nothing. |
| D4 | Themes live in `$XDG_CONFIG_HOME/sheetui/themes/<name>.theme`, else `~/.config/sheetui/themes/`. Chosen by `--theme`, then a `theme` line in `$XDG_CONFIG_HOME/sheetui/config`, then the built-in default. `:theme <name>` switches at runtime. |
| D5 | Built-in themes: `default`, `light`, `terminal`, `dracula`, `papercolor-dark`. |
| D6 | Color values: `#RRGGBB`, the 16 ANSI names, `terminal`, and palette names. No fallback for terminals without truecolor. |
| D7 | Lines are `style <slot> fg <color> bg <color> bold true ...`. Attributes: `bold`, `italic`, `underline`, `dim`, `reverse`. |
| D8 | The theme lives in `ws.state.theme`, resolved once into ratatui `Style` values. Built-ins are `.theme` files pulled in with `include_str!` and parsed by the same parser. |
| D9 | Work is staged on a long-lived `theme` branch, one commit per stage. Each stage builds and passes tests. |
| D10 | Not upstreamed. `origin` stays as the parent. Merge upstream into `main` when it is cheap. |
| D11 | `cell.date` and `cell.negative` come from simple rules in sheetui, with a fixed precedence (below). |
| D12 | A cell with a file fill and no file font color gets black or white text, chosen from the fill's brightness. |
| D13 | Add a one-line message, shown in the outer block's bottom title and cleared on the next key. `info` and `error` style it. Theme warnings go there. Existing error dialogs stay as they are. |
| D14 | One `heading` slot applies to every heading level, on top of the level's bold (H1) or italic (H2). `default` sets `fg blue`, so H1 and H2 turn blue. |
| D15 | Row numbers and the corner cell get their own slot, `header.row`, unset in `default`. Version 1 has 25 slots. |
| D16 | Help link text is styled by `link`. `default` sets `underline true`. |
| D17 | `cell.date` uses IronCalc's number format parser (`formatter::parser`, any `Date` part), not a hand-written token scan. `General` is never a date. Amends D11. |
| D18 | Cells outside a range selection keep their colors. Today they lose them. Fixed in stage 4. |

## File format

```
# Dracula palette, from draculatheme.com (MIT)
palette comment #6272A4
palette altbg   #44475A

style cell          fg #F8F8F2  bg terminal
style header.even   fg comment  bg altbg   bold true
style cursor        fg #F8F8F2  bg altbg   bold true  underline true
style cell.number   fg #BD93F9
style cell.error    fg #282A36  bg #FF5555 bold true
```

- Blank lines and lines starting with `#` are ignored.
- `palette <name> <color>` defines a name for later lines. A palette name
  must be defined before it is used.
- `style <slot> <key> <value> ...` sets properties on one slot. Keys are
  `fg`, `bg`, `bold`, `italic`, `underline`, `dim`, `reverse`.
- Booleans are `true` or `false`, lowercase, as in `.sui`.
- A color is `#RRGGBB`, an ANSI name (`red`, `lightblue`, ...), `terminal`,
  or a palette name.
- The parser is lenient. A bad line, an unknown slot, an unknown key or an
  unknown color is skipped and recorded as a warning with its line number.
  The rest of the file still loads.
- A user theme starts from `default`. A slot the file does not set keeps
  the `default` value.

## Slots

| Group | Slots |
|-------|-------|
| Cells | `cell`, `cell.number`, `cell.string`, `cell.boolean`, `cell.date`, `cell.negative`, `cell.error` |
| Grid | `header.even`, `header.odd`, `header.row`, `cursor`, `selection` |
| Frame | `border`, `title`, `tab`, `tab.active` |
| Lines | `status`, `command`, `edit`, `info`, `error` |
| Dialogs | `dialog`, `dialog.border` |
| Help viewer | `heading`, `link`, `hint` |

- The Frame and Lines groups default to `terminal`, so the `default` theme
  looks the same as today.
- Every slot has a widget behind it. `planning/theme-plan.md` maps each
  one.

## Cell colors

For each cell, in this order:

1. Range selection and cursor styles apply as today, from `selection` and
   `cursor`.
2. If the file sets a font color on the cell, use it (D3).
3. If the file sets a fill and no font color, use black or white from the
   fill's brightness (D12). The theme's type colors do not apply.
4. Otherwise use the theme's type slot, by this precedence: `cell.error`,
   `cell.boolean`, `cell.date`, `cell.negative`, `cell.number`,
   `cell.string`, then `cell`.

Cell type comes from IronCalc's `get_cell_type`: Number, Text,
LogicalValue, ErrorValue. An empty cell is `cell`: IronCalc reports empty
cells as Number. A Number cell is `cell.negative` when its value is
below 0. It is `cell.date` when IronCalc's number format parser finds a date
part in its format, and the format is not `General` (D17). Formula cells use the type of their result. There is no
`formula` slot. Arrays and compound data use `cell`.

The theme supplies the default fill (`cell` bg) when the file sets none.

## Selecting a theme

1. `--theme <name-or-path>` on the command line.
2. A `theme <name>` line in `$XDG_CONFIG_HOME/sheetui/config`.
3. The built-in `default`.

A name is looked up in the user themes directory first, then among the
built-ins. A value with a `/` is read as a path. A theme that cannot be found
or read falls back to `default` with a one-line warning. Sheetui still starts.

`:theme <name>` loads and applies a theme now. `:theme` with no argument lists
the built-in and user themes. Parse warnings show as a one-line status
message, and the full list is available from the theme command.

## Code

- New module `src/ui/theme/`: parser, `Theme` type, directory lookup,
  built-ins in `src/ui/theme/builtin/*.theme`.
- `Theme` holds a ratatui `Style` for each slot. It is built once, stored in
  `ws.state.theme`, and read by the renderer. Nothing parses text per frame.
- Render code takes its colors from `ws.state.theme` instead of literals.
- `main.rs` resolves the theme from the flag and config, as it does for
  command history.
- Cell classification (type, negative, date, fill brightness) goes in a
  small function the viewport calls, so it can be tested alone.

## Stages

Each stage is one commit on the `theme` branch. Each builds and passes the
full test suite.

1. **Theme type, no behavior change.** Add `Theme` and `ws.state.theme`.
   Move the hardcoded colors in `viewport.rs`, `dialog.rs` and
   `markdown.rs` behind it. Add the `default` theme as a `.theme` file and
   the parser it needs. A test renders with `default` and compares every
   color to today's literals.
2. **Selection.** Add `--theme`, the `config` file, the XDG lookup, `:theme`,
   and the `light` and `terminal` built-ins. Tests for lookup order, the
   missing-theme fallback, and `:theme` listing.
3. **Frame and Lines slots.** Replace the terminal defaults for borders,
   tabs, status, command line, edit box and messages with slots. Drop any
   slot with no widget.
4. **Cell types.** Add the classifier, the type slots, the precedence, and
   the fill brightness rule. Tests for each type, the precedence order, and
   a workbook with fills. Check the result against `K12 Freight.xlsx`.
5. **Palettes and docs.** Add `dracula` and `papercolor-dark`, with the
   source of each palette in a comment. Write `docs/themes.md`: the format,
   the slot list, an example, and the truecolor note.

## Tests

- Every built-in theme parses with zero warnings.
- `default` equals the old hardcoded colors.
- Parser: each line kind, each color kind, palette names, booleans, and the
  lenient cases (bad line, unknown slot, unknown key, unknown color,
  undefined palette name), each with its line number.
- A user theme starts from `default`: an unset slot keeps the default.
- Lookup order: flag, then config, then built-in default.
- Missing or unreadable theme falls back to `default` with a warning.
- Cell colors: file font color wins; file fill with no font color gives
  black or white; each type slot; the precedence order.
- Date detection: common formats, quoted text, `[...]` sections, `General`.

## Out of scope

- Downsampling hex colors for terminals without truecolor. Use `terminal`.
- A setting to ignore the file's colors.
- A `formula` slot, or colors by anything other than result type.
- Reloading a theme file when it changes on disk.
- Theme-aware colors for `:color-cell`, `:color-rows` and `:color-cols`.
  Those set fills in the workbook and stay as they are.

## Open items

- Confirm the licenses of the Dracula and PaperColor palettes before
  shipping them (stage 5). sc-im's own theme files are not copied: its
  license is not a standard one.
