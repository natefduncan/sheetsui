# Range Select Mode

Range Select mode copies a range reference for use later or delete a range's contents. You can enter range
select mode from CellEdit mode with `CTRL-r`.

## Moving

{{keys:movement}}

## Selecting and copying

{{keys:selection}}

## Other

{{keys:other}}

`s` and `S` sort the rows from the range start to the cursor by the cursor's
column. All of each row moves. They work only when you entered range select
mode from navigation mode. See the `sort` command for more options.

When you have selected the end of the range you will exit range select mode and
the range reference will be placed into the cell contents you are editing.

<aside>We only support continuous ranges for the moment. Planned for
discontinuous ranges still needs the interaction interface to be
determined.</aside>
