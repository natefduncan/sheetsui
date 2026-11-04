# Navigation Mode

The interface will start out in navigation mode. You can navigate around the
table and between the sheets using the following keybinds. Keys are written in
vim notation: `<C-n>` is Ctrl-n, `<A-h>` is Alt-h, `<CR>` is Enter, and a
capital letter is the shifted key.

## Cell Navigation

{{keys:cell-navigation}}

## Sheet Navigation

{{keys:sheet-navigation}}

Sheet navigation moving will loop around when you reach the ends.

## Numeric prefixes

You can prefix each of the keybinds above with a numeric prefix to do them that
many times. So typing `123h` will move to the left 123 times. Hitting `Esc`
will clear the numeric prefix if you want to cancel it.

**Modifying the Sheet or Cells**

{{keys:modify}}

## Undo and Redo

{{keys:undo}}

A numeric prefix undoes or redoes that many changes: `3u` undoes three.

Everything that one key or one command changes undoes in one step: a cell
edit, a paste, a sort, `:insert-rows 5`, or `5 Ctrl-l`. `o` and `O` are two
steps: the new row, then the edit you type in it. Moving the cursor, selecting,
switching sheets, and saving are not changes.

When you undo back to the point where you last saved, the sheet counts as not
changed, so `q` quits with no prompt. The history starts empty each time you
open a file, and it is not saved when you quit.

## Other Keybindings

{{keys:other}}

Range selections made from navigation mode will be available to paste into a Cell Edit.

<aside>Note that for `q` this will not currently prompt you if the sheet is not
saved.</aside>

Note also that copy paste works with the system clipboard.

