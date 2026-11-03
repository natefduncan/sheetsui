# Plan: Themes

Implements `planning/theme-design.md` (D1–D18). Work goes on a long-lived
`theme` branch, one commit per stage (D9). Each stage builds, passes
`cargo test` with zero warnings, and is tracked by one GitHub issue on
`natefduncan/sheetsui`.

## Findings from the code

The design was checked against the code on 2026-11-03. Things it got wrong
or left open:

1. **No message line exists.** Every message and error opens a dialog
   (`enter_dialog_mode`, `src/ui/mod.rs:1184`). The nearest thing to a
   status line is the outer block's bottom titles, mode and `row,col`
   (`src/ui/render/mod.rs:126-140`). Resolved by D13.
2. **Links are unstyled.** The `Color::Blue` in `src/ui/render/markdown.rs:81`
   is the H3+ heading color. H1 is bold, H2 italic. `handle_link_tag` only
   records the destination. Resolved by D15, D16.
3. **Help text is styled at parse time.** `Markdown::from_str` builds and
   caches the styled `Text` when a dialog opens, so the theme must be passed
   in then. `:theme` runs from command mode, so no dialog is open when the
   theme changes.
4. **Empty cells type as Number.** IronCalc's `get_cell_type` returns
   `Number` for a missing cell and for `Cell::EmptyCell`. The classifier
   must check for an empty value first, or `cell.number` paints every empty
   cell.
5. **Range-select drops the default fill.** With a range set, cells outside
   it get no fg/bg at all (`src/ui/render/viewport.rs:227-234`). Stage 1
   keeps this. Stage 4 fixes it (D18).
6. **The active tab is `REVERSED`**, ratatui's `Tabs` default. `tab.active`
   defaults to `reverse true`.
7. **Header underline is on the row** (`viewport.rs:187`), so it also covers
   the corner cell. It moves into `header.even`, `header.odd` and
   `header.row`.
8. **`TextPrompt` has no style setter** (tui-prompts 0.6.1). The `command`
   slot is applied with `buf.set_style(rect, ..)` after the prompt renders.
9. **`reset_text_area` builds a new `TextArea`** on every edit
   (`src/ui/mod.rs:1318`), so the `edit` and `border` styles go there.
10. **`K12 Freight.xlsx` is not in the repo.** Stage 4 checks against
    `examples/test.xlsx`, and `K12 Freight.xlsx` if it is at hand.

## Slots (25)

| Group | Slots |
|-------|-------|
| Cells | `cell`, `cell.number`, `cell.string`, `cell.boolean`, `cell.date`, `cell.negative`, `cell.error` |
| Grid | `header.even`, `header.odd`, `header.row`, `cursor`, `selection` |
| Frame | `border`, `title`, `tab`, `tab.active` |
| Lines | `status`, `command`, `edit`, `info`, `error` |
| Dialogs | `dialog`, `dialog.border` |
| Help viewer | `heading`, `link`, `hint` |

Widget behind each non-cell slot:

| Slot | Widget |
|------|--------|
| `header.even`, `header.odd` | Column header cells, `Viewport::to_table` |
| `header.row` | Row number cells and the corner cell |
| `cursor`, `selection` | `compute_cell_colors` overlays |
| `border` | Outer block, viewport block, edit box block (`border_style`) |
| `title` | Outer block title (file name), viewport title (sheet name), dialog title |
| `tab`, `tab.active` | `Tabs::style`, `Tabs::highlight_style` |
| `status` | Outer block bottom titles: modality and `row,col` |
| `command` | Command `TextPrompt` rect, applied after render |
| `edit` | `TextArea::set_style` in `reset_text_area` |
| `info`, `error` | The message line (D13) |
| `dialog` | Dialog block and paragraph style |
| `dialog.border` | Dialog block `border_style` |
| `heading` | Markdown headings, all levels (D14) |
| `link` | Markdown link text (D16) |
| `hint` | "ALT-h to toggle help dialog", dialog bottom title |

## `default.theme`

Exactly today's look, with three exceptions: H1/H2 headings turn blue
(D14), help links are underlined (D16), and the blank corner cell above
the row numbers loses its underline. The corner and the row numbers both
use `header.row`, which is unset.

