use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::{
    builtin::{self, Lookup, NavHandler, RangeHandler},
    command,
    map::{
        format_rhs, parse_rhs, resolve, resolve_timeout, KeyMap, MapCommand, MapMode, Resolution,
        RhsItem, Target, UserLookup, UserTable,
    },
    format_keys, parse_key, parse_keys, Key,
};

const NONE: KeyModifiers = KeyModifiers::NONE;
const CTRL: KeyModifiers = KeyModifiers::CONTROL;
const ALT: KeyModifiers = KeyModifiers::ALT;
const SHIFT: KeyModifiers = KeyModifiers::SHIFT;

fn key(code: KeyCode, mods: KeyModifiers) -> Key {
    Key::new(code, mods)
}

fn ch(c: char) -> Key {
    Key::plain(KeyCode::Char(c))
}

fn one(text: &str) -> Key {
    let keys = parse_keys(text).unwrap_or_else(|e| panic!("{text:?}: {e}"));
    assert_eq!(1, keys.len(), "{text:?} should be one key");
    keys[0]
}

fn error_of(text: &str) -> String {
    parse_keys(text).expect_err(text).to_string()
}

#[test]
fn shift_on_a_character_is_dropped() {
    assert_eq!(key(KeyCode::Char('B'), SHIFT), key(KeyCode::Char('B'), NONE));
    assert_eq!(key(KeyCode::Char(':'), SHIFT), ch(':'));
}

#[test]
fn shift_on_a_named_key_is_kept() {
    let k = key(KeyCode::Enter, SHIFT);
    assert_eq!(SHIFT, k.mods);
    assert_ne!(k, Key::plain(KeyCode::Enter));
}

#[test]
fn back_tab_is_shift_tab() {
    assert_eq!(key(KeyCode::BackTab, SHIFT), key(KeyCode::Tab, SHIFT));
    assert_eq!(key(KeyCode::BackTab, NONE), key(KeyCode::Tab, SHIFT));
}

#[test]
fn ctrl_pairs_fold_into_the_key_a_terminal_sends() {
    assert_eq!(Key::plain(KeyCode::Tab), key(KeyCode::Char('i'), CTRL));
    assert_eq!(Key::plain(KeyCode::Enter), key(KeyCode::Char('m'), CTRL));
    assert_eq!(Key::plain(KeyCode::Esc), key(KeyCode::Char('['), CTRL));
}

#[test]
fn ctrl_on_a_capital_letter_is_its_own_key() {
    assert_ne!(key(KeyCode::Char('c'), CTRL), key(KeyCode::Char('C'), CTRL));
    // A terminal that reports Shift as well sends the same key.
    assert_eq!(
        key(KeyCode::Char('C'), CTRL),
        key(KeyCode::Char('C'), CTRL | SHIFT)
    );
    assert_eq!("<C-S-c>", key(KeyCode::Char('C'), CTRL).to_string());
    assert!(parse_keys("<C-S-c>").is_err());
}

#[test]
fn alt_keeps_its_case() {
    assert_ne!(key(KeyCode::Char('b'), ALT), key(KeyCode::Char('B'), ALT));
}

#[test]
fn modifiers_other_than_shift_ctrl_alt_are_dropped() {
    assert_eq!(
        ch('j'),
        key(KeyCode::Char('j'), KeyModifiers::SUPER | KeyModifiers::META)
    );
}

#[test]
fn from_event_normalizes() {
    let event = KeyEvent::new(KeyCode::Char('B'), SHIFT);
    assert_eq!(ch('B'), Key::from_event(&event));
    assert_eq!(ch('B'), Key::from(&event));
}

#[test]
fn as_char_is_only_for_plain_characters() {
    assert_eq!(Some('j'), ch('j').as_char());
    assert_eq!(Some('J'), ch('J').as_char());
    assert_eq!(None, key(KeyCode::Char('j'), CTRL).as_char());
    assert_eq!(None, key(KeyCode::Char('j'), ALT).as_char());
    assert_eq!(None, Key::plain(KeyCode::Enter).as_char());
}

