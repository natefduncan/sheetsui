use std::path::PathBuf;

use super::*;
use crate::ui::{keys::map::KeyMap, options::Options};

fn run(text: &str) -> (Options, KeyMap, Vec<Warning>) {
    let mut options = Options::default();
    let mut keymap = KeyMap::default();
    let warnings = load(text, "config", &mut options, &mut keymap, &mut None);
    (options, keymap, warnings)
}

fn dir(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("sheetui-config-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn test_set_lines_apply_to_the_options() {
    let (options, _, warnings) = run("set ic scs\nset searchin=raw\nset nowrapscan\n");
    assert_eq!(Vec::<Warning>::new(), warnings);
    assert!(options.ignorecase());
    assert!(options.smartcase());
    assert!(!options.wrapscan());
    assert_eq!(crate::book::search::SearchIn::Raw, options.searchin());
}

#[test]
fn test_mappings_and_unmaps_apply() {
    let (_, keymap, warnings) = run("nnoremap <Space> jj\nvnoremap z k\nnunmap q\n");
    assert_eq!(Vec::<Warning>::new(), warnings);
    assert_eq!(2, keymap.nav.mappings().len());
    assert_eq!(1, keymap.range.mappings().len());
}

#[test]
fn test_comments_blanks_and_leading_colon() {
    let (options, keymap, warnings) =
        run("# a comment\n\n   \n:set ic\n  :nnoremap a j\nnnoremap <Space>h #\n");
    assert_eq!(Vec::<Warning>::new(), warnings);
    assert!(options.ignorecase());
    assert_eq!(2, keymap.nav.mappings().len());
}

#[test]
fn test_disallowed_commands_warn_with_line_and_later_lines_run() {
    let (options, _, warnings) = run("write x\nsort\ns/a/b/\nset ic\n");
    assert!(options.ignorecase(), "the lines after the bad ones still run");
    let lines: Vec<usize> = warnings.iter().map(|w| w.line).collect();
    assert_eq!(vec![1, 2, 3], lines);
    assert!(warnings.iter().all(|w| w.source == "config"));
    assert!(warnings[0].msg.contains("write"), "{}", warnings[0].msg);
    assert!(warnings[0].msg.contains("config file"), "{}", warnings[0].msg);
    assert_eq!("config:1: ".len() + warnings[0].msg.len(), warnings[0].to_string().len());
}

#[test]
fn test_bad_options_are_one_warning_and_apply_nothing_on_that_line() {
    let (options, _, warnings) = run("set foo\nset ws=1\nset searchin=nope\n");
    assert_eq!(3, warnings.len());
    assert!(warnings[0].msg.contains("E518"), "{}", warnings[0].msg);
    assert!(warnings[1].msg.contains("E474") || warnings[1].msg.contains("E518"));
    assert!(warnings[2].msg.contains("E474"), "{}", warnings[2].msg);
    assert!(options.wrapscan());
    assert!(!options.ignorecase());
}

#[test]
fn test_bad_mapping_warns_and_theme_and_unknown_lines() {
    let (_, keymap, warnings) = run("nnoremap <Nope> j\ntheme dracula\nother 1\nnnoremap a j\n");
    assert_eq!(1, keymap.nav.mappings().len());
    let lines: Vec<usize> = warnings.iter().map(|w| w.line).collect();
    assert_eq!(vec![1, 3], lines);
}

#[test]
fn test_a_theme_only_file_has_no_warnings() {
    let (_, _, warnings) = run("theme dracula\n");
    assert!(warnings.is_empty());
}

#[test]
fn test_load_file_reads_config_in_the_dir() {
    let dir = dir("file");
    std::fs::write(dir.join("config"), "set ic\n").unwrap();
    let mut options = Options::default();
    let mut keymap = KeyMap::default();
    let loaded = load_file(Some(&dir), &mut options, &mut keymap);
    assert!(options.ignorecase());
    assert_eq!(Some(dir.join("config")), loaded.path);
    assert!(loaded.warnings.is_empty());
}

#[test]
fn test_missing_default_file_is_silent_and_no_dir_is_too() {
    let dir = dir("missing");
    let mut options = Options::default();
    let mut keymap = KeyMap::default();
    assert_eq!(Loaded::default(), load_file(Some(&dir), &mut options, &mut keymap));
    assert_eq!(Loaded::default(), load_file(None, &mut options, &mut keymap));
}

#[test]
fn test_unreadable_config_is_a_warning() {
    let dir = dir("unreadable");
    std::fs::create_dir(dir.join("config")).unwrap();
    let mut options = Options::default();
    let mut keymap = KeyMap::default();
    let loaded = load_file(Some(&dir), &mut options, &mut keymap);
    assert_eq!(1, loaded.warnings.len());
    assert!(loaded.warnings[0].msg.contains("cannot read"));
}

#[test]
fn test_theme_lines_set_the_theme_and_the_last_one_wins() {
    let mut theme = None;
    let warnings = load(
        "theme light\n:theme dracula\n",
        "config",
        &mut Options::default(),
        &mut KeyMap::default(),
        &mut theme,
    );
    assert!(warnings.is_empty());
    assert_eq!(Some("dracula".to_owned()), theme);
}

#[test]
fn test_bare_theme_line_warns_and_keeps_the_earlier_theme() {
    let mut theme = None;
    let warnings = load(
        "theme light\ntheme\n",
        "config",
        &mut Options::default(),
        &mut KeyMap::default(),
        &mut theme,
    );
    assert_eq!(vec![2], warnings.iter().map(|w| w.line).collect::<Vec<_>>());
    assert_eq!(Some("light".to_owned()), theme);
}
