# Search and Replace

This page is the overview. The details are in
[Navigation](./navigation.md#search) (the `/`, `?`, `n`, `N`, `*` and `#` keys)
and [Command Mode](./command.md#replacing) (`:s` and `:set`).

The interface is vim's: `/` and `?` search, `n` and `N` repeat, `*` and `#`
search for cells like this one, and `:s` replaces. A search works on the
current sheet only, left to right across a row and then down, and only visits
cells that have content.

## Patterns are not vim's `magic` mode

A pattern is a regular expression in the syntax of Rust's `regex` crate. It is
close to vim's "very magic" `\v` mode, not to vim's default `magic` mode:

* Write `(a|b)`, not `\(a\|b\)`.
* Write `a+`, not `a\+`.
* Write `\.` for a literal dot.
* `\<` and `\>` word atoms and `\@<=` lookbehind do not exist.
* `\c` and `\C` work as in vim: they ignore or match case, and are not part of
  what is matched.

A pattern that does not parse shows the reason and does nothing.

## Value and raw text

A cell has two kinds of text: the value the sheet shows (`$12.00`, `50%`, the
result of a formula) and the raw contents you see with `e` (`12`, `0.5`,
`=A1*2`).

* `/`, `?`, `n`, `N`, `*` and `#` match the shown value, or the raw contents
  with `:set searchin=raw`. Use `raw` to search formulas: `/SUM\(`.
* `:s` always edits the raw contents, whatever `searchin` is, and enters the
  result as if you typed it. A formula stays a formula, and the format and
  style of the cell do not change.

## Why `/` can find what `:s` does not

`/12\.00` finds a cell that stores `12` and shows `$12.00`, but
`:s/12\.00/x/` does not match it, because `:s` sees the stored `12`. To search
the text that `:s` edits, `:set searchin=raw`. The other way round also holds:
`/` with the default `searchin=value` does not see the `=` in a formula, but
`:s/SUM/AVERAGE/` changes it.

## Options

`:set` changes them for the session. See [Options](./command.md#options).

* `searchin`: `value` (default) or `raw`. Which text a search matches.
* `wrapscan` (`ws`): on by default. A search goes on from the other end.
* `ignorecase` (`ic`): off by default.
* `smartcase` (`scs`): off by default. With `ignorecase`, a pattern with an
  uppercase letter matches case.
* `hlsearch` (`hls`): on by default. Matching cells are highlighted, using the
  theme slots `search` and `search.current`. `:noh` hides them until the next
  search.
* `incsearch` (`is`): on by default. The cursor follows the pattern as you
  type, and `Esc` puts it back.

## Replacing

* `:s/pat/rep/` replaces in the current cell, `:%s/pat/rep/` in the whole
  sheet, and `:'<,'>s/pat/rep/` in the last range selection.
* In the replacement, `&` is the whole match, `\1` to `\9` are groups, and
  `\&`, `\\` and `\/` are literal characters.
* Flags are `g` (every match), `i` and `I` (case), `n` (count only), `e` (no
  error without a match) and `c` (confirm each).
* A whole `:s` is one undo step.

### The `c` prompt

With `c` the cursor goes to each match and the bottom line asks
`replace with <rep> (y/n/a/q/l)?`:

* `y` replaces this one and goes on, `n` skips it.
* `a` replaces this one and all the rest, `l` replaces this one and stops.
* `q` or `Esc` stops.

The whole session is one undo step. When it ends the cursor is on the last
cell that changed.

## Keys

The search keys are `nnoremap`-able like any other; see
[Key Mappings](./keys.md). `/` and `?` are not bound in range select mode.