#[test]
fn plain_characters_are_one_key_each() {
    assert_eq!(vec![ch('g'), ch('g')], parse_keys("gg").unwrap());
    assert_eq!(vec![ch('y'), ch('j'), ch('p')], parse_keys("yjp").unwrap());
    assert_eq!(vec![ch(':')], parse_keys(":").unwrap());
    assert_eq!(vec![ch('#')], parse_keys("#").unwrap());
    assert_eq!(vec![ch('>')], parse_keys(">").unwrap());
    assert_eq!(vec![ch(' ')], parse_keys(" ").unwrap());
    assert_eq!(vec![ch('é')], parse_keys("é").unwrap());
}

#[test]
fn an_empty_sequence_is_no_keys() {
    assert_eq!(Vec::<Key>::new(), parse_keys("").unwrap());
}

#[test]
fn named_keys() {
    assert_eq!(Key::plain(KeyCode::Enter), one("<CR>"));
    assert_eq!(Key::plain(KeyCode::Enter), one("<Enter>"));
    assert_eq!(Key::plain(KeyCode::Esc), one("<Esc>"));
    assert_eq!(Key::plain(KeyCode::Tab), one("<Tab>"));
    assert_eq!(ch(' '), one("<Space>"));
    assert_eq!(Key::plain(KeyCode::Backspace), one("<BS>"));
    assert_eq!(Key::plain(KeyCode::Delete), one("<Del>"));
    assert_eq!(Key::plain(KeyCode::Up), one("<Up>"));
    assert_eq!(Key::plain(KeyCode::Down), one("<Down>"));
    assert_eq!(Key::plain(KeyCode::Left), one("<Left>"));
    assert_eq!(Key::plain(KeyCode::Right), one("<Right>"));
    assert_eq!(Key::plain(KeyCode::Home), one("<Home>"));
    assert_eq!(Key::plain(KeyCode::End), one("<End>"));
    assert_eq!(Key::plain(KeyCode::PageUp), one("<PageUp>"));
    assert_eq!(Key::plain(KeyCode::PageDown), one("<PageDown>"));
    assert_eq!(ch('<'), one("<lt>"));
}

#[test]
fn function_keys() {
    assert_eq!(Key::plain(KeyCode::F(1)), one("<F1>"));
    assert_eq!(Key::plain(KeyCode::F(12)), one("<F12>"));
    assert!(parse_keys("<F0>").is_err());
    assert!(parse_keys("<F13>").is_err());
    assert!(parse_keys("<F01>").is_err());
}

#[test]
fn names_ignore_case() {
    assert_eq!(one("<CR>"), one("<cr>"));
    assert_eq!(one("<PageUp>"), one("<pageup>"));
    assert_eq!(one("<C-s>"), one("<c-s>"));
    assert_eq!(one("<F5>"), one("<f5>"));
    assert_eq!(one("<lt>"), one("<LT>"));
}

#[test]
fn ctrl_and_alt_with_a_character() {
    assert_eq!(key(KeyCode::Char('s'), CTRL), one("<C-s>"));
    assert_eq!(key(KeyCode::Char('h'), ALT), one("<A-h>"));
    assert_eq!(key(KeyCode::Char('h'), ALT), one("<M-h>"));
    assert_eq!(key(KeyCode::Char('x'), CTRL | ALT), one("<C-A-x>"));
    assert_eq!(key(KeyCode::Char('x'), CTRL | ALT), one("<A-C-x>"));
}

#[test]
fn ctrl_with_a_capital_letter_in_notation_is_the_lowercase_key() {
    assert_eq!(one("<C-s>"), one("<C-S>"));
}

#[test]
fn ctrl_i_and_ctrl_m_are_tab_and_enter() {
    assert_eq!(one("<Tab>"), one("<C-i>"));
    assert_eq!(one("<CR>"), one("<C-m>"));
    assert_eq!(one("<Esc>"), one("<C-[>"));
}

#[test]
fn shift_on_a_named_key() {
    assert_eq!(key(KeyCode::Tab, SHIFT), one("<S-Tab>"));
    assert_eq!(key(KeyCode::Enter, SHIFT), one("<S-CR>"));
    assert_eq!(key(KeyCode::Up, SHIFT | CTRL), one("<C-S-Up>"));
}

#[test]
fn a_dash_can_be_a_key() {
    assert_eq!(key(KeyCode::Char('-'), CTRL), one("<C-->"));
    assert_eq!(ch('-'), one("<->"));
}

#[test]
fn shift_on_a_character_is_an_error() {
    assert!(error_of("<S-b>").contains("capital letter"));
}