```
style cell         fg white  bg #232136
style header.even  fg white  bg #393D47  bold true  underline true
style header.odd   fg black  bg #A5A9A0  bold true  underline true
style cursor       fg white  bg #393D47
style selection    fg black  bg lightblue
style tab.active   reverse true
style dialog       bg black
style heading      fg blue
style link         underline true
```

The header row's `.underlined()` call is removed, so the underline comes
only from the slots.

All other slots are unset (`Style::default()`), so they inherit the
terminal's colors as today.

## Code shape

```
src/ui/theme/
  mod.rs        Slot enum, Slot::ALL, name <-> Slot, Theme, Theme::default()
  color.rs      parse_color(&str, &palette) -> Option<Color>; shared with map_color
  parse.rs      parse(text, base: &Theme) -> (Theme, Vec<Warning>)
  lookup.rs     config_dir, read_config, resolve, list   (stage 2)
  builtin/*.theme
src/ui/render/cell_class.rs   classify, contrast_fg, cell_style   (stage 4)
```

- `Theme { name: String, styles: [Style; Slot::COUNT] }`, `get(Slot) -> Style`.
- `Warning { line: usize, msg: String }`.
- `Theme::default()` parses `builtin/default.theme` onto an all-default base.
  The parse is done once, for example through `std::sync::OnceLock`.
- Color semantics: `terminal` maps to `Color::Reset`, which clears an inherited
  color. An unset `fg`/`bg` stays `None`, which inherits. `<attr> true` uses
  `add_modifier`. `<attr> false` uses `remove_modifier`.
- `AppState` gains `pub theme: Theme`, `pub theme_warnings: Vec<Warning>` and
  `pub message: Option<Message>` (stage 2).

## Stages

### Stage 1: Theme type, no behavior change

1. Add `src/ui/theme/` with `Slot`, `Theme`, `Warning`, `color.rs` and `parse.rs`,
   and `builtin/default.theme` as above.
2. Move color-name parsing out of `viewport::map_color` into `theme/color.rs`.
   `map_color` keeps its signature and delegates, so `test_color_mapping` still
   passes.
3. Add `AppState::theme`, defaulting to `Theme::default()`.
4. `Viewport::new(book, range, theme: &Theme)`. Header, row-number, corner,
   cell default, selection and cursor colors come from slots. The
   range-select quirk is kept.
5. `Dialog::new` takes the `dialog`, `dialog.border` and `title` styles from the
   theme. Pass `&Theme`, or a small `DialogStyle`.
6. Markdown: add `Markdown::from_str_themed(input, &Theme)`. `from_str` calls
   it with `Theme::default()`. Headings use `level modifier.patch(heading)`.
   Text inside `Tag::Link`..`TagEnd::Link` is patched with `link`. Thread the
   theme through `help::to_widget` and `enter_dialog_mode` call sites.
7. Tests:
   - `default.theme` parses with zero warnings, and each set slot equals the old literal.
   - Parser: comment/blank lines, `palette`, `style`, every key, `#RRGGBB`,
     every ANSI name, `terminal`, palette names, `true`/`false`. Lenient cases,
     each with its line number: malformed line, unknown slot, unknown key,
     unknown color, palette used before definition, bad boolean.
   - A theme on a `default` base keeps unset slots.
   - `TestBackend` frame on an empty workspace, compared to the old literals:
     cursor cell, even header, odd header, plain cell, a dialog cell.

### Stage 2: Selection

1. `theme/lookup.rs`, pure functions with directories passed in:
   - `config_dir() -> Option<PathBuf>`: `$XDG_CONFIG_HOME/sheetui`, else
     `$HOME/.config/sheetui`.
   - `read_config(dir) -> Option<String>`: the value of the first `theme <name>` line.
   - `resolve(flag, config, themes_dir) -> (Theme, Vec<Warning>)`: a value
     containing `/` is a path. Otherwise try `themes_dir/<name>.theme`, then a
     built-in. If nothing is found or the file is unreadable, use `default`
     plus a warning.
   - `list(themes_dir) -> Vec<(String, Source)>`, where `Source` is `Builtin` or `User`.
2. Built-in registry: `&[(&str, &str)]` of name to `include_str!`. Add
   `light.theme`, and `terminal.theme` (everything `terminal`, cursor and
   selection by `reverse true`).
3. `main.rs`: add `--theme <name-or-path>` to `Args` and resolve after the
   workspace is built. Set `ws.state.theme` and report warnings to the
   message line.
