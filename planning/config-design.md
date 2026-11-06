# Design: Startup Config File

Status: proposed 2026-11-05 — decisions C1–C14, all defaults chosen while
writing this up and not yet reviewed. Related: `planning/keys-design.md`
(K4, K15), `planning/search-design.md` ("Options"), `planning/theme-design.md`.

## User

A person who uses sheetui every day and has settled on how they like it:
`ignorecase` on, `smartcase` on, a theme, a few key mappings. They know vim's
`~/.vimrc`. They want to write those choices once in one file and have every
session start that way.

## Today

Three things persist between sessions, each in its own place and its own way:

- The theme: a `theme <name>` line in `$XDG_CONFIG_HOME/sheetui/config`. The
  loader (`read_config` in `src/ui/theme/lookup.rs`) takes the first such line
  and ignores every other line.
- Key mappings: `$XDG_CONFIG_HOME/sheetui/keys`, a script of `nnoremap`,
  `vnoremap`, `nunmap` and `vunmap` lines (`src/ui/keys/file.rs`). Any other
  command in it is skipped with a warning.
- Command and search history: written for you under `$XDG_STATE_HOME`.

`:set` options (`wrapscan`, `ignorecase`, `smartcase`, `searchin`, `hlsearch`,
`incsearch`) last until you quit and are not saved. There is nowhere to put
`set ic`. `set ic` in the keys file is a warning.

So a person who wants three settings to stay edits two files, and cannot make
the third stay at all. Each new setting would need its own file or its own
special line.

## Decisions

| # | Decision |
|---|----------|
| C1 | One startup file, `$XDG_CONFIG_HOME/sheetui/config`, else `~/.config/sheetui/config`. It is the file that exists today for `theme`. `--config <path>` names another. |
| C2 | The file is a script of `:` commands, run in order at startup, as the keys file already is. It is the same model as `~/.vimrc`: no new syntax, anything you can type at `:` that is allowed here works here. |
| C3 | The allowed commands are `set`, `theme`, `nnoremap`, `vnoremap`, `nunmap` and `vunmap`. Any other command is skipped with a warning. Commands that act on a workbook (`write`, `edit`, `sort`, `s/…/…/`, `fill-down` and the rest) are never allowed: there is no workbook yet. |
| C4 | Line rules are the keys file's: `#` starts a comment line, blank lines are skipped, a leading `:` is optional. A `#` anywhere else is part of the command, so `nnoremap <Space>h #` still maps `#`. |
| C5 | A bad line is skipped and recorded as a warning with its file and line number. The rest of the file still runs. This is the keys file's rule, extended to every command. |
| C6 | Order is first `config`, then the legacy `keys` file. A mapping in `keys` replaces a mapping in `config` for the same keys, as a later mapping always does. The legacy file keeps working with no change; nothing is deprecated yet. |
| C7 | `--theme` beats a `theme` line in the config. The `theme` line in the file works as `:theme` does at runtime, so a file with two `theme` lines uses the last. Today's loader uses the first. |
| C8 | `--keys <path>` replaces the legacy `keys` file only. It does not stop `config` from running. |
| C9 | Options set in the file are the starting values for the session. `:set` at the command line still changes them for the session only, and `:set` never writes the file. |
| C10 | There is no `:source`, no `:let`, no conditionals and no way for the file to run a file. Version 1 reads exactly the one file (plus the legacy keys file). |
| C11 | A missing config file in the config directory is normal and says nothing. A file named with `--config` that cannot be read is a warning, as is any other read error. A `theme` that is not found is a warning, and the default theme stays. |
| C12 | Warnings from the config, the keys file and the theme all go to the same list. The message line says how many there were. `:nmap` lists the key warnings and `:theme` lists the theme warnings, as now; a new `:config` command lists all startup warnings in a dialog. |
| C13 | The file is never written by sheetui. Persisting a `:set` or a mapping you typed is a separate feature: this design only reads. |
| C14 | The `.sui` workbook format is not touched. The config file is user settings, loaded once at startup. A workbook is shared data, written on every save. |

## Out of scope

- Writing the config from the app (`:mkconfig`, `:set` with a save flag, or a
  config editor).