#[test]
fn ctrl_shift_on_a_character_is_an_error() {
    assert!(error_of("<C-S-x>").contains("terminal cannot report"));
    assert!(error_of("<S-C-x>").contains("terminal cannot report"));
}

#[test]
fn bad_notations_are_errors() {
    assert!(error_of("<>").contains("'<>'"));
    assert!(error_of("<").contains("no '>'"));
    assert!(error_of("a<").contains("no '>'"));
    assert!(error_of("<Nope>").contains("not a key name"));
    assert!(error_of("<C-Nope>").contains("not a key name"));
    assert!(error_of("<C-C-x>").contains("repeats"));
    assert!(error_of("<xy>").contains("not a key name"));
    assert!(error_of("<C->").contains("not a key name"));
}

#[test]
fn a_bad_key_anywhere_fails_the_sequence() {
    assert!(parse_keys("gg<Nope>").is_err());
    assert!(parse_keys("<Nope>gg").is_err());
}

#[test]
fn parse_key_reports_how_much_it_used() {
    assert_eq!((ch('g'), 1), parse_key("gg").unwrap());
    assert_eq!((one("<C-s>"), 5), parse_key("<C-s>x").unwrap());
    assert_eq!((ch('é'), 2), parse_key("é!").unwrap());
    assert!(parse_key("").is_err());
}

#[test]
fn printing_uses_canonical_notation() {
    assert_eq!("j", ch('j').to_string());
    assert_eq!("B", ch('B').to_string());
    assert_eq!("<Space>", ch(' ').to_string());
    assert_eq!("<lt>", ch('<').to_string());
    assert_eq!("<CR>", Key::plain(KeyCode::Enter).to_string());
    assert_eq!("<Esc>", Key::plain(KeyCode::Esc).to_string());
    assert_eq!("<BS>", Key::plain(KeyCode::Backspace).to_string());
    assert_eq!("<Del>", Key::plain(KeyCode::Delete).to_string());
    assert_eq!("<PageUp>", Key::plain(KeyCode::PageUp).to_string());
    assert_eq!("<F5>", Key::plain(KeyCode::F(5)).to_string());
    assert_eq!("<C-s>", key(KeyCode::Char('s'), CTRL).to_string());
    assert_eq!("<A-h>", key(KeyCode::Char('h'), ALT).to_string());
    assert_eq!("<C-A-x>", key(KeyCode::Char('x'), CTRL | ALT).to_string());
    assert_eq!("<S-Tab>", key(KeyCode::Tab, SHIFT).to_string());
    assert_eq!("<S-CR>", key(KeyCode::Enter, SHIFT).to_string());
    assert_eq!("<C-S-Up>", key(KeyCode::Up, CTRL | SHIFT).to_string());
    assert_eq!("<C-A-lt>", key(KeyCode::Char('<'), CTRL | ALT).to_string());
}

#[test]
fn printing_a_sequence() {
    assert_eq!("gg", format_keys(&parse_keys("gg").unwrap()));
    assert_eq!(
        "<Space>c<C-s><CR>",
        format_keys(&parse_keys("<Space>c<C-s><CR>").unwrap())
    );
    assert_eq!("", format_keys(&[]));
}

#[test]
fn every_notation_round_trips_through_the_printer() {
    for text in [
        "j", "B", ":", "#", ">", "é", "<Space>", "<lt>", "<CR>", "<Esc>", "<Tab>", "<BS>",
        "<Del>", "<Insert>", "<Up>", "<Down>", "<Left>", "<Right>", "<Home>", "<End>",
        "<PageUp>", "<PageDown>", "<F1>", "<F12>", "<C-s>", "<A-h>", "<C-A-x>", "<S-Tab>",
        "<S-CR>", "<C-S-Up>", "<C-->", "<A-lt>", "<C-Space>",
    ] {
        let keys = parse_keys(text).unwrap_or_else(|e| panic!("{text:?}: {e}"));
        let printed = format_keys(&keys);
        assert_eq!(keys, parse_keys(&printed).unwrap(), "{text:?} printed as {printed:?}");
    }
}

