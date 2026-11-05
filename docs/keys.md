# Key Mappings

You can change the keys that sheetui uses in navigation mode and in range
select mode. Mappings work like vim's `nnoremap` and `vnoremap`: a key, or a
sequence of keys, runs other keys or a command.

Edit mode, the command line and the dialogs keep their fixed keys. Only
navigation mode and range select mode can be remapped.

## A first example

Put this in your keys file:

```keys
# Swap j and k.
nnoremap j k
nnoremap k j

# Space then c copies the cell and moves down.
nnoremap <Space>c yj

# Space then w saves.
nnoremap <Space>w <Cmd>write<CR>

# Never quit by accident.
nunmap q
```

The next time you start sheetui the mappings are in effect. `Alt-h` shows the
help with your keys in it, and `:nmap` lists your mappings.

The search keys (`/`, `?`, `n`, `N`, `*`, `#`) are built-in navigation keys like
the rest, so they can be remapped or turned off with `nunmap`. They are listed
in the `search` group of the help. See [Search and Replace](./search.md).

## The keys file

sheetui reads the keys file when it starts, after the theme. It looks in this
order:

1. `--keys <path>` on the command line.
2. `$XDG_CONFIG_HOME/sheetui/keys`, by default `~/.config/sheetui/keys`.

A missing file in the config directory is normal and says nothing. A file you
name with `--keys` that cannot be read is a warning.

The file is a list of commands, one on each line:

* A line that starts with `#` is a comment. Blank lines are ignored. A `#`
  anywhere else is a key: `nnoremap <Space>h #` maps to the `#` key.
* A leading `:` is allowed and ignored, so you can paste a command from the
  command line.
* Only `nnoremap`, `vnoremap`, `nunmap` and `vunmap` work in the file.

A bad line is skipped and recorded as a warning with its line number, and the
rest of the file still loads. The message line shows how many warnings there
were, and `:nmap` lists them.

## Commands

You can type these commands at the `:` command line too. A mapping made there
lasts until you quit. It is not written to the keys file.

* `nnoremap <keys> <rhs>` maps keys in navigation mode.
* `vnoremap <keys> <rhs>` maps keys in range select mode.
* `nunmap <keys>` removes a mapping you made in navigation mode, or turns off
  a built-in key. `vunmap` does the same for range select mode.
* `nmap` lists your navigation mappings, with any warnings from loading the
  keys file. `vmap` lists the range select mappings.

Mapping the same keys again replaces the earlier mapping.

`nunmap` of one of your mappings removes it, and the built-in meaning of those
keys, if any, comes back. `nunmap` of a built-in key turns it off. `nunmap` of
keys that are neither says so.

The commands start with `n` and `v` for the mode, as in vim. They are
`noremap` commands: a mapping never triggers another mapping. That is why
swapping `j` and `k` works and cannot loop.

## Key notation

Keys are written the way vim writes them:

* A plain key is itself: `j`, `B`, `:`, `;`. A capital letter is the shifted
  key, so write `B` and not `<S-b>`.
* `<C-x>` is Ctrl and a character. `<A-x>` is Alt. `<M-x>` is the same as
  `<A-x>`. They combine: `<C-A-x>`.
* Named keys: `<CR>` (Enter), `<Esc>`, `<Tab>`, `<Space>`, `<BS>`, `<Del>`,
  `<Insert>`, `<Up>`, `<Down>`, `<Left>`, `<Right>`, `<Home>`, `<End>`,
  `<PageUp>`, `<PageDown>` and `<F1>` to `<F12>`.
* `<S-Tab>` and `<S-CR>` are Shift with a named key.
* `<lt>` is a literal `<`. A bare `<` is an error, because it starts a name.

Names are not case sensitive: `<c-s>` is `<C-s>`.

A terminal cannot tell some keys apart, and the notation follows the
terminal. `<C-i>` is the same key as `<Tab>`, and `<C-m>` is `<CR>`. Ctrl and
Shift together on a letter, `<C-S-x>`, cannot be mapped: most terminals do
not send it. The built-in copy keys use it where a terminal does, and it
shows in the help as `<C-S-c>`.