4. Message line (D13): `Message { level: Info | Error, text }`. It is drawn as
   a bottom title of the outer block, before the modality, in the `info` or
   `error` slot. It is cleared at the top of `handle_input` on a key press.
5. `:theme`: `Cmd::Theme(Option<&str>)` with `try_consume_theme`.
   - With a name: resolve and apply. On success with no warnings the message
     is `theme <name>`. With warnings it is `theme <name>: N warnings (:theme to list)`.
     An unknown name gives an `error` message and keeps the current theme.
   - With no name: a dialog lists built-in and user themes, marks the current
     one, and shows the last load's warnings with line numbers.
6. Help: add `theme` to the command help and the help topic list.
7. Tests: lookup order, user theme shadowing a built-in, path values, the
   missing and unreadable fallbacks, `read_config` leniency, `cmd::parse` for
   `theme`/`theme x`, and the listing contents. The message is cleared on the
   next key.

### Stage 3: Frame and line slots

Apply `border`, `title`, `tab`, `tab.active`, `status`, `command`, `edit`,
`hint` and `dialog.border` per the widget table. `info` and `error` are
already used by the message line. All default unset except `tab.active`, so
the look does not change. Test: a theme setting a distinct fg per slot,
drawn on a `TestBackend`, with one known cell of each widget checked.

### Stage 4: Cell types

1. `Book::get_cell_class_inputs(&Address) -> (CellValue, CellType, String)`
   returning the value, type and number format, using `get_cell_value_by_index`,
   `get_cell_type` and `get_cell_style`.
2. `cell_class.rs`:
   - `classify(value, ty, num_fmt) -> Slot`: empty value gives `Cell`.
     Otherwise the precedence is `ErrorValue` → `cell.error`, `LogicalValue` →
     `cell.boolean`, Number and date format → `cell.date`, Number < 0 →
     `cell.negative`, Number → `cell.number`, Text → `cell.string`, else `cell`.
   - `is_date_format(fmt)`: false for `general` (case-insensitive). Otherwise
     run IronCalc's `formatter::parser::Parser::new(fmt)`, `.parse()`, then
     `parts.iter().any(ParsePart::is_date)` (D17).
   - `contrast_fg(r, g, b) -> Color`: relative luminance
     `0.2126 R + 0.7152 G + 0.0722 B` on 0–255. Above 140 gives `Black`, else `White`.
   - `cell_style(theme, file_font: Option<Color>, file_fill: Option<Color>, class, in_selection, is_cursor) -> Style`:
     - Start with `cell.patch(type slot)`.
     - If there is a file fill, use it as bg.
     - If there is a file font color, use it as fg. Else, if there is a file
       fill, use `contrast_fg(fill)` as fg and skip the type slot fg.
     - Then patch `selection` if selected, then `cursor`.
     - File bold and italic are still applied after this by `compute_cell_style`.
3. `Viewport::compute_cell_colors` calls `cell_style`. Cells outside a range
   keep their style (D18).
4. Tests: each type, empty and styled-empty cells, each precedence step, file
   font color wins, dark and light fills give white and black, selection and
   cursor overlays. Date formats: `yyyy-mm-dd`, `m/d/yy`, `h:mm:ss`,
   `"day "0`, `[Red]0.00`, `General`, `0.00`, `#,##0`. Check by eye on
   `examples/test.xlsx`.

### Stage 5: Palettes and docs

1. `dracula.theme` (MIT, draculatheme.com) and `papercolor-dark.theme`
   (PaperColor, NLKNguyen/papercolor-theme). Confirm each license first and
   cite source and license in a header comment.
2. `docs/themes.md`: selecting a theme, the file format, the slot table,
   an annotated example, the truecolor note, and cell color precedence.
   Link it from `docs/index.md` and add `:theme` to `docs/command.md`.
3. Test: every built-in parses with zero warnings.

## Risks

- Stage 1's markdown threading touches every `enter_dialog_mode` caller.
  Keeping `from_str` keeps the existing markdown tests unchanged.
- Stage 4 adds two IronCalc lookups and a format parse per visible cell per
  frame. This is fine at terminal sizes. If it shows up in profiling, cache
  `is_date_format` per format string.
- The `Color::Reset` vs `None` difference is easy to get wrong. The stage 1
  parser tests pin it down.