#[test]
fn aliases_print_as_one_canonical_form() {
    assert_eq!("<CR>", one("<Enter>").to_string());
    assert_eq!("<CR>", one("<C-m>").to_string());
    assert_eq!("<Tab>", one("<C-i>").to_string());
    assert_eq!("<A-h>", one("<M-h>").to_string());
    assert_eq!("<Esc>", one("<Escape>").to_string());
    assert_eq!("<C-s>", one("<C-S>").to_string());
}

#[test]
fn keys_hash_the_same_when_equal() {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(key(KeyCode::Char('B'), SHIFT));
    assert!(set.contains(&ch('B')));
    set.insert(key(KeyCode::BackTab, SHIFT));
    assert!(set.contains(&one("<S-Tab>")));
}

#[test]
fn the_built_in_tables_build() {
    assert!(!builtin::navigation().entries().is_empty());
    assert!(!builtin::range_select().entries().is_empty());
}

#[test]
fn every_handler_has_a_key() {
    for handler in NavHandler::ALL {
        assert!(
            builtin::navigation().entries().iter().any(|(_, h)| h == handler),
            "{handler:?} has no key in navigation mode"
        );
    }
    for handler in RangeHandler::ALL {
        assert!(
            builtin::range_select().entries().iter().any(|(_, h)| h == handler),
            "{handler:?} has no key in range-select mode"
        );
    }
}

#[test]
fn every_handler_has_a_group_and_a_description() {
    for handler in NavHandler::ALL {
        assert!(!handler.group().is_empty(), "{handler:?}");
        assert!(!handler.description().is_empty(), "{handler:?}");
    }
    for handler in RangeHandler::ALL {
        assert!(!handler.group().is_empty(), "{handler:?}");
        assert!(!handler.description().is_empty(), "{handler:?}");
    }
}

#[test]
fn lookup_finds_a_match_a_prefix_or_nothing() {
    let nav = builtin::navigation();
    assert_eq!(Lookup::Match(NavHandler::MoveDown), nav.lookup(&[ch('j')]));
    assert_eq!(Lookup::Prefix, nav.lookup(&[ch('g')]));
    assert_eq!(Lookup::Match(NavHandler::MoveTop), nav.lookup(&[ch('g'), ch('g')]));
    assert_eq!(Lookup::None, nav.lookup(&[ch('g'), ch('j')]));
    assert_eq!(Lookup::None, nav.lookup(&[ch('z')]));
}

#[test]
fn ctrl_shift_c_is_a_copy_in_both_modes() {
    let k = key(KeyCode::Char('C'), CTRL);
    assert_eq!(
        Lookup::Match(NavHandler::CopyCellFormatted),
        builtin::navigation().lookup(&[k])
    );
    assert_eq!(
        Lookup::Match(RangeHandler::CopyRaw),
        builtin::range_select().lookup(&[k])
    );
    // And Ctrl-c is the other one, in the other order, as before.
    let k = key(KeyCode::Char('c'), CTRL);
    assert_eq!(
        Lookup::Match(NavHandler::CopyCell),
        builtin::navigation().lookup(&[k])
    );
    assert_eq!(
        Lookup::Match(RangeHandler::CopyFormatted),
        builtin::range_select().lookup(&[k])
    );
}

#[test]
fn range_select_has_hjkl_but_no_arrows() {
    let range = builtin::range_select();
    assert_eq!(Lookup::Match(RangeHandler::MoveDown), range.lookup(&[ch('j')]));
    assert_eq!(Lookup::None, range.lookup(&[Key::plain(KeyCode::Down)]));
}

fn keys(text: &str) -> Vec<Key> {
    parse_keys(text).unwrap_or_else(|e| panic!("{text:?}: {e}"))
}

fn replay(text: &str) -> Target {
    Target::Replay(parse_rhs(text).unwrap_or_else(|e| panic!("{text:?}: {e}")))
}

fn user_table(mappings: &[(&str, &str)]) -> UserTable {
    let mut table = UserTable::default();
    for (lhs, rhs) in mappings {
        table.insert(keys(lhs), replay(rhs));
    }
    table
}

#[test]
fn a_user_table_starts_empty() {
    let table = UserTable::default();
    assert!(table.is_empty());
    assert_eq!(UserLookup::None, table.lookup(&keys("j")));
}