- `:source <path>`, `:let`, `:if`, user-defined commands and abbreviations.
- Per-workbook settings, such as a modeline or a project-local config.
- Settings that are not commands today. Each new setting becomes a `:set`
  option first (add a row to `OPTIONS` in `src/ui/options.rs`) and is then
  available in the file with no more work.
- Removing the legacy `keys` file or the `--keys` flag.

## The file

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

`set` takes several options on one line, as it does at the command line: they
apply in order and stop at the first error, and the error is then the line's
one warning.

## What runs a line

Today there are two loaders with two parsers: `read_config` looks for the word
`theme`, and `keys::file::load` calls the key command parser and skips anything
that is not a mapping. The change is one loader for the whole file:

1. Read the file, split it into lines, and drop blank lines and comments (C4).
2. Parse each line with the same function the `:` command line uses.
3. Check the parsed command against the allowlist (C3). A command that is not
   on it becomes a warning that names the command and says which ones work in
   a config file.
4. Apply it: `set` to the options, `theme` to the theme loader, a mapping to
   the key map.

The allowlist is a match on the parsed command, not a list of words, so a new
command is off by default in the file until someone adds it.

Applying `set` needs the options at startup, before the first frame is drawn.
`Options` already exists on the workspace; only the order in `main.rs` changes
(config first, then `init_theme`, then `init_keys`).

## Precedence

For each setting, the last of these wins:

1. The built-in default.
2. `config`, top to bottom.
3. `keys` (legacy file or `--keys`): mappings only.
4. A command-line flag: `--theme` is the only one that sets a thing the
   config sets (C7).
5. Anything typed at the `:` command line, for the session.

## Warnings

A warning today is `{ line, msg }` with the file implied. With three sources
that is not enough. A warning gets the path it came from, and the message line
shows `config:7: unknown option: foo`. `:nmap` and `:theme` filter the list to
their own source, so their dialogs do not change. `:config` shows all of them.

The message line at startup shows one summary, `3 warnings: see :config`, not
one summary per file.

## Compatibility

- A `config` file with only a `theme <name>` line behaves as before.
- A `config` file written for the old loader that has other lines the old
  loader ignored, such as `other 1`, now gets a warning on each. These lines
  did nothing, so this changes no setting; it only makes the mistake visible.
- The `keys` file is read as before and needs no change.
- Two `theme` lines now use the last one, not the first (C7).

## Testing

Tests take their directories as arguments, as the theme and keys tests do, so
none of them touch the real environment.

- Each allowed command in a file takes effect: `set ic`, `set searchin=raw`,
  `theme light`, a mapping and an unmap.
- A disallowed command (`write`, `sort`, `s/a/b/`) is a warning that names the
  line, and the lines after it still run.
- A bad option (`set foo`, `set ws=1`) is a warning with the `E518` or `E474`
  text, and applies nothing on that line.
- Comments, blank lines, a leading `:`, and `#` as a mapped key.
- `--theme` beats the file. `--config` replaces the default location. A
  missing default file is silent; a missing `--config` file is a warning.
- `config` then `keys`: the same keys mapped in both give the `keys` mapping.
- Old behaviour: a file with only `theme x` and one with `other 1` plus `theme x`.
- A UI test that starts a workspace from a config with `set ic scs` and checks
  that the search ignores case, and that `:set ic?` reads `ignorecase`.

## Documentation

- A new `docs/config.md`: where the file lives, what may go in it, an example,
  and the precedence list. Link it from `docs/index.md`.
- `docs/command.md`, "Options": replace "They last until you quit and are not
  saved" with a note that a `set` line in the config file sets the start value.
- `docs/themes.md`: the `theme` line section points to the config page.
- `docs/keys.md`: say that mappings can go in `config` too, and that `keys`
  still works.

## Implementation steps

1. Make warnings carry their source, with no change in what is shown.
2. A config loader that runs lines through the command parser with an
   allowlist; `set`, mapping commands and `theme` work in it.
3. Wire it into startup in the order above; `theme` reads from it, and
   `read_config` goes away.
4. `--config` flag and the `:config` warnings dialog.
5. Docs.

Each step builds and passes the tests on its own, and each is one commit.
