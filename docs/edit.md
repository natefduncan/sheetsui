# Edit Mode

You enter Edit mode by hitting `e` or `i` while in navigation mode. Type
what you want into the cell.

Starting with:

* `=` will treat what you type as a formula.
* `$` will treat it as us currency.

Typing a number will treat the contents as a number. While typing non-numeric
text will treat it as text content.

<aside>We do not yet support modifying the type of a cell after the fact. We
may add this in the future. To change how a number is shown, as currency, a
percent or a date, use `:format`. See [Number formats](./command.md#number-formats). To make it bold, italic, underlined or centered, or to color its text, use `:style`. See [Text styles](./command.md#text-styles).</aside>

For the most part this should work the same way you expect a spreadsheet to
work.

* `Enter` will update the cell contents.
* `Esc` will cancel editing the cell and leave it unedited.
* `Ctrl-p` will paste the range selection if it exists into the cell.

`Ctrl-r` will enter range select mode when editing a formula. You can navigate
around the sheet and hit space to select that cell in the sheet to set the
start of the range. Navigate some more and hit space to set the end of the
range.

You can find the functions we support documented here:
[ironcalc docs](https://docs.ironcalc.com/functions/lookup-and-reference.html)