#[test]
fn user_lookup_distinguishes_match_prefix_and_none() {
    let table = user_table(&[("<Space>c", "yj"), ("zz", "gg")]);
    let target = replay("yj");
    assert_eq!(UserLookup::Match(&target), table.lookup(&keys("<Space>c")));
    assert_eq!(UserLookup::Prefix, table.lookup(&keys("<Space>")));
    assert_eq!(UserLookup::Prefix, table.lookup(&keys("z")));
    assert_eq!(UserLookup::None, table.lookup(&keys("<Space>x")));
    assert_eq!(UserLookup::None, table.lookup(&keys("q")));
}

#[test]
fn a_mapping_that_starts_a_longer_one_is_match_or_more() {
    let table = user_table(&[("gt", "j"), ("gtt", "jj")]);
    let short = replay("j");
    assert_eq!(UserLookup::MatchOrMore(&short), table.lookup(&keys("gt")));
    let long = replay("jj");
    assert_eq!(UserLookup::Match(&long), table.lookup(&keys("gtt")));
    assert_eq!(UserLookup::Prefix, table.lookup(&keys("g")));
}

#[test]
fn inserting_the_same_keys_again_replaces_the_mapping() {
    let mut table = user_table(&[("x", "j")]);
    table.insert(keys("x"), replay("k"));
    assert_eq!(1, table.mappings().len());
    assert_eq!(replay("k"), table.mappings()[0].target);
}

#[test]
fn removing_a_mapping() {
    let mut table = user_table(&[("x", "j"), ("y", "k")]);
    assert!(table.remove(&keys("x")));
    assert!(!table.remove(&keys("x")));
    assert_eq!(1, table.mappings().len());
}

#[test]
fn longest_match_takes_the_longest_mapping_that_starts_the_keys() {
    let table = user_table(&[("a", "j"), ("abc", "jj")]);
    assert_eq!(Some(&replay("j")), table.longest_match(&keys("ab")));
    assert_eq!(Some(&replay("jj")), table.longest_match(&keys("abc")));
    assert_eq!(None, table.longest_match(&keys("b")));
}

#[test]
fn the_built_ins_see_keys_that_no_user_mapping_wants() {
    let user = user_table(&[("<Space>c", "yj")]);
    let nav = builtin::navigation();
    assert_eq!(
        Resolution::Builtin(NavHandler::MoveDown),
        resolve(&user, nav, &keys("j"))
    );
    // `g` is a built-in prefix: wait, but with no timeout.
    assert_eq!(
        Resolution::Wait { timeout: false },
        resolve(&user, nav, &keys("g"))
    );
    assert_eq!(Resolution::Drop, resolve(&user, nav, &keys("z")));
}

#[test]
fn a_user_mapping_wins_over_a_built_in() {
    let user = user_table(&[("j", "k")]);
    assert_eq!(
        Resolution::User(replay("k")),
        resolve(&user, builtin::navigation(), &keys("j"))
    );
}

#[test]
fn a_user_prefix_waits_with_a_timeout() {
    let user = user_table(&[("<Space>c", "yj")]);
    assert_eq!(
        Resolution::Wait { timeout: true },
        resolve(&user, builtin::navigation(), &keys("<Space>"))
    );
}

#[test]
fn a_user_mapping_shadows_the_built_ins_it_starts() {
    // Mapping `g` hides `gg`: `g` is a mapping now.
    let user = user_table(&[("g", "jj")]);
    assert_eq!(
        Resolution::User(replay("jj")),
        resolve(&user, builtin::navigation(), &keys("g"))
    );
    // `nnoremap gg gg` gets it back: `g` waits, `gg` is a mapping.
    let user = user_table(&[("g", "jj"), ("gg", "gg")]);
    assert_eq!(
        Resolution::Wait { timeout: true },
        resolve(&user, builtin::navigation(), &keys("g"))
    );
    assert_eq!(
        Resolution::User(replay("gg")),
        resolve(&user, builtin::navigation(), &keys("gg"))
    );
}

#[test]
fn unmapping_hides_a_built_in() {
    let mut user = UserTable::default();
    user.insert(keys("q"), Target::Unmapped);
    assert_eq!(
        Resolution::User(Target::Unmapped),
        resolve(&user, builtin::navigation(), &keys("q"))
    );
}

#[test]
fn an_empty_user_table_never_waits_on_a_user_timeout() {
    let user = UserTable::default();
    let nav = builtin::navigation();
    assert_eq!(Resolution::Wait { timeout: false }, resolve(&user, nav, &keys("g")));
    assert_eq!(Resolution::Builtin(NavHandler::MoveTop), resolve(&user, nav, &keys("gg")));
}

