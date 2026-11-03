# Plan: Key Mappings

Implements `planning/keys-design.md` (K1–K15). Work goes on a long-lived
`keys` branch, one commit per stage, as the theme work did. Each stage
builds, passes `cargo test` with zero warnings, and is tracked by one GitHub
issue on `natefduncan/sheetsui`.

## Findings from the code

Checked against the code on 2026-11-03. Things the design assumed or left
open:

1. **Every key is already one undo action.** `handle_input` calls
   `book.begin_action()` and `end_action()` around each key
   (`src/ui/mod.rs:347-357`). If replay runs inside a single `handle_input`
   call, a whole mapping is one undo step with no new mechanism. The design
   lists this as out of scope (K10); stage 4 checks whether it holds,
   including across a mode change, and if it does the design is amended.
2. **The message is cleared on every key** (`self.state.message = None`,
   `src/ui/mod.rs:346`). Pending keys (K5) cannot use that slot or they would
   vanish when the next key arrives. They need their own field and their own
   place in the bottom title.
3. **The loop blocks on `event::read()`.** `ReadFn` is
   `Box<dyn FnMut() -> Result<Event>>` and the `--log-input` wrapper is built
   around it (`src/main.rs:24-56`). A timeout needs
   `event::poll(deadline)`. Tests call `handle_input` directly, so the
   timeout is exposed as a method (`handle_key_timeout`) they can call
   without a clock.
4. **Modifier guards change behavior.** Arms like
   `'j' | Down if modifiers != CONTROL` also fire for `Alt-j` and
   `Ctrl-Shift-…` today. Exact lookup on normalized keys stops that. The
   changes are listed in the design and pinned by tests in stage 2.
5. **Shift arrives two ways.** Terminals send `Char('B')` with SHIFT set, or
   without it. The normalizer drops SHIFT from character keys. `Shift+Enter`
   and `Shift+Tab` keep it. `Ctrl` on a capital letter stays its own key
   (an existing test sends it), so the Ctrl-Shift-C arms are kept as direct
   table entries. `BackTab` is folded into `Shift+Tab`: the old
   `Tab if SHIFT` arm never matched what terminals send.
6. **`s` has two arms and `p`, `v`, `r`, `h`, `l` have a bare and a Ctrl
   arm** that depend on arm order and guards. A table has no order, so each
   becomes an entry per distinct key. A test per pair pins the old behavior.
7. **Digits are handled twice**: `handle_numeric_prefix` in navigation and in
   range-select (`src/ui/mod.rs:673, 931`). Both stay built-in (K12), outside
   the table, ahead of the lookup.
8. **`<Cmd>` must not use `exit_command_mode`.** That path adds the text to
   the command history and pops the modality (`src/ui/mod.rs:1335-1343`).
   `<Cmd>` calls `handle_command` directly.
9. **`cmd::parse` returns a `Cmd<'a>` that borrows its input**, one
   `try_consume_*` function per command. The four mapping commands follow
   that pattern, and own their strings, since a mapping outlives the line
   that defined it.
10. **Help is `include_str!` markdown** run through
    `Markdown::from_str_themed` (`src/ui/help/mod.rs`). Placeholder expansion
    happens on the string before that call, so `to_widget` gains a keymap
    argument and nothing in the renderer changes.
11. **The theme loader is the template** for the file loader: lenient,
    per-line warnings, XDG lookup, a flag override, warnings on the message
    line. Reuse its path helper rather than copy it.
12. **Edit mode and the command line must not see user mappings.** Their
    handlers are untouched, and replayed keys reach them as plain input
    (K10). A test pins this: a user mapping on `j` does not fire in the edit
    box.

## Stages

### 1. Key type and notation, no behavior change

New `src/ui/keys/mod.rs` with `Key { code, mods }`, a `normalize`
function from a crossterm `KeyEvent`, a parser from notation to `Key`
sequences, and a printer back to canonical notation.

- Normalization: drop SHIFT on `Char`; keep it on `Tab`, `Enter` and the
  rest; map `Ctrl-i`/`Tab` and `Ctrl-m`/`Enter` to one key.
- Parser: the full grammar in the design, with errors for `<C-S-x>`, unknown
  names and malformed brackets. Names are case-insensitive.
- Nothing calls it yet. Unit tests only.

### 2. Table-driven built-ins, no behavior change

Replace the `match` in `handle_navigation_input` and
`handle_range_select_input` with a lookup into tables in
`src/ui/keys/builtin.rs`.

- A `Handler` enum for each action (`MoveDown`, `CopyCell`, `NextSheet`,
  `EnterEdit`, …), each with a description and a help group. One `run`
  function maps a handler to the existing code.
