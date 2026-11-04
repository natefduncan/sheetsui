# Design: Key Mappings

Status: proposed 2026-11-03 — decisions K1–K15. Implementation plan:
`planning/keys-plan.md`.

K1–K11 were agreed in a design interview. K12–K15 are defaults chosen while
writing this up. They are marked, and need a yes or no.

## User

A person who uses sheetui every day and wants keys that suit their hands:
swap `j` and `k`, move a command to a key they like, add a personal
shortcut, or string a few keys into one. They know vim's `:nnoremap`. They
want to write the mappings once in a file and have them stay.

## Today

Keys are hardcoded as `match key.code` arms in `src/ui/mod.rs`, in six
handlers: navigation, range-select, edit, command, dialog and the quit
dialog. Navigation alone has about 40 arms. They mix modifier guards
(`if key.modifiers != KeyModifiers::CONTROL`), an arm that accepts several
keys (`'j' | Down`), and a hand-rolled two-key chord (`gg`, using
`state.char_queue` and a `TODO` asking for a better state machine). The
numeric prefix (`123h`) is handled by `handle_numeric_prefix` and
`run_with_prefix`.

There is no way to change a key without editing the source.

## Decisions

| # | Decision |
|---|----------|
| K1 | Only navigation (`n`) and range-select (`v`) are rebindable. Edit mode, the command line and the dialogs keep their fixed keys. |
| K2 | The model is vim's: `nnoremap`, `vnoremap`, `nunmap`, `vunmap`. There is no recursive `map` and no action names. |
| K3 | A right-hand side is keys, or `<Cmd>command<CR>` to run a `:` command. Keys are replayed non-recursively against the built-in table. |
| K4 | The mapping commands are real `:` commands. `$XDG_CONFIG_HOME/sheetui/keys`, else `~/.config/sheetui/keys`, is a script of them, run at startup. `--keys <path>` overrides the location. There is no `default.keys`: the built-in table stays in code. |
| K5 | A key that is a prefix of a user mapping waits 1000 ms (fixed) for the rest, then the shorter match runs. `<Esc>` cancels. Pending keys show on the message line. |
| K6 | No `<Leader>` in version 1. |
| K7 | `:nmap` and `:vmap` list the user's mappings and the load warnings in a dialog, as `:theme` does for themes. |
| K8 | The key lists in the help are generated from the key tables. The docs mark where a list goes with `{{keys:<group>}}`. A "Your mappings" section lists user mappings. |
| K9 | Vim key notation. One `Key { code, mods }` normalizer is used by the file parser and by live input. Shift is folded into a character key. `<C-S-x>` is rejected with a warning. The kitty keyboard protocol is not enabled. |
| K10 | Replay goes through the live input pipeline. User mappings apply only to the keys the user typed, not to replayed ones. An error drops the rest of the sequence. A mapping is one undo step. |
| K11 | Lookup is two-tier: user mappings first, then the built-ins. A user mapping that matches or is a prefix of the typed keys wins. Mapping `g` shadows `gg`; `nnoremap gg gg` restores it. |
| K12 | *(Default chosen.)* A left-hand side cannot start with a digit `0`–`9`: digits are the numeric prefix. The line is skipped with a warning. |
| K13 | *(Default chosen.)* `<Esc>` cannot be the first key of a left-hand side. It stays "cancel pending keys and clear the numeric prefix". |
| K14 | *(Default chosen.)* A user mapping is never applied while a numeric prefix is half typed, only to the key that ends it: `3` then a mapped key repeats that mapping's replay 3 times, like a built-in. |
| K15 | *(Default chosen.)* Mappings are for this session only. `:nnoremap` typed at runtime is not written to the file. |

## Out of scope

Compared with full vim, version 1 leaves out:

- Recursive `map`, `<Plug>`, `<expr>`, `<buffer>`, `<silent>`, `<nowait>`,
  `<unique>`, and callback mappings.
- Mappings for edit mode, the command line and dialogs (`inoremap`,
  `cnoremap`). Operator-pending mode: `d` here is one key, not an operator.
- `<Leader>`, `:set`, `:let`, `timeoutlen`, `:source`, conditionals.
- Macros and registers (`q`, `@`).
- `<C-S-x>` and any key a terminal cannot report without the kitty protocol.
  The notation parses, so a later opt-in changes no files.
- Mouse mappings.

## Key notation

A key is one of:

- A single character: `j`, `B`, `:`, `;`. Shift is part of the character, so
  `B` and not `<S-b>`.