#[test]
fn the_timeout_runs_the_longest_mapping_and_drops_nothing_else_it_cannot_run() {
    let user = user_table(&[("gt", "j"), ("gtt", "jj")]);
    assert_eq!(Some(replay("j")), resolve_timeout(&user, &keys("gt")));
    assert_eq!(None, resolve_timeout(&user, &keys("g")));
}

fn rk(text: &str) -> RhsItem {
    RhsItem::Key(one(text))
}

#[test]
fn a_right_hand_side_of_keys() {
    assert_eq!(vec![rk("y"), rk("j"), rk("p")], parse_rhs("yjp").unwrap());
    assert_eq!(vec![rk("<C-s>"), rk("<Down>")], parse_rhs("<C-s><Down>").unwrap());
    assert_eq!(Vec::<RhsItem>::new(), parse_rhs("").unwrap());
}

#[test]
fn a_right_hand_side_with_a_command() {
    assert_eq!(
        vec![RhsItem::Cmd("write".to_owned())],
        parse_rhs("<Cmd>write<CR>").unwrap()
    );
    assert_eq!(
        vec![RhsItem::Cmd("sort skip 1".to_owned())],
        parse_rhs("<Cmd>sort skip 1<CR>").unwrap()
    );
}

#[test]
fn keys_and_commands_mix() {
    assert_eq!(
        vec![
            rk("j"),
            RhsItem::Cmd("write".to_owned()),
            rk("k"),
            RhsItem::Cmd("undo".to_owned()),
        ],
        parse_rhs("j<Cmd>write<CR>k<Cmd>undo<CR>").unwrap()
    );
}

#[test]
fn cmd_tokens_ignore_case_and_the_text_is_trimmed() {
    assert_eq!(
        vec![RhsItem::Cmd("Write Foo.sui".to_owned())],
        parse_rhs("<cmd>  Write Foo.sui <CR>").unwrap()
    );
    assert_eq!(
        vec![RhsItem::Cmd("write".to_owned())],
        parse_rhs("<CMD>write<cr>").unwrap()
    );
}

#[test]
fn a_command_needs_text_and_a_closing_cr() {
    assert!(parse_rhs("<Cmd>write").unwrap_err().to_string().contains("closing <CR>"));
    assert!(parse_rhs("<Cmd><CR>").unwrap_err().to_string().contains("no command"));
    assert!(parse_rhs("<Cmd>   <CR>").is_err());
    assert!(parse_rhs("j<Nope>").is_err());
}

#[test]
fn a_right_hand_side_prints_back() {
    for text in ["yj", "<Space><C-s>", "<Cmd>sort skip 1<CR>", "j<Cmd>write<CR>k"] {
        assert_eq!(text, format_rhs(&parse_rhs(text).unwrap()));
    }
}

fn parsed(text: &str) -> MapCommand {
    command::parse(text)
        .unwrap_or_else(|e| panic!("{text:?}: {e}"))
        .unwrap_or_else(|| panic!("{text:?} is not a mapping command"))
}

fn command_error(text: &str) -> String {
    command::parse(text).expect_err(text)
}

#[test]
fn the_four_commands_parse() {
    assert_eq!(
        MapCommand::Map {
            mode: MapMode::Navigate,
            lhs: keys("<Space>c"),
            rhs: parse_rhs("yj").unwrap(),
        },
        parsed("nnoremap <Space>c yj")
    );
    assert_eq!(
        MapCommand::Map {
            mode: MapMode::RangeSelect,
            lhs: keys("<Space>s"),
            rhs: parse_rhs("<Cmd>sort<CR>").unwrap(),
        },
        parsed("vnoremap <Space>s <Cmd>sort<CR>")
    );
    assert_eq!(
        MapCommand::Unmap { mode: MapMode::Navigate, lhs: keys("q") },
        parsed("nunmap q")
    );
    assert_eq!(
        MapCommand::Unmap { mode: MapMode::RangeSelect, lhs: keys("<C-n>") },
        parsed("vunmap <C-n>")
    );
}

