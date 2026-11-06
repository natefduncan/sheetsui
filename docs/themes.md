# Themes

A theme sets the colors and text styles of the sheetui screen.

## Choosing a theme

sheetui picks a theme in this order:

1. `--theme <name-or-path>` on the command line.
2. A `theme <name>` line in the [config file](./config.md),
   `$XDG_CONFIG_HOME/sheetui/config` (by default
   `~/.config/sheetui/config`). If the file has several `theme` lines, the last
   one is used, as `:theme` does at runtime.
3. The built-in `default`.

You can also switch while sheetui runs with `:theme <name>`. `:theme` with no
name lists the themes and the warnings from the last theme load.

A name is looked up first in the user themes directory,
`$XDG_CONFIG_HOME/sheetui/themes/<name>.theme`, then among the built-in
themes. A user theme with the same name as a built-in one wins. A value that
contains a `/` is read as a path to a theme file.

If a theme cannot be found or read, sheetui uses `default` and shows a
message. It still starts.

The built-in themes are `default`, `light`, `terminal`, `dracula` and
`papercolor-dark`.

## Terminal colors

`#RRGGBB` colors need a truecolor terminal. On a terminal without truecolor,
use `terminal` or the ANSI color names, so the terminal picks the exact
shades. The `terminal` theme uses only the terminal's own colors.

## File format

A theme is a plain text file. Each line is one of these:

* A comment: a line that starts with `#`. Blank lines are ignored.
* `palette <name> <color>` gives a color a name. Use the name in any later
  line. A palette line must come before the lines that use it.
* `style <slot> <key> <value> ...` sets the style of one slot.

A style line takes pairs of a key and a value:

* `fg` and `bg` take a color.
* `bold`, `italic`, `underline`, `dim` and `reverse` take `true` or `false`.
  `true` adds the attribute and `false` removes it.

A color is `#RRGGBB`, a palette name, `terminal`, or an ANSI name: `black`,
`red`, `green`, `yellow`, `blue`, `magenta`, `cyan`, `white`, `gray`,
`darkgray`, `lightred`, `lightgreen`, `lightyellow`, `lightblue`,
`lightmagenta` and `lightcyan`. `terminal` means the terminal's own color.

A slot that a theme does not set keeps the `default` theme's style.

The parser is lenient. A bad line, or a bad part of a line, is skipped and
recorded as a warning with its line number, and the rest of the file still
loads. Warnings are for a malformed line, an unknown slot, key or color, a
palette name used before it is defined, and a boolean that is not `true` or
`false`. The count shows in the message line, and `:theme` lists them.

## Slots

There are 28 slots.

Cells:

* `cell`: the default style of a cell, including its fill.
* `cell.number`: number cells.
* `cell.string`: text cells.
* `cell.boolean`: TRUE and FALSE cells.
* `cell.date`: number cells with a date or time format.
* `cell.negative`: number cells below zero.
* `cell.error`: cells that show an error.

Grid:

* `header.even`, `header.odd`: column headers, alternating.
* `header.row`: row numbers and the corner cell.
* `cursor`: the cell under the cursor.
* `selection`: the cells of a visual selection.
* `search`: cells that match the last search, while `hlsearch` is on.
* `search.current`: the match under the cursor. It is applied on top of
  `search`.

Frame:

* `border`: the borders of the sheet, the outer frame and the edit box.
* `title`: the file name, the sheet name and dialog titles.
* `tab`: the sheet tabs.
* `tab.active`: the current sheet tab.
* `freeze.line`: the line under frozen rows and right of frozen columns. Drawn only while a freeze is set.

Lines:

* `status`: the mode and the `row,col` at the bottom of the frame.
* `command`: the command line.
* `edit`: the cell editor.
* `info`: informational messages.
* `error`: error messages.

Dialogs:

* `dialog`: the body of a dialog.
* `dialog.border`: the border of a dialog.

Help viewer:

* `heading`: headings.
* `link`: links.
* `hint`: the hint at the bottom of a dialog.

## Cell color precedence

For each cell, sheetui builds the style in this order:

1. The `cell` slot, patched with the type slot of the cell. The type is the
   first match of: error, boolean, date, negative, number, string. An empty
   cell is just `cell`. A formula cell has the type of its result.
2. If the file gives the cell a fill, the fill is the background.
3. If the file gives the cell a font color, it is the foreground. If the file
   gives a fill but no font color, the foreground is black or white, whichever
   contrasts with the fill, and the type slot's foreground does not apply.
4. In a visual selection, `selection` is applied, then `cursor` on the cursor
   cell.
5. A cell that matches the last search gets `search`, and then, if the cursor
   is on it, `search.current`.
6. Bold and italic from the file are applied last.

## Example

```
# my.theme: save as ~/.config/sheetui/themes/my.theme, then run :theme my
palette ink    #1B1D26
palette paper  #E8E6DF
palette accent #D8A657

style cell          fg paper   bg ink
style cell.number   fg #8EC07C
style cell.negative fg #FB4934
style header.even   fg ink     bg accent  bold true
style header.odd    fg ink     bg paper   bold true
style cursor        reverse true
style heading       fg accent  bold true
```

Slots that are not listed keep the `default` theme.

## Credits

`dracula` uses the Dracula palette, https://draculatheme.com (MIT License).
`papercolor-dark` uses the PaperColor palette,
https://github.com/NLKNguyen/papercolor-theme (MIT License).