- `<C-x>`, `<A-x>` (alias `<M-x>`): Ctrl and Alt with a character.
- A named key: `<CR>` `<Esc>` `<Tab>` `<Space>` `<BS>` `<Del>` `<Up>`
  `<Down>` `<Left>` `<Right>` `<Home>` `<End>` `<PageUp>` `<PageDown>`
  `<F1>`–`<F12>`, and `<lt>` for a literal `<`.
- `<S-Tab>` and `<S-CR>`: Shift is allowed only on non-character keys.

Names are case-insensitive: `<c-s>` is `<C-s>`. `<C-i>` and `<C-m>` are
accepted and are the same key as `<Tab>` and `<CR>`: a terminal sends the
same code for both. `<C-S-x>` is rejected: a terminal cannot report it
without the kitty protocol.

Ctrl on a capital letter is a different key from Ctrl on the lowercase
letter, for the few terminals that report it (Ctrl-Shift-C copies a cell as
it is displayed). The notation cannot write it: `<C-C>` is `<C-c>`, as in
vim, and `<C-S-c>` is rejected. The built-in tables add the key directly, and
it prints as `<C-S-c>`.

Modifiers combine: `<C-A-x>`. A key press event with kind `Release` or
`Repeat` is ignored, as today.

## Commands

All four are `:` commands. In the `keys` file, a leading `:` is optional.

```
nnoremap <lhs> <rhs>     in navigation mode, <lhs> acts as <rhs>
vnoremap <lhs> <rhs>     in range-select mode
nunmap   <lhs>           hide a built-in or remove a user mapping
vunmap   <lhs>
nmap                     list navigation mappings and warnings
vmap                     list range-select mappings
```

- The left-hand side is one key sequence, in the notation above. The
  right-hand side is the rest of the line.
- A right-hand side is a sequence of keys and `<Cmd>…<CR>` tokens.
  `<Cmd>` text runs up to the next `<CR>`.
- `unmap` of a user mapping removes it, and the built-in meaning of those keys,
  if any, comes back. `unmap` of a built-in key hides it, and a second
  `unmap` of it changes nothing. `unmap` of keys that are neither is a
  warning, "No mapping for <keys>". Hiding a built-in is how `nunmap q`
  works.
- A later mapping of the same left-hand side replaces the earlier one.
- Comments are whole lines only. Blank lines and lines starting with `#` are
  ignored. A `#` anywhere else is a literal key, so `nnoremap <Space>h #`
  maps to the key `#`.

### `keys` file

A line in the file is a command. The file is read once at startup, after the
theme. Each line runs on its own. A bad line (unknown command, bad key
notation, K12 or K13 violation) is skipped and recorded as a warning with its
line number. The rest still loads. Warnings show on the message line as a
count, and in full in `:nmap` and `:vmap`.

Example `keys` file:

```
# ~/.config/sheetui/keys
nnoremap k j
nnoremap j k
nnoremap <Space>c yj
nnoremap <Space>w <Cmd>write<CR>
nnoremap <Space>s <Cmd>sort skip 1<CR>
nunmap q
vnoremap <Space>s <Cmd>sort<CR>
nnoremap gg gg
nnoremap g <Cmd>select-sheet Sheet1<CR>
```

## Lookup

When a key arrives in navigation or range-select mode:

1. Append it to the pending keys.
2. If the pending keys equal a user mapping and no longer user mapping starts
   with them: run it, clear pending.
3. If they are a strict prefix of some user mapping: wait. After 1000 ms
   with no key, run the longest user mapping that matches, or drop the keys
   if none does.
4. If they match no user mapping and are not a prefix of one: look them up in
   the built-in table, exactly as today. If the pending keys are a strict
   prefix of a built-in (`g`), wait for the next key, with no timeout, as
   today.
5. A key that matches neither: drop all pending keys.

`<Esc>` with keys pending clears them and the numeric prefix.

Known simplification: vim, given mappings `a` and `abc` and the keys `ab` then
`x`, runs `a` and then handles `b` and `x` again. Here the whole pending
sequence is dropped, and a timeout runs the longest mapping that starts the
pending keys and drops the rest. Both follow the steps above, and neither
re-feeds keys.

Step 4 is why a user with no mappings sees no change: with an empty user
table the lookup goes straight to the built-ins, and `gg` needs no timeout.

## Replay

Running a user mapping replays its right-hand side, one item at a time,
through the same dispatch a typed key uses:

- A key goes to the current mode's handler. In navigation and range-select
  the handler looks the key up in the built-in table only, so a replayed key
  never triggers a user mapping (K10). In edit mode and the command line a
  key is ordinary text input.
- `<Cmd>text<CR>` runs `text` through the `:` command parser, as if `:text`
  were entered, but never enters command mode and never goes into the
  command history.