#[test]
fn other_commands_are_not_mapping_commands() {
    for text in ["write foo", "quit", "new-sheet x", "nnoremapx a b", "noremap a b", "nmap", ""] {
        assert_eq!(Ok(None), command::parse(text), "{text:?}");
    }
}

#[test]
fn whitespace_around_a_mapping_command_is_forgiven() {
    let expected = parsed("nnoremap x j");
    assert_eq!(expected, parsed("  nnoremap   x    j  "));
    assert_eq!(expected, parsed("nnoremap\tx\tj"));
}

#[test]
fn a_right_hand_side_keeps_its_inner_spaces() {
    let cmd = parsed("nnoremap x <Cmd>sort skip 1<CR>");
    let MapCommand::Map { rhs, .. } = cmd else { panic!("not a map") };
    assert_eq!(vec![RhsItem::Cmd("sort skip 1".to_owned())], rhs);
}

#[test]
fn a_hash_in_a_mapping_is_a_key() {
    let cmd = parsed("nnoremap <Space>h #");
    let MapCommand::Map { rhs, .. } = cmd else { panic!("not a map") };
    assert_eq!(vec![rk("#")], rhs);
}

#[test]
fn a_mapping_command_needs_its_arguments() {
    assert!(command_error("nnoremap").contains("Usage"));
    assert!(command_error("nnoremap x").contains("Usage"));
    assert!(command_error("nunmap").contains("Usage"));
    assert!(command_error("nunmap x y").contains("Usage"));
}

#[test]
fn bad_notation_names_the_command_and_the_problem() {
    let e = command_error("nnoremap <Nope> j");
    assert!(e.contains("nnoremap") && e.contains("not a key name"), "{e}");
    let e = command_error("nnoremap x <Cmd>write");
    assert!(e.contains("closing <CR>"), "{e}");
    let e = command_error("vunmap <C-S-x>");
    assert!(e.contains("terminal cannot report"), "{e}");
}

#[test]
fn a_mapping_cannot_start_with_a_digit_or_esc() {
    for text in ["nnoremap 1 j", "nnoremap 0j k", "nunmap 5", "vnoremap 9 j"] {
        let e = command_error(text);
        assert!(e.contains("digits are the numeric prefix"), "{text}: {e}");
    }
    for text in ["nnoremap <Esc> j", "nnoremap <Esc>x j", "nunmap <Esc>"] {
        let e = command_error(text);
        assert!(e.contains("<Esc> cannot start a mapping"), "{text}: {e}");
    }
    // A digit later in the keys is fine.
    assert!(command::parse("nnoremap g1 j").is_ok());
    // And a digit on the right is fine.
    assert!(command::parse("nnoremap x 3j").is_ok());
}

#[test]
fn apply_maps_and_replaces() {
    let mut map = KeyMap::default();
    map.apply(parsed("nnoremap x j")).unwrap();
    map.apply(parsed("nnoremap x k")).unwrap();
    map.apply(parsed("vnoremap x l")).unwrap();
    assert_eq!(1, map.nav.mappings().len());
    assert_eq!(replay("k"), map.nav.mappings()[0].target);
    assert_eq!(replay("l"), map.range.mappings()[0].target);
}

#[test]
fn apply_unmap_removes_a_user_mapping_and_the_built_in_returns() {
    let mut map = KeyMap::default();
    map.apply(parsed("nnoremap j k")).unwrap();
    map.apply(parsed("nunmap j")).unwrap();
    assert!(map.nav.is_empty());
}

#[test]
fn apply_unmap_hides_a_built_in() {
    let mut map = KeyMap::default();
    map.apply(parsed("nunmap q")).unwrap();
    assert_eq!(Target::Unmapped, map.nav.mappings()[0].target);
    // Again: already hidden, not an error, and still hidden.
    map.apply(parsed("nunmap q")).unwrap();
    assert_eq!(Target::Unmapped, map.nav.mappings()[0].target);
    // The range mode table is separate: `q` is not a key there.
    assert!(map.apply(parsed("vunmap q")).is_err());
}

#[test]
fn apply_unmap_of_nothing_is_an_error() {
    let mut map = KeyMap::default();
    let e = map.apply(parsed("nunmap z")).unwrap_err();
    assert_eq!("No mapping for z", e);
    // The start of a built-in sequence is not a built-in key.
    assert!(map.apply(parsed("nunmap g")).is_err());
    assert!(map.nav.is_empty());
}
