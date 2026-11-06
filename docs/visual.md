# Range Select Mode

Range Select mode copies a range reference for use later or delete a range's contents. You can enter range
select mode from CellEdit mode with `CTRL-r`, or from navigation mode with `Alt-r` (this was `Ctrl-r` before fill took that key).

## Moving

{{keys:movement}}

## Selecting and copying

{{keys:selection}}

## Other

{{keys:other}}

`s` and `S` sort the rows from the range start to the cursor by the cursor's
column. All of each row moves. They work only when you entered range select
mode from navigation mode. See the `sort` command for more options.

`d` deletes the rows the selection touches and `D` deletes its columns, then the sheet returns to navigation mode. A numeric prefix is ignored. Clearing the selection's contents is `<Del>`. See [Deleting rows and columns](./command.md#deleting-rows-and-columns).

`w` fits every column the selection touches to the selected rows. A numeric prefix is ignored. `:col-width` and `:autofit-cols` also apply to those columns in this mode, and ignore their `count`.

`<C-d>` fills each column of the selection down from its top cell and `<C-r>` fills each row right from its left cell. Range select mode stays active, so `<C-d>` then `<C-r>` fills a block from its top left cell. `x` stamps the start cell instead and does not copy formats. See [Filling](./command.md#filling).

`:` opens the command prompt with `'<,'>`, the range of the selection, in
it. `:'<,'>s/pattern/replacement/` replaces in the selected cells. See
[Replacing](./command.md#replacing).

When you have selected the end of the range you will exit range select mode and
the range reference will be placed into the cell contents you are editing.

<aside>We only support continuous ranges for the moment. Planned for
discontinuous ranges still needs the interaction interface to be
determined.</aside>

`<A-1>` to `<A-6>` and `<A-0>` set the number format of the selected cells (number, time, date, currency, percent, scientific, general), and `<A-.>` and `<A-,>` add and remove a decimal place. `:format` and `:decimals` apply to exactly the selected cells, and `:format-cols` and `:format-rows` to every column or row the selection touches. See [Number formats](./command.md#number-formats).

`<A-b>`, `<A-i>`, `<A-u>` and `<A-s>` toggle bold, italic, underline and strikethrough on the selected cells, decided by the first selected cell so the selection ends up uniform. `<A-l>`, `<A-e>` and `<A-;>` align the selection left, center and right. Each key is one undo step. `:style-cols` and `:style-rows` cover whole columns and rows.