- If a replayed key or command fails, the rest of the sequence is dropped and
  the error shows on the message line. A command that does not parse, is
  unknown, or fails when it runs all count, and none of them end the app. A
  command typed in command mode keeps its dialogs, as before.
- The whole replay runs inside the one key that started it, so it is one undo
  step, even across a mode change. A mapping that waited out the timeout is
  one step too.
- A mapping that ends in edit mode leaves you in edit mode.

The numeric prefix applies to the whole replay (K14).

## Built-in key table

The `match` arms in `src/ui/mod.rs` become one table per mode, mapping a
sequence of normalized keys to a handler. A handler is a small enum value
(`MoveDown`, `CopyCell`, `NextSheet`, …) with a one-line description, a help
group, and the code to run. Arms that accept several keys, such as
`'j' | Down`, become several entries pointing at the same handler. The
`!= CONTROL` and `contains(CONTROL)` guards go away: lookup is exact on the
normalized key.

The table is the single source for dispatch, for the lookup above, and for
the generated help (K8). `char_queue` goes away: `gg` is a two-key entry.

Behavior changes on purpose, all small:

- A key with an extra modifier no longer falls through to a bare-key arm.
  Today `Alt-j` moves down because the guard only excludes Ctrl; so do
  `Shift-Down`, `Alt-Right`, `Ctrl-e` (edit), `Alt-1` (numeric prefix) and
  `Alt-s`. After the change they do nothing.
- A key that cannot continue a pending sequence is swallowed with it. Today
  `g` then `j` moves down and leaves the `g` pending. After the change `gj`
  is not a key and both are dropped.
- A digit typed while keys are pending is part of the sequence, not a
  numeric prefix.
- `Shift-Tab` works. Terminals send it as `BackTab`, which no arm matched.

The edit-mode and dialog handlers are untouched.

## Help

`docs/navigation.md` and `docs/visual.md` keep their prose. Where a key list
goes, they hold a line `{{keys:<group>}}`. When the help opens, each
placeholder is replaced with a markdown list built from the table entries in
that group, using the keys that currently trigger each handler, so a
remapped `j` shows its new key. Handlers with no key show nothing. A "Your
mappings" section lists the user's mappings from `:nmap` or `:vmap`.

Placeholders are expanded before the markdown is parsed, so the existing
renderer is unchanged. Each placeholder becomes a bullet list, and two lists
with only a blank line between them merge into one loose list that the help
renders badly. A doc therefore puts a heading or prose between placeholders,
and a test enforces it.

A handler's keys are the built-in keys the user has not hidden, then the
user's mappings whose right-hand side is exactly the keys of one built-in
entry. A user `k` mapped to `j` is listed for "move down". A mapping to a
sequence or a `<Cmd>` is only in "Your mappings". The prose around the lists
does not follow remaps: it still says "`q` quits". An unknown group name is left in the text as a
visible `{{keys:name}}` so a typo is noticed.

## Code

- New module `src/ui/keys/`: the `Key` type, the notation parser and printer,
  the built-in tables (`builtin.rs`), the user mapping store and lookup
  (`map.rs`), the replay queue, the `keys` file loader.
- `ws.state.keymap` holds the user mappings, the pending keys, the pending
  deadline and the load warnings.
- `src/ui/cmd.rs`: four new commands in the existing `try_consume_*` style.
- `src/main.rs`: a `--keys` flag, loading the file after the theme, and an
  event loop that can wake on a timeout.
- `src/ui/help/mod.rs`: placeholder expansion.

## Tests

- Notation: every key kind parses and prints back to a canonical form;
  `<C-S-x>`, unknown names, empty `<>` and a lone `<` are errors.
- Normalizer: crossterm events for `B` with and without a SHIFT flag give the
  same key; `Ctrl-i` and `Tab` are the same key.
- Built-in table: every key that works today still works. The existing
  tests in `src/ui/test.rs` pass unchanged.
- Lookup: step 2, 3, 4, 5 each have a test, including timeout expiry,
  shadowing (`g` over `gg`), and `nnoremap gg gg`.
- Replay: key sequences, `<Cmd>`, mode changes (`i`, `v`, `:`), an error
  dropping the rest, and no user mapping firing inside a replay.
- Swapping two keys (`j`↔`k`) works and does not loop.
- File: each command, comments, bad lines with line numbers, K12 and K13
  violations, a missing file (no warning), `--keys` with a bad path.
- `:nmap` and `:vmap` list mappings and warnings.
- Help: placeholders expand, an unknown group stays visible, a remap shows
  in the list.

## Open items

- The `Release` and `Repeat` events: today only `Press` is handled. Check that
  holding a key still repeats on terminals that send `Repeat` as `Press`.
- Confirm K12–K15.