- Tables map `&[Key]` to `Handler`, per mode. `gg` is a two-key entry.
  `char_queue` is deleted.
- Digits and the numeric prefix stay ahead of the table (finding 7).
- Every key that worked before still works, except the modifier cases in
  finding 4. The commit message lists them.
- `src/ui/test.rs` passes unchanged. New tests pin the paired arms in
  finding 6 and each modifier case that changed.

### 3. User mappings, lookup and timeout

- `ws.state.keymap`: user mappings per mode, the pending keys, the
  deadline, the warnings.
- Lookup as in the design (steps 1–5), in front of the built-in table.
- `main.rs` loop: when keys are pending, wait with `event::poll` until the
  deadline, then call `handle_key_timeout`. `ReadFn` becomes a function that
  takes an optional deadline. `--log-input` logs events as before.
- Pending keys show in the bottom title, apart from the message (finding 2).
- No way to create a mapping yet: tests insert them directly.
- Tests: exact match, prefix then match, prefix then timeout, prefix then
  mismatch, `<Esc>` cancel, shadowing (`g` over `gg`), `nnoremap gg gg`, and
  that an empty user table never waits on `gg`.

### 4. Mapping commands and replay

- The four commands are parsed in `src/ui/keys/command.rs`, not `cmd.rs`:
  `cmd::parse` returns `&'static str` errors, and a key-notation error needs
  text built at run time ("`<Nope>` is not a key name"). `handle_command`
  tries the mapping parser first, then `cmd::parse`. K12 and K13 checks live
  in the mapping parser, so the file and the command line share them.
  `KeyMap::apply` does the work and returns the error text, for stage 5.
- Replay queue: items are keys or `<Cmd>` text. Keys go through the current
  mode's dispatch with user mappings disabled. `<Cmd>` calls
  `handle_command` (finding 8). An error clears the queue.
- Replay runs inside one `handle_input` call. Result of the check (finding 1):
  a mapping is one undo step, through a mode change and through the timeout
  path. The design now says so.
- The numeric prefix repeats the whole replay (K14).
- Tests: key sequences, `<Cmd>`, the `i`, `v` and `:` mode changes, error
  dropping the rest, no user mapping firing during replay, a `j`/`k` swap
  with no loop, edit mode not seeing user mappings (finding 12), the undo
  check.

### 5. `keys` file, `--keys`, `:nmap` and `:vmap`

- Path: `--keys`, else `$XDG_CONFIG_HOME/sheetui/keys`, else
  `~/.config/sheetui/keys`, reusing the theme path helper (finding 11).
  A missing default file is not an error. A missing `--keys` path is a
  warning.
- Load after the theme, in `main.rs`. Each non-comment line goes through the
  command parser. A line that fails is skipped and recorded with its line
  number. A count of warnings shows on the message line.
- `:nmap` and `:vmap` open a dialog listing mappings (left-hand side, right-
  hand side) and warnings. With no mappings and no warnings it says so.
- Tests: every command in a file, comments, blank lines, bad lines with line
  numbers, a K12 and a K13 violation, no file, bad `--keys` path, and the
  `:nmap` and `:vmap` output.

### 6. Generated help

- Give each `Handler` a help group name and a one-line description.
- `{{keys:<group>}}` expansion in `src/ui/help/mod.rs`, run before
  `Markdown::from_str_themed` (finding 10). Each entry lists every key that
  triggers the handler now, user mappings included.
- Edit `docs/navigation.md` and `docs/visual.md`: replace the hand-written
  key lists with placeholders, keep the prose.
- A "Your mappings" section, from the same data as `:nmap`.
- Tests: expansion, an unknown group left visible, a remap appearing in the
  list, every group used in the docs exists in the table.

### 7. Docs

- `docs/keys.md`: the commands, the notation, the file location, the lookup
  and timeout rules, shadowing, replay, the out-of-scope list from the design
  in plain terms, and a worked `keys` file.
- Add `:nnoremap`, `:vnoremap`, `:nunmap`, `:vunmap`, `:nmap`, `:vmap` to
  `docs/command.md`, and a `keys` topic to `docs/index.md` and the help
  topics.
- Update `README.md` and `CONTRIBUTING.md` where they list keys or files.

## Out of scope

See the design. Nothing is added in this plan beyond it, apart from the
undo check in stage 4.

## Open items

- Confirm K12–K15 in the design.
- The `Release` and `Repeat` events: confirm repeat still works on terminals
  that send it as `Press` (stage 2).
- Windows Terminal under WSL: confirm the notation round-trips through
  `Alt` and `Ctrl` keys there (stage 1).