## What a mapping runs

The right-hand side is a list of keys and commands:

* Keys run as if you had typed them. `yj` copies the cell and moves down.
* `<Cmd>command<CR>` runs a `:` command without opening the command line.
  `<Cmd>sort skip 1<CR>` sorts the rows below the first. The text between
  `<Cmd>` and `<CR>` is any command, including `nmap`. It does not go into
  the command history.

You can mix them: `jj<Cmd>write<CR>`.

The keys in a right-hand side mean what the built-in keys mean. They never
run your own mappings, so `nnoremap j jj` moves down two rows and does not
run itself again.

A mapping runs inside the key that started it, so:

* It can change modes. `nnoremap <Space>e ihello<CR>` enters edit mode, types
  `hello` and commits it. A mapping that ends while you are in edit mode
  leaves you in edit mode.
* Everything it changes undoes in one step with `u`.
* A numeric prefix runs the whole mapping that many times: `3<Space>` runs the
  mapping on `<Space>` three times.
* If a step fails, the rest of the mapping is dropped and the error shows on
  the message line. A `<Cmd>` that does not parse, names an unknown command, or
  fails when it runs all count.

## Mappings of several keys

A mapping can be more than one key, such as `<Space>c`. While you are partway
through one, the keys typed so far show on the bottom border. Then:

* If the next key finishes a mapping, it runs.
* If it continues one, sheetui keeps waiting.
* If it matches nothing, all the keys typed so far are dropped.
* `Esc` drops them at once, and clears the numeric prefix.

When your keys are both a complete mapping and the start of a longer one, for
example `<Space>` and `<Space>w`, sheetui waits one second for the rest. If
nothing comes, it runs the shorter mapping. The wait is fixed at one second.

Your mappings come before the built-in keys, as in vim. A mapping on `g`
hides the built-in `gg`, because `g` is a mapping now. To get `gg` back, map
it to itself:

```keys
nnoremap g <Cmd>select-sheet Sheet1<CR>
nnoremap gg gg
```

A mapping that only continues from `g`, like `gt`, leaves `gg` alone. A
built-in key that is the start of another built-in, such as the first `g`,
waits for the next key with no time limit.

This is simpler than vim in one case. With mappings `a` and `abc`, vim runs
`a` and handles `b` and `x` again when you type `abx`. sheetui drops all
three keys.

## Keys you cannot map

* A left-hand side cannot start with a digit. Digits are the numeric prefix.
* `<Esc>` cannot start a left-hand side. It cancels the keys you have
  typed so far.
* `<C-S-x>` cannot be written, as above.

## Not in vim's mapping model

sheetui has the part of vim's mapping that is useful for a spreadsheet. It does
not have:

* A recursive `map`. There is only `noremap`.
* Mappings for edit mode, the command line or dialogs, so no `inoremap` or
  `cnoremap`.
* `<Leader>`, `:set`, `:let`, `timeoutlen` and `:source`.
* The flags `<silent>`, `<buffer>`, `<expr>`, `<nowait>` and `<unique>`, and
  mappings that call a function.
* Macros and registers.

## A larger example

```keys
# A keys file for sheetui.

# Swap j and k.
nnoremap j k
nnoremap k j

# H goes to the top of the column, as gg does.
nnoremap H gg

# Space is my prefix in navigation mode.
nnoremap <Space>c yj
nnoremap <Space>w <Cmd>write<CR>
nnoremap <Space>s <Cmd>sort skip 1<CR>
nnoremap <Space>n <Cmd>new-sheet<CR>

# Ctrl-s in range select mode sorts the selected rows, as s does.
vnoremap <C-s> s

# Never quit with a single key.
nunmap q
nnoremap <Space>q <Cmd>quit<CR>
```

Be careful with a prefix key that already means something. In range select
mode `<Space>` selects the start and end of the range, so a mapping that
starts with `<Space>` there would make it wait for a second key. Check
`Alt-h` in each mode before you pick one.
