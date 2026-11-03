use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::{
    builtin::{self, Lookup, NavHandler, RangeHandler},
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
