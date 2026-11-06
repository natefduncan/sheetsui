# The Config File

The config file holds the choices you want every session to start with: options,
a theme and key mappings. It works like vim's `~/.vimrc`.

## Where it lives

sheetui reads `$XDG_CONFIG_HOME/sheetui/config`, by default
`~/.config/sheetui/config`. `--config <path>` names another file instead.

A missing file in the config directory is normal and says nothing. A file you
name with `--config` that cannot be read is a warning, as is any other read
error.

sheetui only reads the file. It never writes it, and `:set` never saves.

## An example

```
# ~/.config/sheetui/config

theme dracula

set ignorecase smartcase
set searchin=raw
set nowrapscan

# Space then w saves.
nnoremap <Space>w <Cmd>write<CR>

# Never quit by accident.
nunmap q
```

## What may go in it

The file is a list of `:` commands, one on each line, run in order at startup.

* A line that starts with `#` is a comment. Blank lines are ignored. A `#`
  anywhere else is part of the command: `nnoremap <Space>h #` maps the `#` key.
* A leading `:` is allowed and ignored, so you can paste a command from the
  command line.
* Only these commands work in the file:
  * `set`, see [Options](./command.md#options). Several options on one line
    apply in order and stop at the first error.
  * `theme`, see [Themes](./themes.md). If the file has several `theme` lines,
    the last one is used.
  * `nnoremap`, `vnoremap`, `nunmap` and `vunmap`, see
    [Key Mappings](./keys.md).
* Any other command is skipped with a warning. Commands that act on a workbook,
  such as `write`, `edit` and `sort`, never work here: there is no workbook yet.

There is no `:source`, no `:let` and no conditionals. The file cannot run
another file.

## Warnings

A bad line is skipped and recorded as a warning with its file and line number,
for example `config:7: unknown option: foo`. The rest of the file still runs.
A `theme` that is not found is a warning too, and the default theme stays.

At startup the message line shows one summary, such as `3 warnings: see :config`.
`:config` lists every startup warning, from this file, the keys file and the
theme, in a dialog. `:nmap` lists only the key warnings and `:theme` only the
theme warnings.

## Precedence

For each setting, the last of these wins:

1. The built-in default.
2. The `config` file, top to bottom.
3. The legacy `keys` file, or the file named by `--keys`: mappings only.
   A mapping there replaces a mapping in `config` for the same keys.
4. `--theme` on the command line.
5. Anything you type at the `:` command line, for that session.

`--keys` replaces the legacy `keys` file only. It does not stop `config` from
running.

## The legacy keys file

`$XDG_CONFIG_HOME/sheetui/keys` still works with no change. Mappings can now go
in `config` as well; use whichever you like.
