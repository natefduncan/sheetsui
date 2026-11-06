use std::process::ExitCode;

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use tui_prompts::State;
use serial_test::serial;

use crate::book;
use crate::book::DecimalsChange;
use crate::book::search::Direction;
use ironcalc::base::types::Color;
use crate::ui::cmd::parse_color;
use crate::ui::keys::{
    map::{parse_rhs, Target},
    parse_keys,
};
use crate::ui::{Address, ClipboardContents, Modality};

use super::cmd::{parse, parse_style_attrs, Cmd, CmdSortKey};
use crate::book::{StyleAttr, StyleFlag};
use super::render::markdown::Markdown;
use super::{parse_csv_rows, Workspace};

#[derive(Default)]
pub struct InputScript {
    events: Vec<Event>,
}

impl InputScript {
    pub fn char(self, c: char) -> Self {
        self.event(construct_key_event(KeyCode::Char(c)))
    }

    pub fn chars(self, cs: &str) -> Self {
        cs.chars().fold(self, |s, c| s.char(c))
    }

    pub fn ctrl(self, c: char) -> Self {
        self.modified_char(c, KeyModifiers::CONTROL)
    }

    pub fn alt(self, c: char) -> Self {
        self.modified_char(c, KeyModifiers::ALT)
    }

    pub fn tab(self) -> Self {
        self.event(construct_key_event(KeyCode::Tab))
    }

    pub fn del(self) -> Self {
        self.event(construct_key_event(KeyCode::Delete))
    }

    pub fn shift_del(self) -> Self {
        self.event(construct_modified_key_event(KeyCode::Delete, KeyModifiers::SHIFT))
    }

    pub fn enter(self) -> Self {
        self.event(construct_key_event(KeyCode::Enter))
    }

    pub fn modified_char(self, c: char, mods: KeyModifiers) -> Self {
        self.event(construct_modified_key_event(KeyCode::Char(c), mods))
    }

    pub fn event(mut self, evt: Event) -> Self {
        self.events.push(evt);
        self
    }

    pub fn esc(self) -> Self {
        self.event(construct_key_event(KeyCode::Esc))
    }

    pub fn run(self, ws: &mut Workspace) -> anyhow::Result<Option<ExitCode>> {
        for evt in self.events {
            if let Some(e) = ws.handle_input(evt)? {
                return Ok(Some(e));
            }
        }
        Ok(None)
    }
}

fn script() -> InputScript {
    InputScript::default()
}

fn construct_key_event(code: KeyCode) -> Event {
    construct_modified_key_event(code, KeyModifiers::empty())
}

fn construct_modified_key_event(code: KeyCode, mods: KeyModifiers) -> Event {
    Event::Key(KeyEvent::new(code, mods))
}

#[test]
fn test_write_cmd() {
    let input = "write foo.xlsx";
    let result = parse(input);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.is_some());
    let cmd = output.unwrap();
    assert_eq!(cmd, Cmd::Write(Some("foo.xlsx")));
}

#[test]
fn test_short_write_cmd() {
    let input = "w foo.xlsx";
    let result = parse(input);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.is_some());
    let cmd = output.unwrap();
    assert_eq!(cmd, Cmd::Write(Some("foo.xlsx")));
}

#[test]
fn test_insert_rows_cmd() {
    let input = "insert-rows 1";
    let result = parse(input);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.is_some());
    let cmd = output.unwrap();
    assert_eq!(cmd, Cmd::InsertRows(1));
}

#[test]
fn test_insert_rows_cmd_short() {
    let input = "ir 1";
    let result = parse(input);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.is_some());
    let cmd = output.unwrap();
    assert_eq!(cmd, Cmd::InsertRows(1));
}

#[test]
fn test_insert_cols_cmd() {
    let input = "insert-cols 1";
    let result = parse(input);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.is_some());
    let cmd = output.unwrap();
    assert_eq!(cmd, Cmd::InsertColumns(1));
}

#[test]
fn test_insert_cols_cmd_short() {
    let input = "ic 1";
    let result = parse(input);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.is_some());
    let cmd = output.unwrap();
    assert_eq!(cmd, Cmd::InsertColumns(1));
}

#[test]
fn test_edit_cmd() {
    let input = "edit path.txt";
    let result = parse(input);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.is_some());
    let cmd = output.unwrap();
    assert_eq!(cmd, Cmd::Edit("path.txt"));
}

#[test]
fn test_edit_cmd_short() {
    let input = "e path.txt";
    let result = parse(input);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.is_some());
    let cmd = output.unwrap();
    assert_eq!(cmd, Cmd::Edit("path.txt"));
}

#[test]
fn test_help_cmd() {
    let input = "help topic";
    let result = parse(input);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.is_some());
    let cmd = output.unwrap();
    assert_eq!(cmd, Cmd::Help(Some("topic")));
}

#[test]
fn test_help_cmd_short() {
    let input = "? topic";
    let result = parse(input);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.is_some());
    let cmd = output.unwrap();
    assert_eq!(cmd, Cmd::Help(Some("topic")));
}

#[test]
fn test_quit_cmd_short() {
    let input = "q";
    let result = parse(input);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.is_some());
    let cmd = output.unwrap();
    assert_eq!(cmd, Cmd::Quit);
}

#[test]
fn test_quit_cmd() {
    let input = "quit";
    let result = parse(input);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.is_some());
    let cmd = output.unwrap();
    assert_eq!(cmd, Cmd::Quit);
}

#[test]
fn test_cmd_new_sheet_with_name() {
    let input = "new-sheet test";
    let result = parse(input);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.is_some());
    let cmd = output.unwrap();
    assert_eq!(cmd, Cmd::NewSheet(Some("test")));
}

#[test]
fn test_cmd_export() {
    let input = "export-csv test.csv";
    let result = parse(input);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.is_some());
    let cmd = output.unwrap();
    assert_eq!(cmd, Cmd::ExportCsv("test.csv"));
}

#[test]
fn test_cmd_new_sheet_no_name() {
    let input = "new-sheet";
    let result = parse(input);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.is_some());
    let cmd = output.unwrap();
    assert_eq!(cmd, Cmd::NewSheet(None));
}

#[test]
fn test_cmd_select_sheet_with_name() {
    let input = "select-sheet test";
    let result = parse(input);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.is_some());
    let cmd = output.unwrap();
    assert_eq!(cmd, Cmd::SelectSheet("test"));
}

#[test]
fn test_cmd_rename_sheet_with_name() {
    let input = "rename-sheet test";
    let result = parse(input);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.is_some());
    let cmd = output.unwrap();
    assert_eq!(cmd, Cmd::RenameSheet(None, "test"));
}

#[test]
fn test_cmd_rename_sheet_with_idx_and_name() {
    let input = "rename-sheet 0 test";
    let result = parse(input);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.is_some());
    let cmd = output.unwrap();
    assert_eq!(cmd, Cmd::RenameSheet(Some(0), "test"));
}

#[test]
fn test_cmd_color_rows_with_color() {
    let input = "color-rows red";
    let result = parse(input);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.is_some());
    let cmd = output.unwrap();
    assert_eq!(cmd, Cmd::ColorRows(None, parse_color("red").unwrap()));
}

#[test]
fn test_cmd_color_rows_with_idx_and_color() {
    let input = "color-rows 1 red";
    let result = parse(input);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.is_some());
    let cmd = output.unwrap();
    assert_eq!(cmd, Cmd::ColorRows(Some(1), parse_color("red").unwrap()));
}

#[test]
fn test_cmd_color_columns_with_color() {
    let input = "color-columns red";
    let result = parse(input);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.is_some());
    let cmd = output.unwrap();
    assert_eq!(cmd, Cmd::ColorColumns(None, parse_color("red").unwrap()));
}

#[test]
fn test_cmd_color_columns_with_idx_and_color() {
    let input = "color-columns 1 red";
    let result = parse(input);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.is_some());
    let cmd = output.unwrap();
    assert_eq!(cmd, Cmd::ColorColumns(Some(1), parse_color("red").unwrap()));
}

#[test]
fn test_input_navitation_enter_key() {
    let mut ws = new_workspace();
    let row = ws.book.location.row;
    assert_eq!(Some(&Modality::Navigate), ws.state.modality_stack.last());
    script()
        .enter()
        .run(&mut ws)
        .expect("Failed to handle enter key");
    assert_eq!(row + 1, ws.book.location.row);
}

#[test]
fn test_input_navitation_tab_key() {
    let mut ws = new_workspace();
    let col = ws.book.location.col;
    assert_eq!(Some(&Modality::Navigate), ws.state.modality_stack.last());
    script()
        .tab()
        .run(&mut ws)
        .expect("Failed to handle enter key");
    assert_eq!(col + 1, ws.book.location.col);
}

#[test]
fn test_input_navitation_shift_enter_key() {
    let mut ws = new_workspace();
    let row = ws.book.location.row;
    assert_eq!(Some(&Modality::Navigate), ws.state.modality_stack.last());
    script()
        .enter()
        .run(&mut ws)
        .expect("Failed to handle enter key");
    assert_eq!(row + 1, ws.book.location.row);
    script()
        .event(construct_modified_key_event(
            KeyCode::Enter,
            KeyModifiers::SHIFT,
        ))
        .run(&mut ws)
        .expect("Failed to handle enter key");
    assert_eq!(row, ws.book.location.row);
}

#[test]
fn test_input_navitation_shift_tab_key() {
    let mut ws = new_workspace();
    let col = ws.book.location.col;
    assert_eq!(Some(&Modality::Navigate), ws.state.modality_stack.last());
    script()
        .tab()
        .run(&mut ws)
        .expect("Failed to handle enter key");
    assert_eq!(col + 1, ws.book.location.col);
    script()
        .event(construct_modified_key_event(
            KeyCode::Tab,
            KeyModifiers::SHIFT,
        ))
        .run(&mut ws)
        .expect("Failed to handle enter key");
    assert_eq!(col, ws.book.location.col);
}

#[test]
fn test_back_tab_moves_left() {
    // Most terminals send Shift+Tab as BackTab.
    let mut ws = new_workspace();
    let col = ws.book.location.col;
    script().tab().run(&mut ws).expect("Failed to run script");
    assert_eq!(col + 1, ws.book.location.col);
    script()
        .event(construct_modified_key_event(
            KeyCode::BackTab,
            KeyModifiers::SHIFT,
        ))
        .run(&mut ws)
        .expect("Failed to run script");
    assert_eq!(col, ws.book.location.col);
}

#[test]
fn test_gg_goes_to_top() {
    let mut ws = new_workspace();
    let row = ws.book.location.row;
    script().chars("jjj").run(&mut ws).expect("Failed to run script");
    assert_eq!(row + 3, ws.book.location.row);
    script().chars("gg").run(&mut ws).expect("Failed to run script");
    assert_eq!(row, ws.book.location.row);
    assert!(ws.state.pending_keys.is_empty());
}

#[test]
fn test_g_waits_for_the_next_key() {
    let mut ws = new_workspace();
    script().chars("jjjg").run(&mut ws).expect("Failed to run script");
    assert_eq!(1, ws.state.pending_keys.len());
}

#[test]
fn test_g_then_another_key_drops_both() {
    let mut ws = new_workspace();
    let row = ws.book.location.row;
    script().chars("jjjgj").run(&mut ws).expect("Failed to run script");
    // The `j` after `g` is swallowed: `gj` is not a key.
    assert_eq!(row + 3, ws.book.location.row);
    assert!(ws.state.pending_keys.is_empty());
    // And the `g` does not wait around for a later `g`.
    script().char('g').run(&mut ws).expect("Failed to run script");
    assert_eq!(row + 3, ws.book.location.row);
}

#[test]
fn test_esc_cancels_a_pending_g() {
    let mut ws = new_workspace();
    let row = ws.book.location.row;
    script()
        .chars("jjjg")
        .esc()
        .char('g')
        .run(&mut ws)
        .expect("Failed to run script");
    assert_eq!(row + 3, ws.book.location.row);
    assert_eq!(1, ws.state.pending_keys.len());
}

#[test]
fn test_a_digit_after_g_is_not_a_numeric_prefix() {
    let mut ws = new_workspace();
    let row = ws.book.location.row;
    script().chars("g5j").run(&mut ws).expect("Failed to run script");
    assert!(ws.state.numeric_prefix.is_empty());
    assert_eq!(row + 1, ws.book.location.row);
}

#[test]
fn test_numeric_prefix_applies_to_a_move() {
    let mut ws = new_workspace();
    let row = ws.book.location.row;
    script().chars("12j").run(&mut ws).expect("Failed to run script");
    assert_eq!(row + 12, ws.book.location.row);
}

#[test]
fn test_extra_modifiers_do_not_fire_a_bare_key() {
    // Before the key table these moved, because the arms only excluded Ctrl.
    let mut ws = new_workspace();
    let (row, col) = (ws.book.location.row, ws.book.location.col);
    script()
        .alt('j')
        .alt('l')
        .event(construct_modified_key_event(
            KeyCode::Down,
            KeyModifiers::SHIFT,
        ))
        .event(construct_modified_key_event(
            KeyCode::Right,
            KeyModifiers::ALT,
        ))
        .run(&mut ws)
        .expect("Failed to run script");
    assert_eq!((row, col), (ws.book.location.row, ws.book.location.col));
    assert_eq!(Some(&Modality::Navigate), ws.state.modality_stack.last());
}

#[test]
fn test_a_modified_digit_is_not_a_numeric_prefix() {
    let mut ws = new_workspace();
    script().alt('5').run(&mut ws).expect("Failed to run script");
    assert!(ws.state.numeric_prefix.is_empty());
}

#[test]
fn test_bare_and_ctrl_keys_are_different_keys() {
    // `s` clears and edits, `r` redoes, `v` selects, `h` moves, `l` moves.
    // Ctrl-r selects a range and Alt-h opens help.
    let mut ws = new_workspace();
    script().char('s').run(&mut ws).expect("Failed to run script");
    assert_eq!(Some(&Modality::CellEdit), ws.state.modality_stack.last());

    let mut ws = new_workspace();
    script().char('r').run(&mut ws).expect("Failed to run script");
    assert_eq!(Some(&Modality::Navigate), ws.state.modality_stack.last());
    script().alt('r').run(&mut ws).expect("Failed to run script");
    assert_eq!(Some(&Modality::RangeSelect), ws.state.modality_stack.last());

    let mut ws = new_workspace();
    script().alt('h').run(&mut ws).expect("Failed to run script");
    assert_eq!(Some(&Modality::Dialog), ws.state.modality_stack.last());
}

#[test]
fn test_ctrl_h_and_ctrl_l_resize_the_column() {
    let mut ws = new_workspace();
    let col = ws.book.location.col;
    let size = ws.book.get_col_size(col).expect("no column size");
    script().ctrl('l').run(&mut ws).expect("Failed to run script");
    assert_eq!(size + 1, ws.book.get_col_size(col).expect("no column size"));
    script().ctrl('h').run(&mut ws).expect("Failed to run script");
    assert_eq!(size, ws.book.get_col_size(col).expect("no column size"));
    // Neither moved the cursor.
    assert_eq!(col, ws.book.location.col);
}

#[test]
fn test_ctrl_n_and_ctrl_p_switch_sheets() {
    let mut ws = new_workspace();
    ws.book.new_sheet(Some("Sheet2")).expect("failed to create sheet2");
    let sheet = ws.book.location.sheet;
    script().ctrl('n').run(&mut ws).expect("Failed to run script");
    assert_ne!(sheet, ws.book.location.sheet);
    script().ctrl('p').run(&mut ws).expect("Failed to run script");
    assert_eq!(sheet, ws.book.location.sheet);
}

#[test]
fn test_range_select_digits_are_a_numeric_prefix() {
    let mut ws = new_workspace();
    let row = ws.book.location.row;
    script().char('v').run(&mut ws).expect("Failed to run script");
    assert_eq!(Some(&Modality::RangeSelect), ws.state.modality_stack.last());
    script().chars("4j").run(&mut ws).expect("Failed to run script");
    assert_eq!(row + 4, ws.book.location.row);
}

#[test]
fn test_range_select_esc_clears_a_numeric_prefix_before_leaving() {
    let mut ws = new_workspace();
    script().chars("v4").run(&mut ws).expect("Failed to run script");
    script().esc().run(&mut ws).expect("Failed to run script");
    assert_eq!(Some(&Modality::RangeSelect), ws.state.modality_stack.last());
    assert!(ws.state.numeric_prefix.is_empty());
    script().esc().run(&mut ws).expect("Failed to run script");
    assert_eq!(Some(&Modality::Navigate), ws.state.modality_stack.last());
}

#[test]
fn test_range_select_does_not_take_the_arrow_keys() {
    // Unchanged from before the key table: only hjkl move in range-select.
    let mut ws = new_workspace();
    let row = ws.book.location.row;
    script()
        .char('v')
        .event(construct_key_event(KeyCode::Down))
        .run(&mut ws)
        .expect("Failed to run script");
    assert_eq!(row, ws.book.location.row);
}

fn map_nav(ws: &mut Workspace, lhs: &str, rhs: &str) {
    ws.state.keymap.nav.insert(
        parse_keys(lhs).expect("bad lhs"),
        Target::Replay(parse_rhs(rhs).expect("bad rhs")),
    );
}

fn map_range(ws: &mut Workspace, lhs: &str, rhs: &str) {
    ws.state.keymap.range.insert(
        parse_keys(lhs).expect("bad lhs"),
        Target::Replay(parse_rhs(rhs).expect("bad rhs")),
    );
}

#[test]
fn test_user_mapping_runs() {
    let mut ws = new_workspace();
    let row = ws.book.location.row;
    map_nav(&mut ws, "<Space>", "jj");
    script().char(' ').run(&mut ws).expect("Failed to run script");
    assert_eq!(row + 2, ws.book.location.row);
    assert!(ws.state.pending_keys.is_empty());
    assert_eq!(None, ws.key_deadline());
}

#[test]
fn test_user_mapping_wins_over_a_built_in() {
    let mut ws = new_workspace();
    let (row, col) = (ws.book.location.row, ws.book.location.col);
    map_nav(&mut ws, "j", "l");
    script().char('j').run(&mut ws).expect("Failed to run script");
    assert_eq!((row, col + 1), (ws.book.location.row, ws.book.location.col));
}

#[test]
fn test_swapping_two_keys_does_not_loop() {
    let mut ws = new_workspace();
    let row = ws.book.location.row;
    map_nav(&mut ws, "j", "k");
    map_nav(&mut ws, "k", "j");
    script().char('k').run(&mut ws).expect("Failed to run script");
    assert_eq!(row + 1, ws.book.location.row);
    script().char('j').run(&mut ws).expect("Failed to run script");
    assert_eq!(row, ws.book.location.row);
}

#[test]
fn test_replayed_keys_do_not_trigger_user_mappings() {
    let mut ws = new_workspace();
    let row = ws.book.location.row;
    map_nav(&mut ws, "j", "jj");
    script().char('j').run(&mut ws).expect("Failed to run script");
    assert_eq!(row + 2, ws.book.location.row);
}

#[test]
fn test_a_user_prefix_waits_then_runs() {
    let mut ws = new_workspace();
    let row = ws.book.location.row;
    map_nav(&mut ws, "gt", "jj");
    script().char('g').run(&mut ws).expect("Failed to run script");
    assert_eq!(1, ws.state.pending_keys.len());
    assert!(ws.key_deadline().is_some());
    script().char('t').run(&mut ws).expect("Failed to run script");
    assert_eq!(row + 2, ws.book.location.row);
    assert!(ws.state.pending_keys.is_empty());
    assert_eq!(None, ws.key_deadline());
}

#[test]
fn test_gg_still_works_next_to_a_user_gt() {
    let mut ws = new_workspace();
    let row = ws.book.location.row;
    map_nav(&mut ws, "gt", "l");
    script().chars("jjjgg").run(&mut ws).expect("Failed to run script");
    assert_eq!(row, ws.book.location.row);
}

#[test]
fn test_timeout_runs_a_mapping_that_a_longer_one_starts() {
    let mut ws = new_workspace();
    let (row, col) = (ws.book.location.row, ws.book.location.col);
    map_nav(&mut ws, "<Space>", "j");
    map_nav(&mut ws, "<Space>w", "l");
    script().char(' ').run(&mut ws).expect("Failed to run script");
    assert_eq!(row, ws.book.location.row, "must wait for a possible <Space>w");
    ws.handle_key_timeout().expect("timeout failed");
    assert_eq!(row + 1, ws.book.location.row);
    assert_eq!(col, ws.book.location.col);
    assert!(ws.state.pending_keys.is_empty());
    assert_eq!(None, ws.key_deadline());
}

#[test]
fn test_the_longer_mapping_wins_when_its_keys_arrive() {
    let mut ws = new_workspace();
    let (row, col) = (ws.book.location.row, ws.book.location.col);
    map_nav(&mut ws, "<Space>", "j");
    map_nav(&mut ws, "<Space>w", "l");
    script().chars(" w").run(&mut ws).expect("Failed to run script");
    assert_eq!((row, col + 1), (ws.book.location.row, ws.book.location.col));
}

#[test]
fn test_timeout_with_no_mapping_drops_the_keys() {
    let mut ws = new_workspace();
    let row = ws.book.location.row;
    map_nav(&mut ws, "gt", "jj");
    script().char('g').run(&mut ws).expect("Failed to run script");
    ws.handle_key_timeout().expect("timeout failed");
    assert!(ws.state.pending_keys.is_empty());
    assert_eq!(row, ws.book.location.row);
}

#[test]
fn test_a_built_in_prefix_never_times_out() {
    let mut ws = new_workspace();
    script().char('g').run(&mut ws).expect("Failed to run script");
    assert_eq!(None, ws.key_deadline());
    ws.handle_key_timeout().expect("timeout failed");
    assert_eq!(1, ws.state.pending_keys.len());
}

#[test]
fn test_esc_cancels_a_pending_user_prefix() {
    let mut ws = new_workspace();
    map_nav(&mut ws, "gt", "jj");
    script().char('g').esc().run(&mut ws).expect("Failed to run script");
    assert!(ws.state.pending_keys.is_empty());
    assert_eq!(None, ws.key_deadline());
}

#[test]
fn test_a_key_that_breaks_a_user_prefix_drops_both() {
    let mut ws = new_workspace();
    let row = ws.book.location.row;
    map_nav(&mut ws, "gt", "jj");
    script().chars("gx").run(&mut ws).expect("Failed to run script");
    assert!(ws.state.pending_keys.is_empty());
    assert_eq!(None, ws.key_deadline());
    assert_eq!(row, ws.book.location.row);
}

#[test]
fn test_mapping_g_shadows_gg_and_a_gg_mapping_restores_it() {
    let mut ws = new_workspace();
    let row = ws.book.location.row;
    map_nav(&mut ws, "g", "jj");
    script().chars("gg").run(&mut ws).expect("Failed to run script");
    assert_eq!(row + 4, ws.book.location.row, "each g is the mapping");

    let mut ws = new_workspace();
    let row = ws.book.location.row;
    map_nav(&mut ws, "g", "jj");
    map_nav(&mut ws, "gg", "gg");
    script().chars("jjjgg").run(&mut ws).expect("Failed to run script");
    assert_eq!(row, ws.book.location.row, "gg is back");
}

#[test]
fn test_unmapping_a_built_in_disables_it() {
    let mut ws = new_workspace();
    ws.state
        .keymap
        .nav
        .insert(parse_keys("q").expect("bad lhs"), Target::Unmapped);
    let exit = script().char('q').run(&mut ws).expect("Failed to run script");
    assert_eq!(None, exit);
    assert_eq!(Some(&Modality::Navigate), ws.state.modality_stack.last());
}

#[test]
fn test_a_mapping_can_quit() {
    let mut ws = new_workspace();
    map_nav(&mut ws, "<Space>", "q");
    let exit = script().char(' ').run(&mut ws).expect("Failed to run script");
    assert_eq!(Some(ExitCode::SUCCESS), exit);
}

#[test]
fn test_numeric_prefix_repeats_the_whole_mapping() {
    let mut ws = new_workspace();
    let row = ws.book.location.row;
    map_nav(&mut ws, "<Space>", "jj");
    script().chars("3 ").run(&mut ws).expect("Failed to run script");
    assert_eq!(row + 6, ws.book.location.row);
    assert!(ws.state.numeric_prefix.is_empty());
}

#[test]
fn test_user_mappings_are_per_mode() {
    let mut ws = new_workspace();
    let row = ws.book.location.row;
    map_nav(&mut ws, "z", "jj");
    map_range(&mut ws, "x", "j");
    // In navigation mode only the navigation mapping applies.
    script().chars("zx").run(&mut ws).expect("Failed to run script");
    assert_eq!(row + 2, ws.book.location.row);
    // In range-select mode only the range mapping applies.
    script().char('v').run(&mut ws).expect("Failed to run script");
    script().chars("zx").run(&mut ws).expect("Failed to run script");
    assert_eq!(Some(&Modality::RangeSelect), ws.state.modality_stack.last());
    // `z` did nothing here, and the mapped `x` moved down one, not extended.
    assert_eq!(row + 3, ws.book.location.row);
}

#[test]
fn test_range_select_user_mapping_runs() {
    let mut ws = new_workspace();
    let row = ws.book.location.row;
    map_range(&mut ws, "z", "jj");
    script().chars("vz").run(&mut ws).expect("Failed to run script");
    assert_eq!(Some(&Modality::RangeSelect), ws.state.modality_stack.last());
    assert_eq!(row + 2, ws.book.location.row);
}

#[test]
fn test_edit_mode_does_not_see_user_mappings() {
    let mut ws = new_workspace();
    map_nav(&mut ws, "j", "kk");
    script().chars("ij").run(&mut ws).expect("Failed to run script");
    assert_eq!(Some(&Modality::CellEdit), ws.state.modality_stack.last());
    assert_eq!("j", ws.text_area.lines().join("\n"));
}

#[test]
fn test_a_mapping_can_type_into_edit_mode() {
    let mut ws = new_workspace();
    map_nav(&mut ws, "<Space>", "ihi<CR>");
    script().char(' ').run(&mut ws).expect("Failed to run script");
    assert_eq!(Some(&Modality::Navigate), ws.state.modality_stack.last());
    assert_eq!(
        "hi",
        ws.book.get_current_cell_contents().expect("no contents")
    );
}

fn last_message(ws: &Workspace) -> Option<String> {
    ws.state.message.as_ref().map(|m| m.text.clone())
}

fn type_command(ws: &mut Workspace, text: &str) {
    script()
        .char(':')
        .chars(text)
        .enter()
        .run(ws)
        .expect("Failed to run script");
}

#[test]
fn test_nnoremap_from_the_command_line() {
    let mut ws = new_workspace();
    let row = ws.book.location.row;
    type_command(&mut ws, "nnoremap <Space> jj");
    assert_eq!(Some(&Modality::Navigate), ws.state.modality_stack.last());
    assert_eq!(None, last_message(&ws));
    script().char(' ').run(&mut ws).expect("Failed to run script");
    assert_eq!(row + 2, ws.book.location.row);
}

#[test]
fn test_vnoremap_from_the_command_line() {
    let mut ws = new_workspace();
    let row = ws.book.location.row;
    type_command(&mut ws, "vnoremap z jj");
    script().chars("vz").run(&mut ws).expect("Failed to run script");
    assert_eq!(Some(&Modality::RangeSelect), ws.state.modality_stack.last());
    assert_eq!(row + 2, ws.book.location.row);
}

#[test]
fn test_nunmap_from_the_command_line_hides_a_key() {
    let mut ws = new_workspace();
    type_command(&mut ws, "nunmap q");
    let exit = script().char('q').run(&mut ws).expect("Failed to run script");
    assert_eq!(None, exit);
}

#[test]
fn test_nunmap_of_nothing_shows_a_message() {
    let mut ws = new_workspace();
    type_command(&mut ws, "nunmap z");
    assert_eq!(Some("No mapping for z".to_owned()), last_message(&ws));
    assert_eq!(Some(&Modality::Navigate), ws.state.modality_stack.last());
}

#[test]
fn test_a_bad_mapping_command_opens_a_dialog() {
    let mut ws = new_workspace();
    type_command(&mut ws, "nnoremap <Nope> j");
    assert_eq!(Some(&Modality::Dialog), ws.state.modality_stack.last());
    assert!(ws.state.keymap.nav.is_empty());
}

#[test]
fn test_a_digit_mapping_is_refused() {
    let mut ws = new_workspace();
    type_command(&mut ws, "nnoremap 5 j");
    assert_eq!(Some(&Modality::Dialog), ws.state.modality_stack.last());
    assert!(ws.state.keymap.nav.is_empty());
}

#[test]
fn test_a_mapping_can_run_a_command() {
    let mut ws = new_workspace();
    let sheets = ws.book.get_sheet_names().len();
    map_nav(&mut ws, "<Space>", "<Cmd>new-sheet Extra<CR>");
    script().char(' ').run(&mut ws).expect("Failed to run script");
    assert_eq!(sheets + 1, ws.book.get_sheet_names().len());
    assert_eq!(Some(&Modality::Navigate), ws.state.modality_stack.last());
}

#[test]
fn test_a_mapped_command_does_not_enter_command_mode_or_the_history() {
    let mut ws = new_workspace();
    map_nav(&mut ws, "<Space>", "<Cmd>new-sheet Extra<CR>");
    script().char(' ').run(&mut ws).expect("Failed to run script");
    assert_eq!(Some(&Modality::Navigate), ws.state.modality_stack.last());
    assert_eq!(None, ws.state.command_history.older(""));
}

#[test]
fn test_keys_and_commands_run_in_order() {
    let mut ws = new_workspace();
    let row = ws.book.location.row;
    map_nav(&mut ws, "<Space>", "jj<Cmd>new-sheet Extra<CR>");
    script().char(' ').run(&mut ws).expect("Failed to run script");
    assert_eq!(row + 2, ws.book.location.row);
    assert_eq!(2, ws.book.get_sheet_names().len());
}

#[test]
fn test_a_mapped_command_can_quit() {
    let mut ws = new_workspace();
    map_nav(&mut ws, "<Space>", "<Cmd>quit<CR>");
    let exit = script().char(' ').run(&mut ws).expect("Failed to run script");
    assert_eq!(Some(ExitCode::SUCCESS), exit);
}

#[test]
fn test_an_unknown_command_drops_the_rest_and_shows_the_error() {
    let mut ws = new_workspace();
    let row = ws.book.location.row;
    map_nav(&mut ws, "<Space>", "j<Cmd>bogus<CR>jj");
    script().char(' ').run(&mut ws).expect("Failed to run script");
    assert_eq!(row + 1, ws.book.location.row, "only the key before the error ran");
    assert_eq!(Some("Unrecognized command bogus".to_owned()), last_message(&ws));
    assert_eq!(Some(&Modality::Navigate), ws.state.modality_stack.last());
}

#[test]
fn test_a_command_that_does_not_parse_drops_the_rest() {
    let mut ws = new_workspace();
    let row = ws.book.location.row;
    map_nav(&mut ws, "<Space>", "<Cmd>insert-rows x<CR>jj");
    script().char(' ').run(&mut ws).expect("Failed to run script");
    assert_eq!(row, ws.book.location.row);
    assert!(last_message(&ws).is_some());
}

#[test]
fn test_a_command_that_fails_drops_the_rest_and_does_not_crash() {
    let mut ws = new_workspace();
    let row = ws.book.location.row;
    map_nav(&mut ws, "<Space>", "<Cmd>export-csv /no/such/dir/out.csv<CR>jj");
    let result = script().char(' ').run(&mut ws);
    assert!(result.is_ok(), "a failing mapping must not end the app");
    assert_eq!(row, ws.book.location.row);
    assert!(last_message(&ws).is_some());
}

#[test]
fn test_replay_can_change_a_mapping() {
    let mut ws = new_workspace();
    let row = ws.book.location.row;
    map_nav(&mut ws, "<Space>", "<Cmd>nnoremap z jj<CR>");
    script().chars(" z").run(&mut ws).expect("Failed to run script");
    assert_eq!(row + 2, ws.book.location.row);
}

fn contents_at(ws: &Workspace, row: usize) -> String {
    let col = ws.book.location.col;
    ws.book
        .get_cell_addr_contents(&Address { sheet: 0, row, col })
        .expect("no contents")
}

#[test]
fn test_a_mapping_is_one_undo_step() {
    let mut ws = new_workspace();
    map_nav(&mut ws, "<Space>", "ia<CR>jib<CR>");
    script().char(' ').run(&mut ws).expect("Failed to run script");
    assert_eq!(Some(&Modality::Navigate), ws.state.modality_stack.last());
    assert_eq!("a", contents_at(&ws, 1));
    assert_eq!("b", contents_at(&ws, 2));
    script().char('u').run(&mut ws).expect("Failed to run script");
    assert_eq!("", contents_at(&ws, 1), "one undo should undo the whole mapping");
    assert_eq!("", contents_at(&ws, 2));
}

#[test]
fn test_a_timeout_mapping_is_one_undo_step() {
    let mut ws = new_workspace();
    map_nav(&mut ws, "<Space>", "ia<CR>jib<CR>");
    map_nav(&mut ws, "<Space>w", "l");
    script().char(' ').run(&mut ws).expect("Failed to run script");
    ws.handle_key_timeout().expect("timeout failed");
    assert_eq!("a", contents_at(&ws, 1));
    assert_eq!("b", contents_at(&ws, 2));
    script().char('u').run(&mut ws).expect("Failed to run script");
    assert_eq!("", contents_at(&ws, 1));
    assert_eq!("", contents_at(&ws, 2));
}

fn popup_text(ws: &Workspace) -> String {
    ws.state
        .popup
        .as_ref()
        .expect("no dialog open")
        .get_text()
        .lines
        .iter()
        .map(|l| l.to_string())
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn test_init_keys_loads_the_config_dir_file() {
    let dir = temp_config_dir("keys-load");
    std::fs::write(dir.join("keys"), "nnoremap <Space> jj\nnunmap q\n").unwrap();
    let mut ws = new_workspace();
    ws.init_keys_from(None, Some(dir));
    assert_eq!(None, message_text(&ws));
    let row = ws.book.location.row;
    script().char(' ').run(&mut ws).expect("Failed to run script");
    assert_eq!(row + 2, ws.book.location.row);
    let exit = script().char('q').run(&mut ws).expect("Failed to run script");
    assert_eq!(None, exit, "q is unmapped");
}

#[test]
fn test_init_keys_flag_beats_the_config_dir() {
    let dir = temp_config_dir("keys-flag");
    std::fs::write(dir.join("keys"), "nnoremap a j\n").unwrap();
    let other = dir.join("other.keys");
    std::fs::write(&other, "nnoremap b k\n").unwrap();
    let mut ws = new_workspace();
    ws.init_keys_from(Some(&other), Some(dir));
    assert_eq!(1, ws.state.keymap.nav.mappings().len());
    assert_eq!(Some(other), ws.state.keymap.loaded_from);
}

#[test]
fn test_init_keys_with_no_file_changes_nothing() {
    let dir = temp_config_dir("keys-none");
    let mut ws = new_workspace();
    ws.init_keys_from(None, Some(dir));
    assert_eq!(None, message_text(&ws));
    assert!(ws.state.keymap.nav.is_empty());
    assert_eq!(None, ws.state.keymap.loaded_from);
    let mut ws = new_workspace();
    ws.init_keys_from(None, None);
    assert_eq!(None, message_text(&ws));
}

#[test]
fn test_init_keys_counts_warnings_on_the_message_line() {
    let dir = temp_config_dir("keys-warn");
    std::fs::write(dir.join("keys"), "nnoremap <Nope> j\nnnoremap a j\nwrite x\n").unwrap();
    let mut ws = new_workspace();
    ws.init_keys_from(None, Some(dir));
    assert_eq!(
        Some("2 warnings: see :config".to_owned()),
        message_text(&ws)
    );
    assert_eq!(1, ws.state.keymap.nav.mappings().len(), "the good line loaded");
    assert_eq!(2, ws.state.keymap.warnings.len());
}

#[test]
fn test_init_keys_one_warning_is_singular() {
    let dir = temp_config_dir("keys-one");
    std::fs::write(dir.join("keys"), "nnoremap 1 j\n").unwrap();
    let mut ws = new_workspace();
    ws.init_keys_from(None, Some(dir));
    assert_eq!(
        Some("1 warning: see :config".to_owned()),
        message_text(&ws)
    );
}

#[test]
fn test_a_missing_keys_flag_file_is_a_warning_not_a_crash() {
    let dir = temp_config_dir("keys-missing-flag");
    let mut ws = new_workspace();
    ws.init_keys_from(Some(&dir.join("nope")), Some(dir.clone()));
    assert_eq!(
        Some("1 warning: see :config".to_owned()),
        message_text(&ws)
    );
}

#[test]
fn test_init_config_clears_earlier_mappings() {
    let dir = temp_config_dir("keys-reset");
    let mut ws = new_workspace();
    map_nav(&mut ws, "z", "jj");
    ws.init_config_from(None, Some(dir));
    assert!(ws.state.keymap.nav.is_empty());
}

#[test]
fn test_init_config_set_lines_start_the_session_with_those_options() {
    let dir = temp_config_dir("config-set");
    std::fs::write(dir.join("config"), "set ic scs\nset searchin=raw\n").unwrap();
    let mut ws = new_workspace();
    ws.init_config_from(None, Some(dir));
    assert_eq!(None, message_text(&ws));
    assert!(ws.state.options.ignorecase());
    assert!(ws.state.options.smartcase());
    run_command(&mut ws, "set ic?");
    assert_eq!(Some("ignorecase".to_owned()), message_text(&ws));
}

#[test]
fn test_config_mapping_loses_to_a_keys_mapping_for_the_same_keys() {
    let dir = temp_config_dir("config-keys");
    std::fs::write(dir.join("config"), "nnoremap <Space> jj\nnnoremap a l\n").unwrap();
    std::fs::write(dir.join("keys"), "nnoremap <Space> jjj\n").unwrap();
    let mut ws = new_workspace();
    ws.init_config_from(None, Some(dir.clone()));
    ws.init_keys_from(None, Some(dir));
    assert_eq!(2, ws.state.keymap.nav.mappings().len());
    let row = ws.book.location.row;
    script().char(' ').run(&mut ws).expect("Failed to run script");
    assert_eq!(row + 3, ws.book.location.row);
    let col = ws.book.location.col;
    script().char('a').run(&mut ws).expect("Failed to run script");
    assert_eq!(col + 1, ws.book.location.col);
}

#[test]
fn test_config_warnings_are_counted_and_listed_by_nmap() {
    let dir = temp_config_dir("config-warn");
    std::fs::write(dir.join("config"), "write x\nset foo\nset ic\n").unwrap();
    std::fs::write(dir.join("keys"), "nnoremap 1 j\n").unwrap();
    let mut ws = new_workspace();
    ws.init_config_from(None, Some(dir.clone()));
    assert!(ws.state.options.ignorecase(), "later lines still run");
    assert_eq!(
        Some("2 warnings: see :config".to_owned()),
        message_text(&ws)
    );
    ws.init_keys_from(None, Some(dir));
    assert_eq!(
        Some("3 warnings: see :config".to_owned()),
        message_text(&ws)
    );
    run_command(&mut ws, "nmap");
    let text = popup_text(&ws);
    assert!(text.contains("config:1:"), "{text}");
    assert!(text.contains("config:2:"), "{text}");
    assert!(text.contains("keys:1:"), "{text}");
}

#[test]
fn test_config_with_theme_or_unrelated_lines_still_loads_the_theme() {
    let dir = temp_config_dir("config-theme");
    std::fs::write(dir.join("config"), "other 1\ntheme light\ntheme terminal\n").unwrap();
    let mut ws = new_workspace();
    ws.init_config_from(None, Some(dir.clone()));
    ws.init_theme_from(None, Some(dir));
    assert_eq!("terminal", ws.state.theme.name);
    assert_eq!(1, ws.state.keymap.warnings.len());
}

#[test]
fn test_nmap_lists_mappings_in_a_dialog() {
    let mut ws = new_workspace();
    map_nav(&mut ws, "<Space>c", "yj");
    run_command(&mut ws, "nmap");
    assert_eq!(&Modality::Dialog, ws.state.modality());
    let text = popup_text(&ws);
    assert!(text.contains("Navigation mappings"), "{text}");
    assert!(text.contains("<Space>c"), "{text}");
    assert!(text.contains("yj"), "{text}");
}

#[test]
fn test_vmap_lists_range_select_mappings() {
    let mut ws = new_workspace();
    map_range(&mut ws, "z", "jj");
    run_command(&mut ws, "vmap");
    let text = popup_text(&ws);
    assert!(text.contains("Range-select mappings"), "{text}");
    assert!(text.contains("jj"), "{text}");
}

#[test]
fn test_nmap_with_nothing_says_so_and_shows_load_warnings() {
    let dir = temp_config_dir("keys-listing");
    std::fs::write(dir.join("keys"), "\nnnoremap <Nope> j\n").unwrap();
    let mut ws = new_workspace();
    ws.init_keys_from(None, Some(dir.clone()));
    run_command(&mut ws, "nmap");
    let text = popup_text(&ws);
    assert!(text.contains("No mappings"), "{text}");
    assert!(text.contains("keys:2:"), "{text}");
    assert!(text.contains("not a key name"), "{text}");
    assert!(text.contains(&dir.join("keys").display().to_string()), "{text}");
}

#[test]
fn test_nmap_with_arguments_is_an_error_dialog() {
    let mut ws = new_workspace();
    run_command(&mut ws, "nmap x");
    assert_eq!(&Modality::Dialog, ws.state.modality());
    assert!(popup_text(&ws).contains("Usage"));
}

#[test]
fn test_a_mapping_can_list_mappings() {
    let mut ws = new_workspace();
    map_nav(&mut ws, "<Space>", "<Cmd>nmap<CR>");
    script().char(' ').run(&mut ws).expect("Failed to run script");
    assert_eq!(&Modality::Dialog, ws.state.modality());
}

fn help_line(ws: &Workspace, needle: &str) -> String {
    popup_text(ws)
        .lines()
        .find(|l| l.contains(needle))
        .unwrap_or_else(|| panic!("no line with {needle:?} in:\n{}", popup_text(ws)))
        .to_owned()
}

#[test]
fn test_navigation_help_lists_the_current_keys() {
    let mut ws = new_workspace();
    script().alt('h').run(&mut ws).expect("Failed to run script");
    assert_eq!(&Modality::Dialog, ws.state.modality());
    let line = help_line(&ws, "move one cell down");
    assert!(line.contains('j') && line.contains("<Down>") && line.contains("<CR>"), "{line}");
    assert!(!popup_text(&ws).contains("{{keys:"));
    assert!(!popup_text(&ws).contains("Your mappings"));
}

#[test]
fn test_navigation_help_follows_a_remap() {
    let mut ws = new_workspace();
    map_nav(&mut ws, "k", "j");
    map_nav(&mut ws, "j", "k");
    script().alt('h').run(&mut ws).expect("Failed to run script");
    let down = help_line(&ws, "move one cell down");
    assert!(down.contains("<Down>") && down.contains('k') && !down.contains('j'), "{down}");
    let up = help_line(&ws, "move one cell up");
    assert!(up.contains("<Up>") && up.contains('j') && !up.contains('k'), "{up}");
    assert!(popup_text(&ws).contains("Your mappings"));
}

#[test]
fn test_navigation_help_hides_an_unmapped_key() {
    let mut ws = new_workspace();
    type_command(&mut ws, "nunmap q");
    script().alt('h').run(&mut ws).expect("Failed to run script");
    // The prose still says "`q` quits"; the generated bullet is gone.
    assert!(!popup_text(&ws).contains(": quit"), "{}", popup_text(&ws));
    assert!(popup_text(&ws).contains("save the sheet"));
}

#[test]
fn test_range_select_help_uses_the_range_keys() {
    let mut ws = new_workspace();
    map_range(&mut ws, "z", "y");
    script().char('v').alt('h').run(&mut ws).expect("Failed to run script");
    assert_eq!(&Modality::Dialog, ws.state.modality());
    let line = help_line(&ws, "copy the range as it is displayed");
    assert!(line.contains('y') && line.contains('z'), "{line}");
    assert!(popup_text(&ws).contains("Your mappings"));
}

#[test]
fn test_help_command_expands_the_topic() {
    let mut ws = new_workspace();
    type_command(&mut ws, "nnoremap <Space> jj");
    type_command(&mut ws, "help navigate");
    assert_eq!(&Modality::Dialog, ws.state.modality());
    assert!(popup_text(&ws).contains("move one cell down"));
    assert!(popup_text(&ws).contains("<Space>"));
}

#[test]
fn test_help_keys_opens_the_key_mappings_doc() {
    let mut ws = new_workspace();
    type_command(&mut ws, "help keys");
    assert_eq!(&Modality::Dialog, ws.state.modality());
    let text = popup_text(&ws);
    assert!(text.contains("Key Mappings"), "{text}");
    assert!(text.contains("nnoremap j k"), "{text}");
}

macro_rules! assert_help_dialog {
    ($exit : expr) => {{
        let mut ws = new_workspace();
        assert_eq!(Some(&Modality::Navigate), ws.state.modality_stack.last());
        script()
            .char('i')
            .run(&mut ws)
            .expect("Failed to handle 'i' key");
        assert_eq!(Some(&Modality::CellEdit), ws.state.modality_stack.last());
        let edit_help = ws.render_help_text();
        script()
            .alt('h')
            .run(&mut ws)
            .expect("Failed to handle 'alt-h' key event");
        assert_eq!(Some(&Modality::Dialog), ws.state.modality_stack.last());
        assert_eq!(Some(edit_help), ws.state.popup);
        $exit.run(&mut ws).expect("Failed to handle key event");
        assert_eq!(Some(&Modality::CellEdit), ws.state.modality_stack.last());
    }};
}

#[test]
fn test_edit_mode_help_keycode_esc() {
    assert_help_dialog!(script().esc());
}

#[test]
fn test_edit_mode_help_keycode_enter() {
    assert_help_dialog!(script().enter());
}

#[test]
fn test_edit_mode_help_keycode_q() {
    assert_help_dialog!(script().char('q'));
}

#[test]
fn test_edit_mode_help_keycode_alt_h() {
    assert_help_dialog!(script().alt('h'));
}

#[test]
fn test_navigation_mode_help_keycode() {
    let mut ws = new_workspace();
    assert_eq!(Some(&Modality::Navigate), ws.state.modality_stack.last());
    let help_text = ws.render_help_text();
    script()
        .alt('h')
        .run(&mut ws)
        .expect("Failed to handle 'alt-h' key event");
    assert_eq!(Some(&Modality::Dialog), ws.state.modality_stack.last());
    assert_eq!(Some(help_text), ws.state.popup);
}

#[test]
fn test_command_mode_help_keycode() {
    let mut ws = new_workspace();
    assert_eq!(Some(&Modality::Navigate), ws.state.modality_stack.last());
    script()
        .char(':')
        .run(&mut ws)
        .expect("Failed to handle ':' key");
    assert_eq!(Some(&Modality::Command), ws.state.modality_stack.last());
    let edit_help = ws.render_help_text();
    script()
        .alt('h')
        .run(&mut ws)
        .expect("Failed to handle 'alt-h' key event");
    assert_eq!(Some(&Modality::Dialog), ws.state.modality_stack.last());
    assert_eq!(Some(edit_help), ws.state.popup);
}

#[test]
fn test_edit_mode_esc_keycode() {
    let mut ws = new_workspace();
    assert_eq!(Some(&Modality::Navigate), ws.state.modality_stack.last());
    script()
        .char('i')
        .run(&mut ws)
        .expect("Failed to handle 'i' key");
    assert_eq!(Some(&Modality::CellEdit), ws.state.modality_stack.last());
    script()
        .char('a')
        .esc()
        .run(&mut ws)
        .expect("Failed to handle key squence");
    assert_eq!(
        "",
        ws.book
            .get_current_cell_contents()
            .expect("Failed to get current cell contents")
    );
    assert_eq!("", ws.text_area.lines().join("\n"));
}

#[test]
fn test_navigation_numeric_prefix() {
    let mut ws = new_workspace();
    assert_eq!(Some(&Modality::Navigate), ws.state.modality_stack.last());
    ws.book
        .new_sheet(Some("Sheet2"))
        .expect("failed to create sheet2");
    ws.book
        .new_sheet(Some("Sheet3"))
        .expect("failed to create sheet3");
    script()
        .char('2')
        .char('3')
        .char('9')
        .run(&mut ws)
        .expect("Failed to run script");
    assert_eq!(239, ws.state.get_n_prefix());
}

#[test]
fn test_navigation_numeric_prefix_cancel() {
    let mut ws = new_workspace();
    assert_eq!(Some(&Modality::Navigate), ws.state.modality_stack.last());
    ws.book
        .new_sheet(Some("Sheet2"))
        .expect("failed to create sheet2");
    ws.book
        .new_sheet(Some("Sheet3"))
        .expect("failed to create sheet3");
    script()
        .char('2')
        .char('3')
        .char('9')
        .esc()
        .run(&mut ws)
        .expect("Failed to run script");
    assert_eq!(1, ws.state.get_n_prefix());
}

#[test]
fn test_navigation_tab_next_numeric_prefix() {
    let mut ws = new_workspace();
    assert_eq!(Some(&Modality::Navigate), ws.state.modality_stack.last());
    ws.book
        .new_sheet(Some("Sheet2"))
        .expect("failed to create sheet2");
    ws.book
        .new_sheet(Some("Sheet3"))
        .expect("failed to create sheet3");
    script()
        .char('2')
        .run(&mut ws)
        .expect("Failed to handle '2' key event");
    assert_eq!(2, ws.state.get_n_prefix());
    script()
        .ctrl('n')
        .run(&mut ws)
        .expect("Failed to handle 'Ctrl-n' key event");
    assert_eq!(
        "Sheet3",
        ws.book.get_sheet_name().expect("Failed to get sheet name")
    );
    assert_eq!(1, ws.state.get_n_prefix());
    script()
        .ctrl('n')
        .run(&mut ws)
        .expect("Failed to handle 'Ctrl-n' key event");
    assert_eq!(
        "Sheet1",
        ws.book.get_sheet_name().expect("Failed to get sheet name")
    );
}

#[test]
#[serial]
fn test_range_copy() {
    let mut ws = new_workspace();
    assert_eq!(Some(&Modality::Navigate), ws.state.modality_stack.last());

    let address = Address::default();
    ws.book.move_to(&address).expect("Failed to move to row");
    let original_loc = ws.book.location.clone();
    script()
        .alt('r')
        .run(&mut ws)
        .expect("Failed to handle 'Ctrl-r' key event");
    assert_eq!(Some(&Modality::RangeSelect), ws.state.modality_stack.last());
    assert_eq!(
        Some(original_loc.clone()),
        ws.state.range_select.original_location
    );
    assert!(ws.state.range_select.start.is_none());
    assert!(ws.state.range_select.end.is_none());

    script()
        .char('l')
        .char(' ')
        .run(&mut ws)
        .expect("Failed to handle key sequence");
    assert_eq!(
        Some(Address {
            sheet: 0,
            row: 1,
            col: 2
        }),
        ws.state.range_select.start
    );

    script()
        .char('j')
        .char(' ')
        .run(&mut ws)
        .expect("Failed to handle key sequence");

    assert!(ws.state.range_select.original_location.is_none());
    assert_eq!(
        Some(Address {
            sheet: 0,
            row: 1,
            col: 2
        }),
        ws.state.range_select.start
    );
    assert_eq!(
        Some(Address {
            sheet: 0,
            row: 2,
            col: 2
        }),
        ws.state.range_select.end
    );
    assert_eq!(original_loc, ws.book.location);
    assert_eq!(Some(&Modality::Navigate), ws.state.modality_stack.last());

    ws.book
        .move_to(&Address {
            sheet: 0,
            row: 5,
            col: 5,
        })
        .expect("Failed to move to row");
    let original_loc_2 = ws.book.location.clone();
    assert_eq!(
        Address {
            sheet: 0,
            row: 5,
            col: 5
        },
        original_loc_2
    );

    script()
        .char('v')
        .run(&mut ws)
        .expect("Failed to handle 'v' key event");
    assert_eq!(Some(&Modality::RangeSelect), ws.state.modality_stack.last());
    assert_eq!(
        Some(original_loc_2.clone()),
        ws.state.range_select.original_location
    );
    assert!(ws.state.range_select.start.is_some());
    assert!(ws.state.range_select.end.is_none());

    script()
        .char('h')
        .char(' ')
        .run(&mut ws)
        .expect("Failed to handle key sequence");
    assert_eq!(
        Some(Address {
            sheet: 0,
            row: 5,
            col: 5
        }),
        ws.state.range_select.start
    );

    script()
        .char('k')
        .char(' ')
        .run(&mut ws)
        .expect("Failed to handle key sequence");

    assert!(ws.state.range_select.original_location.is_none());
    assert_eq!(
        Some(Address {
            sheet: 0,
            row: 5,
            col: 5
        }),
        ws.state.range_select.start
    );
    assert_eq!(
        Some(Address {
            sheet: 0,
            row: 5,
            col: 4
        }),
        ws.state.range_select.end
    );
    assert_eq!(
        Address {
            sheet: 0,
            row: 4,
            col: 5
        },
        ws.book.location
    );
    assert_eq!(Some(&Modality::Navigate), ws.state.modality_stack.last());
}

#[test]
fn test_range_copy_mode_from_edit_mode() {
    let mut ws = new_workspace();
    assert_eq!(Some(&Modality::Navigate), ws.state.modality_stack.last());
    script()
        .char('e')
        .run(&mut ws)
        .expect("Failed to handle 'e' key event");
    assert_eq!(Some(&Modality::CellEdit), ws.state.modality_stack.last());
    script()
        .ctrl('r')
        .run(&mut ws)
        .expect("Failed to handle 'Ctrl-r' key event");
    assert_eq!(Some(&Modality::RangeSelect), ws.state.modality_stack.last());
}

#[test]
fn test_gg_movement() {
    let mut ws = new_workspace();
    assert_eq!(Some(&Modality::Navigate), ws.state.modality_stack.last());
    script()
        .char('j')
        .char('j')
        .run(&mut ws)
        .expect("failed to handle event sequence");
    assert_eq!(
        ws.book.location,
        Address {
            sheet: 0,
            row: 3,
            col: 1
        }
    );
    script()
        .char('l')
        .char('g')
        .char('g')
        .run(&mut ws)
        .expect("failed to handle event sequence");
    assert_eq!(
        ws.book.location,
        Address {
            sheet: 0,
            row: 1,
            col: 2
        }
    );
}

#[test]
fn test_h_j_k_l_movement() {
    let mut ws = new_workspace();
    assert_eq!(Some(&Modality::Navigate), ws.state.modality_stack.last());
    script()
        .char('2')
        .char('j')
        .char('l')
        .run(&mut ws)
        .expect("failed to handle event sequence");
    assert_eq!(
        ws.book.location,
        Address {
            sheet: 0,
            row: 3,
            col: 2
        }
    );
    script()
        .char('h')
        .char('2')
        .char('k')
        .run(&mut ws)
        .expect("failed to handle event sequence");
    assert_eq!(
        ws.book.location,
        Address {
            sheet: 0,
            row: 1,
            col: 1
        }
    );
}

macro_rules! assert_copy_paste {
    ($c: expr, $p: expr, $source: expr,) => {
        assert_copy_paste!($c, $p, $source, $source)
    };
    ($c: expr, $p: expr, $source: expr) => {
        assert_copy_paste!($c, $p, $source, $source)
    };
    ($c: expr, $p: expr, $source: expr, $expected: expr,) => {
        assert_copy_paste!($c, $p, $source, $expected)
    };
    ($c: expr, $p: expr, $source: expr, $expected: expr) => {{
        let mut ws = new_workspace();
        script()
            .char('j')
            .char('l')
            .run(&mut ws)
            .expect("Failed to run script");
        ws.book
            .edit_current_cell($source)
            .expect("Failed to edit cell");
        ws.book.evaluate();
        script()
            .event($c)
            .char('l')
            .char('j')
            .event($p)
            .run(&mut ws)
            .expect("Failed to run script");
        let copy = ws
            .book
            .get_current_cell_contents()
            .expect("Failed to get cell contents");
        assert_eq!(copy, $expected);
    }};
}

#[test]
#[serial]
fn test_y_p_copy_paste() {
    assert_copy_paste!(
        construct_key_event(KeyCode::Char('y')),
        construct_key_event(KeyCode::Char('p')),
        "foo",
    );
}

#[test]
#[serial]
fn test_traditional_copy_paste() {
    assert_copy_paste!(
        construct_modified_key_event(KeyCode::Char('c'), KeyModifiers::CONTROL),
        construct_modified_key_event(KeyCode::Char('v'), KeyModifiers::CONTROL),
        "foo",
    );
}

#[test]
#[serial]
fn test_y_p_copy_paste_rendered() {
    assert_copy_paste!(
        construct_key_event(KeyCode::Char('Y')),
        construct_key_event(KeyCode::Char('p')),
        "=1+2",
        "3",
    );
}

#[test]
#[serial]
fn test_traditional_copy_paste_rendered() {
    assert_copy_paste!(
        construct_modified_key_event(KeyCode::Char('C'), KeyModifiers::CONTROL),
        construct_modified_key_event(KeyCode::Char('v'), KeyModifiers::CONTROL),
        "=1+2",
        "3",
    );
}

#[test]
fn test_clear_cell() {
    let mut ws = new_workspace();
    ws.book
        .edit_current_cell("foo")
        .expect("failed to edit cell");
    ws.book.evaluate();
    assert_eq!(
        "foo",
        ws.book
            .get_current_cell_contents()
            .expect("failed to get cell contents")
    );
    script()
        .char('x')
        .run(&mut ws)
        .expect("Failed to run input script");
    assert_eq!(
        "",
        ws.book
            .get_current_cell_contents()
            .expect("failed to get cell contents")
    );
}

#[test]
fn test_clear_cell_all() {
    let mut ws = new_workspace();
    ws.book
        .edit_current_cell("foo")
        .expect("failed to edit cell");
    ws.book.evaluate();
    assert_eq!(
        "foo",
        ws.book
            .get_current_cell_contents()
            .expect("failed to get cell contents")
    );
    script()
        .char('X')
        .run(&mut ws)
        .expect("Failed to run input script");
    assert_eq!(
        "",
        ws.book
            .get_current_cell_contents()
            .expect("failed to get cell contents")
    );
}

#[test]
fn test_sheet_navigation() {
    let mut ws = new_workspace();
    ws.book
        .new_sheet(Some("sheet 2"))
        .expect("Failed to set sheet name");
    ws.book
        .new_sheet(Some("sheet 3"))
        .expect("Failed to set sheet name");
    ws.book
        .new_sheet(Some("sheet 4"))
        .expect("Failed to set sheet name");
    script()
        .ctrl('n')
        .ctrl('n')
        .run(&mut ws)
        .expect("Failed to run input script");
    assert_eq!(
        "sheet 3",
        ws.book.get_sheet_name().expect("Failed to get sheet name")
    );
    script()
        .ctrl('p')
        .run(&mut ws)
        .expect("Failed to run input script");
    assert_eq!(
        "sheet 2",
        ws.book.get_sheet_name().expect("Failed to get sheet name")
    );
}

#[test]
fn test_sheet_column_sizing() {
    let mut ws = new_workspace();
    // Measured against the starting width, so that a change to the default
    // column width upstream does not fail this test.
    let start = ws.book.get_col_size(1).expect("Failed to get column size");
    script()
        .char('3')
        .ctrl('l')
        .run(&mut ws)
        .expect("Failed to run input script");
    assert_eq!(
        start + 3,
        ws.book.get_col_size(1).expect("Failed to get column size")
    );
    script()
        .char('1')
        .ctrl('h')
        .run(&mut ws)
        .expect("Failed to run input script");
    assert_eq!(
        start + 2,
        ws.book.get_col_size(1).expect("Failed to get column size")
    );
}

#[test]
fn test_cell_replace() {
    let mut ws = new_workspace();
    ws.book
        .edit_current_cell("foo")
        .expect("Failed to edit current cell");
    assert_eq!(
        "foo",
        ws.book
            .get_current_cell_contents()
            .expect("failed to get cell contents")
            .as_str()
    );
    script()
        .char('s')
        .char('b')
        .char('a')
        .char('r')
        .enter()
        .run(&mut ws)
        .expect("Failed to run input script");
    assert_eq!(
        "bar",
        ws.book
            .get_current_cell_contents()
            .expect("failed to get cell contents")
            .as_str()
    );
}

macro_rules! assert_command_finish {
    ($script : expr) => {
        let mut ws = new_workspace();
        assert_eq!(Some(&Modality::Navigate), ws.state.modality_stack.last());
        script()
            .char(':')
            .run(&mut ws)
            .expect("Failed to handle ':' key");
        assert_eq!(Some(&Modality::Command), ws.state.modality_stack.last());
        $script.run(&mut ws).expect("Failed to handle script");
        assert_eq!(Some(&Modality::Navigate), ws.state.modality_stack.last());
    };
}

#[test]
fn test_command_mode_esc() {
    assert_command_finish!(script().esc());
}

#[test]
fn test_command_mode_enter() {
    assert_command_finish!(script().enter());
}

#[test]
fn test_edit_mode_paste() {
    let mut ws = new_workspace();
    assert_eq!(Some(&Modality::Navigate), ws.state.modality_stack.last());
    ws.state.range_select.start = Some(Address {
        sheet: 0,
        row: 1,
        col: 1,
    });
    ws.state.range_select.end = Some(Address {
        sheet: 0,
        row: 2,
        col: 2,
    });
    script()
        .char('e')
        .ctrl('p')
        .run(&mut ws)
        .expect("Failed to handle input script");
    assert_eq!(Some(&Modality::CellEdit), ws.state.modality_stack.last());
    assert_eq!(vec!["A1:B2".to_string()], ws.text_area.into_lines());
}

#[test]
fn test_range_select_esc() {
    let mut ws = new_workspace();
    assert_eq!(Some(&Modality::Navigate), ws.state.modality_stack.last());
    script()
        .char('v')
        .run(&mut ws)
        .expect("Failed to handle script");
    assert_eq!(Some(&Modality::RangeSelect), ws.state.modality_stack.last());
    script()
        .esc()
        .run(&mut ws)
        .expect("Failed to handle script");
    assert_eq!(Some(&Modality::Navigate), ws.state.modality_stack.last());
    script()
        .char('v')
        .chars("123")
        .run(&mut ws)
        .expect("Failed to handle script");
    assert_eq!(Some(&Modality::RangeSelect), ws.state.modality_stack.last());
    assert_eq!(3, ws.state.numeric_prefix.len());
    script()
        .esc()
        .run(&mut ws)
        .expect("Failed to handle script");
    assert_eq!(Some(&Modality::RangeSelect), ws.state.modality_stack.last());
    assert_eq!(0, ws.state.numeric_prefix.len());
    script()
        .esc()
        .run(&mut ws)
        .expect("Failed to handle script");
    assert_eq!(Some(&Modality::Navigate), ws.state.modality_stack.last());
}

macro_rules! assert_range_clear {
    ($script : expr) => {{
        let mut ws = new_workspace();
        assert_eq!(Some(&Modality::Navigate), ws.state.modality_stack.last());
        let first_corner = Address {
            sheet: 0,
            row: 1,
            col: 1,
        };
        let second_corner = Address {
            sheet: 0,
            row: 2,
            col: 2,
        };
        ws.book
            .update_cell(&first_corner, "foo")
            .expect("Failed to update cell");
        ws.book
            .update_cell(&second_corner, "bar")
            .expect("Failed to update cell");
        assert_eq!(
            "foo".to_string(),
            ws.book
                .get_cell_addr_contents(&first_corner)
                .expect("failed to get cell contents")
        );
        assert_eq!(
            "bar".to_string(),
            ws.book
                .get_cell_addr_contents(&second_corner)
                .expect("failed to get cell contents")
        );
        script()
            .char('v')
            .run(&mut ws)
            .expect("Failed to handle script");
        assert_eq!(Some(&Modality::RangeSelect), ws.state.modality_stack.last());
        $script.run(&mut ws).expect("Failed to handle script");
        assert_eq!(
            "".to_string(),
            ws.book
                .get_cell_addr_contents(&first_corner)
                .expect("failed to get cell contents")
        );
        assert_eq!(
            "".to_string(),
            ws.book
                .get_cell_addr_contents(&second_corner)
                .expect("failed to get cell contents")
        );
    }};
}

#[test]
fn test_range_select_clear_upper_d() {
    assert_range_clear!(script().char('j').char('l').shift_del());
}

#[test]
fn test_range_select_movement() {
    let mut ws = new_workspace();
    ws.book
        .new_sheet(Some("s2"))
        .expect("Unable create s2 sheet");
    ws.book
        .new_sheet(Some("s3"))
        .expect("Unable create s3 sheet");
    script()
        .alt('r')
        .run(&mut ws)
        .expect("failed to run script");
    assert_eq!(Some(&Modality::RangeSelect), ws.state.modality_stack.last());
    script()
        .char('3')
        .char('j')
        .char('3')
        .char('l')
        .char('1')
        .char('h')
        .char('1')
        .char('k')
        .run(&mut ws)
        .expect("failed to run script");
    assert_eq!(
        &Address {
            sheet: 0,
            row: 3,
            col: 3
        },
        &ws.book.location
    );
    script()
        .ctrl('n')
        .run(&mut ws)
        .expect("Unable to run script");
    assert_eq!(1, ws.book.location.sheet);
    script()
        .ctrl('p')
        .run(&mut ws)
        .expect("Unable to run script");
    assert_eq!(0, ws.book.location.sheet);
}

#[test]
fn test_range_select_clear_lower_d() {
    assert_range_clear!(script().char('j').char('l').del());
}

macro_rules! assert_range_copy {
    ($script: expr) => {{
        let mut ws = new_workspace();
        let top_left_addr = Address {
            sheet: 0,
            row: 2,
            col: 2,
        };
        let bot_right_addr = Address {
            sheet: 0,
            row: 4,
            col: 4,
        };
        ws.book
            .update_cell(&top_left_addr, "top_left")
            .expect("Failed to update top left");
        ws.book
            .update_cell(&bot_right_addr, "bot_right")
            .expect("Failed to update top left");
        assert!(ws.state.clipboard.is_none());
        script()
            .alt('r')
            .char('j')
            .char('l')
            .char(' ')
            .run(&mut ws)
            .expect("failed to run script");
        assert_eq!(
            &top_left_addr,
            ws.state
                .range_select
                .start
                .as_ref()
                .expect("Didn't find a start of range")
        );
        script()
            .char('2')
            .char('j')
            .char('2')
            .char('l')
            .run(&mut ws)
            .expect("failed to run script");
        assert_eq!(
            &bot_right_addr,
            ws.state
                .range_select
                .end
                .as_ref()
                .expect("Didn't find a start of range")
        );
        assert_eq!(
            &Address {
                sheet: 0,
                row: 1,
                col: 1
            },
            ws.state
                .range_select
                .original_location
                .as_ref()
                .expect("Expected an original location")
        );
        assert_eq!(
            Some(&Modality::RangeSelect),
            ws.state.modality_stack.iter().last()
        );
        $script.run(&mut ws).expect("failed to run script");
        assert!(ws.state.clipboard.is_some());
        match ws.state.clipboard.unwrap() {
            crate::ui::ClipboardContents::Cell(_) => assert!(false, "Not rows in Clipboard"),
            crate::ui::ClipboardContents::Range(rows) => {
                assert_eq!(
                    vec![
                        vec!["top_left".to_string(), "".to_string(), "".to_string()],
                        vec!["".to_string(), "".to_string(), "".to_string()],
                        vec!["".to_string(), "".to_string(), "bot_right".to_string()],
                    ],
                    rows
                );
            }
        }
        assert_eq!(
            Some(&Modality::Navigate),
            ws.state.modality_stack.iter().last()
        );
    }};
}

#[test]
#[serial]
fn test_range_select_copy_c() {
    assert_range_copy!(script().ctrl('c'));
}

#[test]
#[serial]
fn test_range_select_copy_y() {
    assert_range_copy!(script().char('y'));
}

#[test]
#[serial]
fn test_range_select_copy_capital_y() {
    assert_range_copy!(script().char('Y'));
}

#[test]
#[serial]
fn test_range_select_copy_capital_c() {
    assert_range_copy!(script().ctrl('C'));
}

#[test]
fn test_extend_to_range() {
    let mut ws = new_workspace();
    ws.book
        .edit_current_cell("=B1+1")
        .expect("Failed to edit cell");
    ws.book.evaluate();
    script()
        .char('v')
        .char('j')
        .char('x')
        .run(&mut ws)
        .expect("Unable to run script");
    let extended_cell = ws
        .book
        .get_cell_addr_contents(&Address {
            sheet: 0,
            row: 2,
            col: 1,
        })
        .expect("Failed to get cell contents");
    assert_eq!("=B2+1".to_string(), extended_cell);
}

#[test]
fn test_color_cells() {
    let mut ws = new_workspace();
    script()
        .char('v')
        .chars("jjll")
        .char(':')
        .chars("color-cell red")
        .enter()
        .run(&mut ws)
        .expect("Unable to run script");
    for ri in 1..=3 {
        for ci in 1..=3 {
            let style = ws
                .book
                .get_cell_style(&Address {
                    sheet: ws.book.location.sheet,
                    row: ri,
                    col: ci,
                })
                .expect("failed to get style");
            assert_eq!(
                Color::Rgb("#800000".to_string()),
                style.fill.color,
                "No background color set for {}:{}",
                ri,
                ci
            );
        }
    }
}

#[test]
fn test_color_row() {
    let mut ws = new_workspace();
    script()
        .char(':')
        .chars("color-rows red")
        .enter()
        .run(&mut ws)
        .expect("Unable to run script");
    for ci in [1, book::LAST_COLUMN] {
        let style = ws
            .book
            .get_cell_style(&Address {
                sheet: ws.book.location.sheet,
                row: 1,
                col: ci as usize,
            })
            .expect("failed to get style");
        assert_eq!(
            Color::Rgb("#800000".to_string()),
            style.fill.color,
            "No background color set for {}:{}",
            1,
            ci
        );
    }
}

#[test]
fn test_color_col() {
    let mut ws = new_workspace();
    script()
        .char(':')
        .chars("color-columns red")
        .enter()
        .run(&mut ws)
        .expect("Unable to run script");
    for ri in [1, book::LAST_ROW] {
        let style = ws
            .book
            .get_cell_style(&Address {
                sheet: ws.book.location.sheet,
                row: ri as usize,
                col: 1,
            })
            .expect("failed to get style");
        assert_eq!(
            Color::Rgb("#800000".to_string()),
            style.fill.color,
            "No background color set for {}:{}",
            ri,
            1
        );
    }
}

#[test]
fn test_bold_text() {
    let mut ws = new_workspace();
    let before_style = ws
        .book
        .get_cell_style(&Address {
            sheet: 0,
            row: 1,
            col: 1,
        })
        .expect("Failed to get style");
    assert!(!before_style.font.b);
    script()
        .char('B')
        .run(&mut ws)
        .expect("Unable to run script");
    let style = ws
        .book
        .get_cell_style(&Address {
            sheet: 0,
            row: 1,
            col: 1,
        })
        .expect("Failed to get style");
    assert!(style.font.b);
    script()
        .char('B')
        .run(&mut ws)
        .expect("Unable to run script");
    assert!(!before_style.font.b);
}

#[test]
fn test_italic_text() {
    let mut ws = new_workspace();
    let before_style = ws
        .book
        .get_cell_style(&Address {
            sheet: 0,
            row: 1,
            col: 1,
        })
        .expect("Failed to get style");
    assert!(!before_style.font.i);
    script()
        .char('I')
        .run(&mut ws)
        .expect("Unable to run script");
    let style = ws
        .book
        .get_cell_style(&Address {
            sheet: 0,
            row: 1,
            col: 1,
        })
        .expect("Failed to get style");
    assert!(style.font.i);
    script()
        .char('I')
        .run(&mut ws)
        .expect("Unable to run script");
    assert!(!before_style.font.i);
}

//#[test]
//fn test_quit() {
//    let mut ws =
//        Workspace::new_empty("en", "America/New_York").expect("Failed to get empty workbook");
//    let result = script()
//        .char('q')
//        .run(&mut ws)
//        .expect("Failed to run input script");
//    assert!(result.is_some());
//}

#[test]
fn test_quit_dialog() {
    let mut ws = new_workspace();
    assert!(!ws.book.dirty);
    let mut result = script()
        .char('q')
        .run(&mut ws)
        .expect("Failed to run input script");
    assert!(result.is_some());

    script()
        .chars("efoo")
        .enter()
        .run(&mut ws)
        .expect("Failed to modify book");
    assert!(ws.book.dirty);
    result = script()
        .char('q')
        .run(&mut ws)
        .expect("Failed to run input script");
    assert!(result.is_none());
    assert_eq!(ws.state.modality(), &Modality::Quit);

    script()
        .char('n')
        .run(&mut ws)
        .expect("Failed to run input script");
    assert_eq!(ws.state.modality(), &Modality::default());
    assert!(ws.book.dirty);
    script()
        .char('q')
        .esc()
        .run(&mut ws)
        .expect("Failed to run input script");
    assert_eq!(ws.state.modality(), &Modality::default());
    assert!(ws.book.dirty);
    // TODO(zaphar): The below will write to disk. so commenting it out for now.
    //script()
    //    .char('q')
    //    .char('y')
    //    .run(&mut ws)
    //    .expect("Failed to run input script");
    //assert!(!ws.book.dirty);
}

#[test]
fn test_moving_the_cursor_does_not_change_the_book() {
    let mut ws = new_workspace();
    let result = script()
        .chars("jjl")
        .char('q')
        .run(&mut ws)
        .expect("Failed to run input script");
    assert!(!ws.book.dirty);
    assert!(result.is_some(), "q quits with no prompt");
}

// parse_csv_rows unit tests

#[test]
fn test_parse_csv_rows_preserves_first_row() {
    let csv_text = "A1,B1\nA2,B2\n";
    let rows = parse_csv_rows(csv_text).expect("Failed to parse csv");
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0], vec!["A1", "B1"]);
    assert_eq!(rows[1], vec!["A2", "B2"]);
}

#[test]
fn test_parse_csv_rows_single_cell() {
    let csv_text = "hello\n";
    let rows = parse_csv_rows(csv_text).expect("Failed to parse csv");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0], vec!["hello"]);
}

#[test]
fn test_parse_csv_rows_empty() {
    let csv_text = "";
    let rows = parse_csv_rows(csv_text).expect("Failed to parse csv");
    assert_eq!(rows.len(), 0);
}

// System clipboard integration tests

#[test]
#[serial]
fn test_single_cell_copy_writes_system_clipboard() {
    use arboard::Clipboard;
    let mut ws = new_workspace();
    ws.book.edit_current_cell("hello world").unwrap();
    ws.book.evaluate();
    script().char('y').run(&mut ws).expect("Failed to run");
    let mut cb = Clipboard::new().expect("Failed to open clipboard");
    assert_eq!(cb.get_text().unwrap(), "hello world");
}

#[test]
#[serial]
fn test_single_cell_copy_rendered_writes_system_clipboard() {
    use arboard::Clipboard;
    let mut ws = new_workspace();
    ws.book.edit_current_cell("=1+2").unwrap();
    ws.book.evaluate();
    script().char('Y').run(&mut ws).expect("Failed to run");
    let mut cb = Clipboard::new().expect("Failed to open clipboard");
    assert_eq!(cb.get_text().unwrap(), "3");
}

#[test]
#[serial]
fn test_range_copy_writes_system_clipboard_csv() {
    use arboard::Clipboard;
    let mut ws = new_workspace();
    ws.book.edit_current_cell("hello").unwrap();
    script().char('l').run(&mut ws).unwrap();
    ws.book.edit_current_cell("world").unwrap();
    ws.book.evaluate();
    script()
        .char('h')
        .alt('r')
        .char(' ')
        .char('l')
        .char('y')
        .run(&mut ws)
        .unwrap();
    let mut cb = Clipboard::new().expect("Failed to open clipboard");
    let text = cb.get_text().unwrap();
    assert!(text.contains("hello"), "clipboard text was: {:?}", text);
    assert!(text.contains("world"), "clipboard text was: {:?}", text);
}

#[test]
#[serial]
fn test_paste_from_system_clipboard() {
    use arboard::Clipboard;
    let mut ws = new_workspace();
    let mut cb = Clipboard::new().expect("Failed to open clipboard");
    cb.set_text("A1,B1\nA2,B2\n").unwrap();
    drop(cb);
    assert!(ws.state.clipboard.is_none());
    script().char('p').run(&mut ws).expect("Failed to run");
    assert_eq!(ws.book.get_current_cell_contents().unwrap(), "A1");
    script().char('l').run(&mut ws).unwrap();
    assert_eq!(ws.book.get_current_cell_contents().unwrap(), "B1");
    script().char('j').char('h').run(&mut ws).unwrap();
    assert_eq!(ws.book.get_current_cell_contents().unwrap(), "A2");
    script().char('l').run(&mut ws).unwrap();
    assert_eq!(ws.book.get_current_cell_contents().unwrap(), "B2");
}

#[test]
#[serial]
fn test_system_paste_command() {
    use arboard::Clipboard;
    let mut ws = new_workspace();
    let mut cb = Clipboard::new().expect("Failed to open clipboard");
    cb.set_text("X1,Y1\nX2,Y2\n").unwrap();
    drop(cb);
    script()
        .char(':')
        .chars("system-paste")
        .enter()
        .run(&mut ws)
        .expect("Failed to run");
    assert_eq!(ws.book.get_current_cell_contents().unwrap(), "X1");
}

#[test]
fn test_paste_from_internal_range_clipboard() {
    let mut ws = new_workspace();
    ws.state.clipboard = Some(ClipboardContents::Range(vec![
        vec!["A1".to_string(), "B1".to_string()],
        vec!["A2".to_string(), "B2".to_string()],
    ]));
    script().char('p').run(&mut ws).expect("Failed to run");
    assert_eq!(ws.book.get_current_cell_contents().unwrap(), "A1");
    script().char('l').run(&mut ws).unwrap();
    assert_eq!(ws.book.get_current_cell_contents().unwrap(), "B1");
}

fn new_workspace<'a>() -> Workspace<'a> {
    Workspace::new_empty("en", "America/New_York").expect("Failed to get empty workbook")
}

// ---------------------------------------------------------------------------
// Phase 3: UI/Command Integration (REQ-005..REQ-009)
// ---------------------------------------------------------------------------

fn ui_tmp_path(name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("sheetui_ui_p3_{name}"))
}

#[test]
fn test_new_workspace_default_format_is_sui() {
    // REQ-008: new empty workspace defaults to SUI format.
    let ws = new_workspace();
    assert_eq!(
        ws.book.get_format(),
        &book::FileFormat::Sui,
        "new workspace book must default to FileFormat::Sui"
    );
}

#[test]
fn test_new_workspace_name_ends_with_sui() {
    // REQ-008: new empty workspace name should default to .sui extension.
    let ws = new_workspace();
    assert!(
        ws.name.to_string_lossy().ends_with(".sui"),
        "new workspace path must end with .sui, got: {}",
        ws.name.display()
    );
}

#[test]
fn test_write_cmd_saves_as_sui_and_updates_format() {
    // REQ-007: :w path.sui saves as SUI and updates the stored format.
    let path = ui_tmp_path("write_sui.sui");
    let mut ws = new_workspace();
    ws.book
        .update_cell(&Address { sheet: 0, row: 1, col: 1 }, "write-sui-test")
        .expect("set cell");
    ws.book.evaluate();
    script()
        .char(':')
        .chars(&format!("w {}", path.display()))
        .enter()
        .run(&mut ws)
        .expect("write command failed");
    assert_eq!(
        ws.book.get_format(),
        &book::FileFormat::Sui,
        "book format must be Sui after :w .sui"
    );
    let text = std::fs::read_to_string(&path).expect("read saved file");
    assert!(
        text.contains("[sheet"),
        "saved .sui file must contain a sheet block, got: {text}"
    );
    std::fs::remove_file(&path).ok();
}

#[test]
fn test_write_cmd_saves_as_xlsx_and_updates_format() {
    // REQ-007: :w path.xlsx saves as xlsx and updates the stored format.
    let path = ui_tmp_path("write_xlsx.xlsx");
    let mut ws = new_workspace();
    ws.book
        .update_cell(&Address { sheet: 0, row: 1, col: 1 }, "xlsx-write-test")
        .expect("set cell");
    ws.book.evaluate();
    script()
        .char(':')
        .chars(&format!("w {}", path.display()))
        .enter()
        .run(&mut ws)
        .expect("write command failed");
    assert_eq!(
        ws.book.get_format(),
        &book::FileFormat::Xlsx,
        "book format must be Xlsx after :w .xlsx"
    );
    let bytes = std::fs::read(&path).expect("read saved xlsx");
    assert!(
        bytes.starts_with(b"PK"),
        "saved .xlsx must be a valid zip/xlsx (starts with PK magic bytes)"
    );
    std::fs::remove_file(&path).ok();
}

#[test]
fn test_edit_cmd_loads_sui_file() {
    // REQ-005: :e path.sui loads SUI content using the SUI parser.
    let path = ui_tmp_path("edit_sui.sui");
    let content = "[sheet \"Sheet1\"]\nA1 = \"loaded-via-edit\"\n[/sheet]\n";
    std::fs::write(&path, content).expect("write temp .sui");
    let mut ws = new_workspace();
    script()
        .char(':')
        .chars(&format!("e {}", path.display()))
        .enter()
        .run(&mut ws)
        .expect("edit command failed");
    assert_eq!(
        ws.book.get_format(),
        &book::FileFormat::Sui,
        "book format must be Sui after :e .sui"
    );
    let val = ws
        .book
        .get_cell_addr_contents(&Address { sheet: 0, row: 1, col: 1 })
        .expect("get A1 after :e");
    assert_eq!(val, "loaded-via-edit", "cell A1 must be loaded from .sui file");
    std::fs::remove_file(&path).ok();
}

#[test]
fn test_write_then_edit_round_trip_sui() {
    // REQ-006 + REQ-010: :w saves, :e reloads, cell values survive.
    let path = ui_tmp_path("round_trip.sui");
    let mut ws = new_workspace();
    ws.book
        .update_cell(&Address { sheet: 0, row: 1, col: 1 }, "round-trip-val")
        .expect("set cell");
    ws.book.evaluate();
    script()
        .char(':')
        .chars(&format!("w {}", path.display()))
        .enter()
        .run(&mut ws)
        .expect(":w command");
    ws.book
        .update_cell(&Address { sheet: 0, row: 1, col: 1 }, "")
        .expect("clear cell locally");
    script()
        .char(':')
        .chars(&format!("e {}", path.display()))
        .enter()
        .run(&mut ws)
        .expect(":e command");
    let val = ws
        .book
        .get_cell_addr_contents(&Address { sheet: 0, row: 1, col: 1 })
        .expect("get A1 after :e");
    assert_eq!(val, "round-trip-val", "cell must survive :w/:e round-trip");
    std::fs::remove_file(&path).ok();
}

#[test]
fn test_parse_warnings_stored_after_loading_malformed_sui() {
    // REQ-009: loading a malformed .sui file stores warnings on the Book.
    let path = ui_tmp_path("malformed.sui");
    let content = "[sheet \"Sheet1\"]\nTHIS IS NOT VALID\nA1 = \"valid-cell\"\n[/sheet]\n";
    std::fs::write(&path, content).expect("write malformed .sui");
    let mut ws = new_workspace();
    script()
        .char(':')
        .chars(&format!("e {}", path.display()))
        .enter()
        .run(&mut ws)
        .expect("edit command");
    assert!(
        !ws.book.parse_warnings.is_empty(),
        "loading a malformed .sui file must store parse warnings on Book"
    );
    let val = ws
        .book
        .get_cell_addr_contents(&Address { sheet: 0, row: 1, col: 1 })
        .expect("get A1");
    assert_eq!(
        val, "valid-cell",
        "valid cell must be loaded despite malformed lines"
    );
    std::fs::remove_file(&path).ok();
}

// ---------------------------------------------------------------------------
// Sort command
// ---------------------------------------------------------------------------

fn sort_key(col: Option<usize>, ascending: bool) -> CmdSortKey {
    CmdSortKey { col, ascending }
}

fn parse_sort(input: &str) -> (usize, Vec<CmdSortKey>) {
    match parse(input) {
        Ok(Some(Cmd::Sort { skip, keys })) => (skip, keys),
        other => panic!("{input:?} did not parse as sort: {other:?}"),
    }
}

#[test]
fn test_cmd_sort_defaults_to_cursor_column_descending() {
    assert_eq!(parse_sort("sort"), (0, vec![sort_key(None, false)]));
    assert_eq!(parse_sort("sort asc"), (0, vec![sort_key(None, true)]));
    assert_eq!(parse_sort("sort desc"), (0, vec![sort_key(None, false)]));
}

#[test]
fn test_cmd_sort_columns_and_directions() {
    assert_eq!(parse_sort("sort B"), (0, vec![sort_key(Some(2), false)]));
    assert_eq!(
        parse_sort("sort B C"),
        (0, vec![sort_key(Some(2), false), sort_key(Some(3), false)])
    );
    assert_eq!(
        parse_sort("sort b ASC c"),
        (0, vec![sort_key(Some(2), true), sort_key(Some(3), false)])
    );
    assert_eq!(
        parse_sort("sort B C asc"),
        (0, vec![sort_key(Some(2), false), sort_key(Some(3), true)])
    );
    assert_eq!(parse_sort("sort AA"), (0, vec![sort_key(Some(27), false)]));
}

#[test]
fn test_cmd_sort_skip() {
    assert_eq!(
        parse_sort("sort skip 2 B asc"),
        (2, vec![sort_key(Some(2), true)])
    );
    assert_eq!(parse_sort("sort SKIP 3 asc"), (3, vec![sort_key(None, true)]));
    assert_eq!(parse_sort("sort skip 0"), (0, vec![sort_key(None, false)]));
}

#[test]
fn test_cmd_sort_rejects_bad_input() {
    for input in [
        "sort skip",
        "sort skip x",
        "sort B skip 1",
        "sort B desc desc",
        "sort desc B",
        "sort sideways",
        "sort ABCD",
        "sort 1",
        "sorting",
    ] {
        assert!(parse(input).is_err(), "{input:?} should be an error");
    }
}

/// A workspace whose first sheet holds `rows` from A1.
fn sort_workspace<'a>(rows: &[&[&str]]) -> Workspace<'a> {
    let mut ws = new_workspace();
    for (ri, row) in rows.iter().enumerate() {
        for (ci, value) in row.iter().enumerate() {
            ws.book
                .update_cell(&Address { sheet: 0, row: ri + 1, col: ci + 1 }, *value)
                .expect("set cell");
        }
    }
    ws.book.evaluate();
    ws
}

fn sheet_column(ws: &Workspace, col: usize, rows: std::ops::RangeInclusive<usize>) -> Vec<String> {
    rows.map(|row| {
        ws.book
            .get_cell_addr_rendered(&Address { sheet: 0, row, col })
            .expect("render cell")
    })
    .collect()
}

fn run_command(ws: &mut Workspace, cmd: &str) {
    script()
        .char(':')
        .chars(cmd)
        .enter()
        .run(ws)
        .expect("run command");
}

#[test]
fn test_sort_command_sorts_used_area_by_cursor_column() {
    let mut ws = sort_workspace(&[&["1", "a"], &["3", "b"], &["2", "c"]]);
    run_command(&mut ws, "sort");
    assert_eq!(sheet_column(&ws, 1, 1..=3), vec!["3", "2", "1"]);
    assert_eq!(sheet_column(&ws, 2, 1..=3), vec!["b", "c", "a"]);
    assert_eq!(ws.book.location, Address { sheet: 0, row: 1, col: 1 });
}

#[test]
fn test_sort_command_skip_and_keys() {
    let mut ws = sort_workspace(&[
        &["name", "group"],
        &["c", "1"],
        &["a", "2"],
        &["b", "1"],
    ]);
    run_command(&mut ws, "sort skip 1 B asc A");
    assert_eq!(sheet_column(&ws, 1, 1..=4), vec!["name", "c", "b", "a"]);
}

#[test]
fn test_sort_command_uses_range_selection() {
    let mut ws = sort_workspace(&[&["2"], &["1"], &["0"]]);
    script()
        .char('v')
        .char('j')
        .char(':')
        .chars("sort asc")
        .enter()
        .run(&mut ws)
        .expect("run script");
    assert_eq!(sheet_column(&ws, 1, 1..=3), vec!["1", "2", "0"]);
}

#[test]
fn test_sort_command_all_rows_skipped_is_a_dialog() {
    let mut ws = sort_workspace(&[&["2"], &["1"]]);
    run_command(&mut ws, "sort skip 2");
    assert_eq!(Some(&Modality::Dialog), ws.state.modality_stack.last());
    assert_eq!(
        Some(Markdown::from_str("All 2 rows are skipped; nothing to sort.")),
        ws.state.popup
    );
    assert_eq!(sheet_column(&ws, 1, 1..=2), vec!["2", "1"]);
}

#[test]
fn test_sort_command_key_outside_selection_is_a_dialog() {
    let mut ws = sort_workspace(&[&["2", "x"], &["1", "y"]]);
    script()
        .char('v')
        .char('j')
        .char(':')
        .chars("sort B")
        .enter()
        .run(&mut ws)
        .expect("run script");
    assert_eq!(
        Some(Markdown::from_str("Key column B is outside the selected range.")),
        ws.state.popup
    );
    assert_eq!(sheet_column(&ws, 1, 1..=2), vec!["2", "1"]);
}

#[test]
fn test_sort_command_book_error_is_a_dialog() {
    let mut ws = sort_workspace(&[&["2", "x"], &["1", "y"]]);
    run_command(&mut ws, "sort B A B");
    assert_eq!(
        Some(Markdown::from_str("Column B is given more than one time.")),
        ws.state.popup
    );
}

#[test]
fn test_range_select_s_sorts_descending_by_cursor_column() {
    let mut ws = sort_workspace(&[&["1", "x"], &["2", "y"], &["0", "z"]]);
    script()
        .char('v')
        .char('j')
        .char('s')
        .run(&mut ws)
        .expect("run script");
    assert_eq!(sheet_column(&ws, 1, 1..=3), vec!["2", "1", "0"]);
    assert_eq!(sheet_column(&ws, 2, 1..=3), vec!["y", "x", "z"]);
    assert_eq!(Some(&Modality::Navigate), ws.state.modality_stack.last());
    assert_eq!(ws.book.location, Address { sheet: 0, row: 1, col: 1 });
}

#[test]
fn test_range_select_shift_s_sorts_ascending_by_cursor_column() {
    let mut ws = sort_workspace(&[&["1", "b"], &["2", "a"], &["0", "c"]]);
    script()
        .char('v')
        .char('j')
        .char('l')
        .char('S')
        .run(&mut ws)
        .expect("run script");
    assert_eq!(sheet_column(&ws, 2, 1..=3), vec!["a", "b", "c"]);
    assert_eq!(sheet_column(&ws, 1, 1..=3), vec!["2", "1", "0"]);
    assert_eq!(Some(&Modality::Navigate), ws.state.modality_stack.last());
}

#[test]
fn test_range_select_s_without_range_start_does_nothing() {
    let mut ws = sort_workspace(&[&["1"], &["2"]]);
    script()
        .alt('r')
        .char('j')
        .char('s')
        .run(&mut ws)
        .expect("run script");
    assert_eq!(sheet_column(&ws, 1, 1..=2), vec!["1", "2"]);
    assert_eq!(Some(&Modality::RangeSelect), ws.state.modality_stack.last());
}

#[test]
fn test_range_select_s_from_cell_edit_does_nothing() {
    let mut ws = sort_workspace(&[&["1"], &["2"]]);
    script()
        .char('i')
        .ctrl('r')
        .char(' ')
        .char('j')
        .char('s')
        .run(&mut ws)
        .expect("run script");
    assert_eq!(sheet_column(&ws, 1, 1..=2), vec!["1", "2"]);
    assert_eq!(Some(&Modality::RangeSelect), ws.state.modality_stack.last());
}

#[test]
fn test_command_mode_esc_does_not_run_the_command() {
    let mut ws = new_workspace();
    script()
        .char(':')
        .chars("new-sheet cancelled")
        .esc()
        .run(&mut ws)
        .expect("run script");
    assert_eq!(Some(&Modality::Navigate), ws.state.modality_stack.last());
    assert_eq!(ws.book.get_sheet_names().len(), 1);
}

impl InputScript {
    fn up(self) -> Self {
        self.event(construct_key_event(KeyCode::Up))
    }

    fn down(self) -> Self {
        self.event(construct_key_event(KeyCode::Down))
    }
}

fn command_text(ws: &Workspace) -> String {
    use tui_prompts::State;
    ws.state.command_state.value().to_owned()
}

#[test]
fn test_command_history_up_and_down() {
    let mut ws = new_workspace();
    run_command(&mut ws, "new-sheet a");
    run_command(&mut ws, "new-sheet b");
    script().char(':').chars("x").up().run(&mut ws).expect("run");
    // "x" is the prefix, and no command starts with it.
    assert_eq!(command_text(&ws), "x");
    script().esc().char(':').up().run(&mut ws).expect("run");
    assert_eq!(command_text(&ws), "new-sheet b");
    script().up().run(&mut ws).expect("run");
    assert_eq!(command_text(&ws), "new-sheet a");
    script().up().run(&mut ws).expect("run");
    assert_eq!(command_text(&ws), "new-sheet a");
    script().down().run(&mut ws).expect("run");
    assert_eq!(command_text(&ws), "new-sheet b");
    script().down().run(&mut ws).expect("run");
    assert_eq!(command_text(&ws), "");
}

#[test]
fn test_command_history_prefix_match() {
    let mut ws = new_workspace();
    run_command(&mut ws, "new-sheet a");
    run_command(&mut ws, "select-sheet a");
    script().char(':').chars("new").up().run(&mut ws).expect("run");
    assert_eq!(command_text(&ws), "new-sheet a");
    script().down().run(&mut ws).expect("run");
    assert_eq!(command_text(&ws), "new");
}

#[test]
fn test_command_history_runs_a_recalled_command() {
    let mut ws = new_workspace();
    run_command(&mut ws, "new-sheet");
    script().char(':').up().enter().run(&mut ws).expect("run");
    assert_eq!(ws.book.get_sheet_names().len(), 3);
}

#[test]
fn test_command_history_does_not_record_a_cancelled_command() {
    let mut ws = new_workspace();
    script()
        .char(':')
        .chars("new-sheet x")
        .esc()
        .char(':')
        .up()
        .run(&mut ws)
        .expect("run");
    assert_eq!(command_text(&ws), "");
}

// ---------------------------------------------------------------------------
// Undo and redo
// ---------------------------------------------------------------------------

fn cell(ws: &Workspace, row: usize, col: usize) -> String {
    ws.book
        .get_cell_addr_contents(&Address { sheet: 0, row, col })
        .expect("cell contents")
}

fn run(ws: &mut Workspace, keys: InputScript) {
    keys.run(ws).expect("Failed to run input script");
}

#[test]
fn test_u_undoes_and_r_redoes_a_cell_edit() {
    let mut ws = new_workspace();
    run(&mut ws, script().chars("efoo").enter().chars("jj"));
    assert_eq!(cell(&ws, 1, 1), "foo");
    run(&mut ws, script().char('u'));
    assert_eq!(cell(&ws, 1, 1), "");
    assert_eq!(ws.book.location, Address { sheet: 0, row: 1, col: 1 });
    assert_eq!(ws.book.get_current_cell_contents().unwrap(), "");
    run(&mut ws, script().chars("jr"));
    assert_eq!(cell(&ws, 1, 1), "foo");
    assert_eq!(ws.book.location, Address { sheet: 0, row: 1, col: 1 });
}

#[test]
fn test_a_range_paste_undoes_in_one_step() {
    let mut ws = new_workspace();
    run(&mut ws, script().chars("ex").enter());
    ws.state.clipboard = Some(ClipboardContents::Range(vec![
        vec!["a".to_string(), "b".to_string()],
        vec!["c".to_string(), "d".to_string()],
    ]));
    run(&mut ws, script().char('p'));
    assert_eq!(cell(&ws, 2, 2), "d");
    run(&mut ws, script().char('u'));
    assert_eq!(
        [cell(&ws, 1, 1), cell(&ws, 1, 2), cell(&ws, 2, 1), cell(&ws, 2, 2)],
        ["x", "", "", ""]
    );
}

#[test]
fn test_a_count_undoes_in_one_step() {
    let mut ws = new_workspace();
    let width = ws.book.get_col_size(1).unwrap();
    run(&mut ws, script().char('5').ctrl('l'));
    assert_eq!(ws.book.get_col_size(1).unwrap(), width + 5);
    run(&mut ws, script().char('u'));
    assert_eq!(ws.book.get_col_size(1).unwrap(), width);
    assert!(!ws.book.undo().unwrap(), "5 Ctrl-l was one action");
}

#[test]
fn test_u_with_a_count_undoes_that_many_actions() {
    let mut ws = new_workspace();
    run(
        &mut ws,
        script()
            .chars("ea")
            .enter()
            .chars("jeb")
            .enter()
            .chars("jec")
            .enter(),
    );
    run(&mut ws, script().chars("2u"));
    assert_eq!([cell(&ws, 1, 1), cell(&ws, 2, 1), cell(&ws, 3, 1)], ["a", "", ""]);
    run(&mut ws, script().chars("5r"));
    assert_eq!([cell(&ws, 1, 1), cell(&ws, 2, 1), cell(&ws, 3, 1)], ["a", "b", "c"]);
}

#[test]
fn test_o_and_the_edit_after_it_are_two_actions() {
    let mut ws = new_workspace();
    run(&mut ws, script().chars("ea").enter().chars("jex").enter().char('k'));
    run(&mut ws, script().chars("ob").enter());
    let column = |ws: &Workspace| [cell(ws, 1, 1), cell(ws, 2, 1), cell(ws, 3, 1)];
    assert_eq!(column(&ws), ["a", "b", "x"]);
    run(&mut ws, script().char('u'));
    assert_eq!(column(&ws), ["a", "", "x"], "the new row is still there");
    run(&mut ws, script().char('u'));
    assert_eq!(column(&ws), ["a", "x", ""]);
}

#[test]
fn test_undo_and_redo_commands() {
    let mut ws = new_workspace();
    run(&mut ws, script().chars("efoo").enter());
    run(&mut ws, script().chars(":undo").enter());
    assert_eq!(cell(&ws, 1, 1), "");
    run(&mut ws, script().chars(":redo").enter());
    assert_eq!(cell(&ws, 1, 1), "foo");
}

#[test]
fn test_parse_undo_and_redo() {
    assert_eq!(parse("undo"), Ok(Some(Cmd::Undo)));
    assert_eq!(parse("redo"), Ok(Some(Cmd::Redo)));
    assert!(parse("undo 2").is_err());
    assert!(parse("redo 2").is_err());
}

#[test]
fn test_u_and_r_in_cell_edit_are_text() {
    let mut ws = new_workspace();
    run(&mut ws, script().chars("eur").enter());
    assert_eq!(cell(&ws, 1, 1), "ur");
}

#[test]
fn test_undo_back_to_the_start_quits_with_no_prompt() {
    let mut ws = new_workspace();
    run(&mut ws, script().chars("efoo").enter());
    assert!(ws.book.dirty);
    run(&mut ws, script().char('u'));
    assert!(!ws.book.dirty);
    let result = script().char('q').run(&mut ws).expect("Failed to run");
    assert!(result.is_some(), "q quits with no prompt");
}

#[test]
fn test_enter_that_ends_an_edit_is_not_typed_into_the_next_edit() {
    let mut ws = new_workspace();
    run(&mut ws, script().chars("ea").enter());
    assert!(!ws.state.dirty, "the edit box has no change after Enter");
    run(&mut ws, script().chars("eb").enter());
    assert_eq!(cell(&ws, 1, 1), "ab");
    run(&mut ws, script().chars("ec").esc().chars("ed").enter());
    assert_eq!(cell(&ws, 1, 1), "abd");
}

#[test]
fn test_parse_theme_cmd() {
    assert_eq!(parse("theme"), Ok(Some(Cmd::Theme(None))));
    assert_eq!(parse("theme dracula"), Ok(Some(Cmd::Theme(Some("dracula")))));
    assert!(parse("themes").is_err());
}

fn temp_config_dir(tag: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("sheetui-ui-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("themes")).unwrap();
    dir
}

fn message_text(ws: &Workspace) -> Option<String> {
    ws.state.message.as_ref().map(|m| m.text.clone())
}

#[test]
fn test_theme_cmd_message_is_cleared_by_the_next_key() {
    let mut ws = new_workspace();
    let before = ws.state.theme.clone();
    run_command(&mut ws, "theme nonesuch");
    let msg = ws.state.message.clone().expect("a message");
    assert_eq!(msg.level, super::MessageLevel::Error);
    assert!(msg.text.contains("nonesuch"), "{}", msg.text);
    assert_eq!(before, ws.state.theme, "the current theme stays");
    run(&mut ws, script().char('j'));
    assert_eq!(None, ws.state.message);
}

#[test]
fn test_theme_cmd_switches_theme() {
    let mut ws = new_workspace();
    run_command(&mut ws, "theme light");
    assert_eq!("light", ws.state.theme.name);
    assert_eq!(Some("theme light".to_owned()), message_text(&ws));
    assert_eq!(super::MessageLevel::Info, ws.state.message.as_ref().unwrap().level);
}

#[test]
fn test_theme_cmd_with_warnings_counts_them() {
    let mut ws = new_workspace();
    let dir = temp_config_dir("warn");
    ws.state.themes_dir = Some(dir.join("themes"));
    std::fs::write(dir.join("themes/bad.theme"), "style nope fg red\nstyle cell fg nocolor\n").unwrap();
    run_command(&mut ws, "theme bad");
    assert_eq!(
        Some("theme bad: 2 warnings (:theme to list)".to_owned()),
        message_text(&ws)
    );
    assert_eq!(2, ws.state.theme_warnings.len());
}

#[test]
fn test_theme_cmd_lists_builtin_and_user_themes() {
    let mut ws = new_workspace();
    let dir = temp_config_dir("list");
    ws.state.themes_dir = Some(dir.join("themes"));
    std::fs::write(dir.join("themes/mine.theme"), "style cell fg nocolor\n").unwrap();
    run_command(&mut ws, "theme mine");
    run_command(&mut ws, "theme");
    assert_eq!(&Modality::Dialog, ws.state.modality());
    let text = ws
        .state
        .popup
        .as_ref()
        .unwrap()
        .get_text()
        .lines
        .iter()
        .map(|l| l.to_string())
        .collect::<Vec<_>>()
        .join("\n");
    for needle in ["default", "light", "terminal", "mine", "current", "mine:1:"] {
        assert!(text.contains(needle), "{needle} missing from:\n{text}");
    }
}

#[test]
fn test_init_theme_flag_beats_config_and_bad_name_falls_back() {
    let dir = temp_config_dir("init");
    std::fs::write(dir.join("config"), "theme terminal\n").unwrap();
    let mut ws = new_workspace();
    ws.init_config_from(None, Some(dir.clone()));
    ws.init_theme_from(None, Some(dir.clone()));
    assert_eq!("terminal", ws.state.theme.name);
    ws.init_theme_from(Some("light"), Some(dir.clone()));
    assert_eq!("light", ws.state.theme.name);
    ws.init_theme_from(Some("nonesuch"), Some(dir.clone()));
    assert_eq!("default", ws.state.theme.name);
    assert_eq!(
        Some("1 warning: see :config".to_owned()),
        message_text(&ws)
    );
    let mut ws = new_workspace();
    ws.init_theme_from(None, None);
    assert_eq!("default", ws.state.theme.name);
    assert_eq!(None, ws.state.message);
}

#[test]
fn test_message_is_drawn_on_the_bottom_border() {
    use ratatui::{backend::TestBackend, Terminal};
    let mut ws = new_workspace();
    run_command(&mut ws, "theme light");
    let mut term = Terminal::new(TestBackend::new(60, 14)).expect("terminal");
    term.draw(|f| super::render::draw(f, &mut ws)).expect("draw");
    let buf = term.backend().buffer().clone();
    let last = buf.area.height - 1;
    let row: String = (0..buf.area.width).map(|x| buf[(x, last)].symbol().to_owned()).collect();
    let msg = row.find("theme light").expect("message on the bottom row");
    let modality = row.find("navigate").expect("modality on the bottom row");
    assert!(msg < modality, "{row}");
}

// ---------------------------------------------------------------------------
// Search: / and n
// ---------------------------------------------------------------------------

/// A workspace on a sheet with `x` at A1, C1, B2 and A3, cursor on A1.
fn search_workspace<'a>() -> Workspace<'a> {
    let book = book::Book::from_csv_reader("x,y,x\nz,x,z\nx,y,y\n".as_bytes(), None)
        .expect("csv");
    Workspace::new(book, std::path::PathBuf::from("search.sui"))
}

fn at(ws: &Workspace) -> (usize, usize) {
    (ws.book.location.row, ws.book.location.col)
}

#[test]
fn test_slash_opens_the_search_prompt() {
    let mut ws = search_workspace();
    run_command(&mut ws, "set noincsearch");
    script().char('/').run(&mut ws).expect("run");
    assert_eq!(Some(&Modality::Search), ws.state.modality_stack.last());
    script().chars("x").run(&mut ws).expect("run");
    assert_eq!("x", ws.state.search_prompt.value());
    // Without `incsearch` the cursor does not move until Enter.
    assert_eq!((1, 1), at(&ws));
}

#[test]
fn test_slash_pattern_enter_goes_to_the_next_match() {
    let mut ws = search_workspace();
    script().char('/').chars("x").enter().run(&mut ws).expect("run");
    assert_eq!(Some(&Modality::Navigate), ws.state.modality_stack.last());
    assert_eq!((1, 3), at(&ws));
    assert_eq!(None, last_message(&ws));
    // The edit box shows the new cell.
    assert_eq!(vec!["x".to_owned()], ws.text_area.lines());
}

#[test]
fn test_n_repeats_the_search_and_wraps() {
    let mut ws = search_workspace();
    script().char('/').chars("x").enter().run(&mut ws).expect("run");
    script().char('n').run(&mut ws).expect("run");
    assert_eq!((2, 2), at(&ws));
    script().char('n').run(&mut ws).expect("run");
    assert_eq!((3, 1), at(&ws));
    assert_eq!(None, last_message(&ws));
    script().char('n').run(&mut ws).expect("run");
    assert_eq!((1, 1), at(&ws));
    assert_eq!(
        Some("search hit BOTTOM, continuing at TOP".to_owned()),
        last_message(&ws)
    );
    // The message lasts until the next key.
    script().char('n').run(&mut ws).expect("run");
    assert_eq!((1, 3), at(&ws));
    assert_eq!(None, last_message(&ws));
}

#[test]
fn test_search_with_no_match_leaves_the_cursor() {
    let mut ws = search_workspace();
    script().char('l').char('/').chars("nope").enter().run(&mut ws).expect("run");
    assert_eq!((1, 2), at(&ws));
    assert_eq!(Some("E486: Pattern not found: nope".to_owned()), last_message(&ws));
    assert_eq!(Some(&Modality::Navigate), ws.state.modality_stack.last());
    script().char('n').run(&mut ws).expect("run");
    assert_eq!((1, 2), at(&ws));
    assert_eq!(Some("E486: Pattern not found: nope".to_owned()), last_message(&ws));
}

#[test]
fn test_search_in_an_empty_sheet() {
    let mut ws = new_workspace();
    script().char('/').chars(".").enter().run(&mut ws).expect("run");
    assert_eq!((1, 1), at(&ws));
    assert_eq!(Some("E486: Pattern not found: .".to_owned()), last_message(&ws));
}

#[test]
fn test_an_invalid_pattern_is_an_error_not_a_panic() {
    let mut ws = search_workspace();
    script().char('/').chars("x").enter().run(&mut ws).expect("run");
    script().char('/').chars("(x").enter().run(&mut ws).expect("run");
    assert_eq!((1, 3), at(&ws));
    assert_eq!(Some("Invalid pattern: unclosed group".to_owned()), last_message(&ws));
    assert_eq!(Some(&Modality::Navigate), ws.state.modality_stack.last());
    // The last good pattern is still the one `n` repeats.
    script().char('n').run(&mut ws).expect("run");
    assert_eq!((2, 2), at(&ws));
}

#[test]
fn test_esc_cancels_the_search_prompt() {
    let mut ws = search_workspace();
    script().char('/').chars("x").esc().run(&mut ws).expect("run");
    assert_eq!(Some(&Modality::Navigate), ws.state.modality_stack.last());
    assert_eq!((1, 1), at(&ws));
    assert_eq!(None, last_message(&ws));
    // Nothing was searched, so there is nothing to repeat.
    script().char('n').run(&mut ws).expect("run");
    assert_eq!((1, 1), at(&ws));
    assert_eq!(
        Some("E35: No previous regular expression".to_owned()),
        last_message(&ws)
    );
    // The prompt starts empty the next time.
    script().char('/').run(&mut ws).expect("run");
    assert_eq!("", ws.state.search_prompt.value());
}

#[test]
fn test_search_prompt_keys_are_text_not_commands() {
    let mut ws = search_workspace();
    script().char('/').chars("jq").run(&mut ws).expect("run");
    assert_eq!("jq", ws.state.search_prompt.value());
    assert_eq!((1, 1), at(&ws));
    assert_eq!(Some(&Modality::Search), ws.state.modality_stack.last());
}

#[test]
fn test_search_keys_can_be_remapped() {
    let mut ws = search_workspace();
    type_command(&mut ws, "nnoremap ; n");
    type_command(&mut ws, "nunmap /");
    script().char('/').run(&mut ws).expect("run");
    assert_eq!(Some(&Modality::Navigate), ws.state.modality_stack.last());
    type_command(&mut ws, "nnoremap <Space>/ /");
    script().chars(" /").chars("x").enter().run(&mut ws).expect("run");
    assert_eq!((1, 3), at(&ws));
    script().char(';').run(&mut ws).expect("run");
    assert_eq!((2, 2), at(&ws));
}

#[test]
fn test_search_prompt_is_drawn() {
    use ratatui::{backend::TestBackend, Terminal};
    let mut ws = search_workspace();
    script().char('/').chars("x+").run(&mut ws).expect("run");
    let mut term = Terminal::new(TestBackend::new(60, 16)).expect("terminal");
    term.draw(|f| super::render::draw(f, &mut ws)).expect("draw");
    let buf = term.backend().buffer().clone();
    let rows: Vec<String> = (0..buf.area.height)
        .map(|y| (0..buf.area.width).map(|x| buf[(x, y)].symbol().to_owned()).collect())
        .collect();
    assert!(rows.iter().any(|r| r.contains("/") && r.contains("x+")), "{rows:#?}");
    assert!(rows.last().unwrap().contains("search"), "{rows:#?}");
}

// ---------------------------------------------------------------------------
// Search: ?, N, counts and empty-pattern repeat
// ---------------------------------------------------------------------------

#[test]
fn test_question_mark_searches_backward_and_wraps() {
    let mut ws = search_workspace();
    script().char('?').run(&mut ws).expect("run");
    assert_eq!(Some(&Modality::Search), ws.state.modality_stack.last());
    script().chars("x").enter().run(&mut ws).expect("run");
    assert_eq!((3, 1), at(&ws));
    assert_eq!(
        Some("search hit TOP, continuing at BOTTOM".to_owned()),
        last_message(&ws)
    );
    // `n` keeps going backward.
    script().char('n').run(&mut ws).expect("run");
    assert_eq!((2, 2), at(&ws));
    assert_eq!(None, last_message(&ws));
    script().char('n').run(&mut ws).expect("run");
    assert_eq!((1, 3), at(&ws));
}

#[test]
fn test_capital_n_reverses_the_last_direction() {
    let mut ws = search_workspace();
    script().char('/').chars("x").enter().run(&mut ws).expect("run");
    assert_eq!((1, 3), at(&ws));
    script().char('N').run(&mut ws).expect("run");
    assert_eq!((1, 1), at(&ws));
    script().char('N').run(&mut ws).expect("run");
    assert_eq!((3, 1), at(&ws));
    assert_eq!(
        Some("search hit TOP, continuing at BOTTOM".to_owned()),
        last_message(&ws)
    );
    // `N` does not change the direction `n` goes.
    script().char('n').run(&mut ws).expect("run");
    assert_eq!((1, 1), at(&ws));
    assert_eq!(
        Some("search hit BOTTOM, continuing at TOP".to_owned()),
        last_message(&ws)
    );
    // After `?`, `N` goes forward.
    script().char('?').chars("x").enter().run(&mut ws).expect("run");
    assert_eq!((3, 1), at(&ws));
    script().char('N').run(&mut ws).expect("run");
    assert_eq!((1, 1), at(&ws));
    assert_eq!(
        Some("search hit BOTTOM, continuing at TOP".to_owned()),
        last_message(&ws)
    );
}

#[test]
fn test_capital_n_with_no_previous_search() {
    let mut ws = search_workspace();
    script().char('N').run(&mut ws).expect("run");
    assert_eq!((1, 1), at(&ws));
    assert_eq!(
        Some("E35: No previous regular expression".to_owned()),
        last_message(&ws)
    );
}

#[test]
fn test_counts_for_n_and_capital_n() {
    let mut ws = search_workspace();
    script().char('/').chars("x").enter().run(&mut ws).expect("run");
    assert_eq!((1, 3), at(&ws));
    script().chars("2n").run(&mut ws).expect("run");
    assert_eq!((3, 1), at(&ws));
    assert_eq!(None, last_message(&ws));
    script().chars("2N").run(&mut ws).expect("run");
    assert_eq!((1, 3), at(&ws));
    // A count that goes past the end wraps, and says so.
    script().chars("3n").run(&mut ws).expect("run");
    assert_eq!((1, 1), at(&ws));
    assert_eq!(
        Some("search hit BOTTOM, continuing at TOP".to_owned()),
        last_message(&ws)
    );
    // The count is used up.
    script().char('n').run(&mut ws).expect("run");
    assert_eq!((1, 3), at(&ws));
    // A count with no match still says E486 and leaves the cursor.
    script().char('/').chars("nope").enter().run(&mut ws).expect("run");
    script().chars("3n").run(&mut ws).expect("run");
    assert_eq!((1, 3), at(&ws));
    assert_eq!(Some("E486: Pattern not found: nope".to_owned()), last_message(&ws));
    // And the count does not leak into the next key.
    script().char('j').run(&mut ws).expect("run");
    assert_eq!((2, 3), at(&ws));
}

#[test]
fn test_empty_pattern_repeats_the_last_one() {
    let mut ws = search_workspace();
    script().char('/').chars("x").enter().run(&mut ws).expect("run");
    script().char('/').enter().run(&mut ws).expect("run");
    assert_eq!((2, 2), at(&ws));
    // `?` with no pattern repeats it backward, and `n` then goes backward.
    script().char('?').enter().run(&mut ws).expect("run");
    assert_eq!((1, 3), at(&ws));
    script().char('n').run(&mut ws).expect("run");
    assert_eq!((1, 1), at(&ws));
}

#[test]
fn test_empty_pattern_with_no_previous_search() {
    let mut ws = search_workspace();
    script().char('/').enter().run(&mut ws).expect("run");
    assert_eq!((1, 1), at(&ws));
    assert_eq!(
        Some("E35: No previous regular expression".to_owned()),
        last_message(&ws)
    );
    script().char('?').enter().run(&mut ws).expect("run");
    assert_eq!(
        Some("E35: No previous regular expression".to_owned()),
        last_message(&ws)
    );
    assert_eq!(Some(&Modality::Navigate), ws.state.modality_stack.last());
}

#[test]
fn test_esc_from_question_mark_keeps_the_direction() {
    let mut ws = search_workspace();
    script().char('/').chars("x").enter().run(&mut ws).expect("run");
    script().char('?').chars("y").esc().run(&mut ws).expect("run");
    script().char('n').run(&mut ws).expect("run");
    assert_eq!((2, 2), at(&ws));
}

#[test]
fn test_backward_search_prompt_is_drawn_with_a_question_mark() {
    use ratatui::{backend::TestBackend, Terminal};
    let mut ws = search_workspace();
    script().char('?').chars("x+").run(&mut ws).expect("run");
    let mut term = Terminal::new(TestBackend::new(60, 16)).expect("terminal");
    term.draw(|f| super::render::draw(f, &mut ws)).expect("draw");
    let buf = term.backend().buffer().clone();
    let rows: Vec<String> = (0..buf.area.height)
        .map(|y| (0..buf.area.width).map(|x| buf[(x, y)].symbol().to_owned()).collect())
        .collect();
    assert!(rows.iter().any(|r| r.contains("?") && r.contains("x+")), "{rows:#?}");
}

#[test]
fn test_a_huge_count_does_not_walk_every_lap() {
    let mut ws = search_workspace();
    script().char('/').chars("x").enter().run(&mut ws).expect("run");
    assert_eq!((1, 3), at(&ws));
    // Four matches: 4k + 1 steps from C1 lands on B2, the next one.
    script().chars("400000000001n").run(&mut ws).expect("run");
    assert_eq!((2, 2), at(&ws));
    assert_eq!(
        Some("search hit BOTTOM, continuing at TOP".to_owned()),
        last_message(&ws)
    );
}

// ---------------------------------------------------------------------------
// Options: :set and wrapscan
// ---------------------------------------------------------------------------

#[test]
fn test_parse_set_command() {
    use super::cmd::{parse, Cmd};
    assert_eq!(parse("set"), Ok(Some(Cmd::Set(""))));
    assert_eq!(parse("set nows ic?"), Ok(Some(Cmd::Set("nows ic?"))));
    assert!(parse("settings").is_err());
}

#[test]
fn test_set_shows_and_changes_wrapscan() {
    let mut ws = search_workspace();
    run_command(&mut ws, "set ws?");
    assert_eq!(Some("wrapscan".to_owned()), last_message(&ws));
    run_command(&mut ws, "set nowrapscan");
    assert!(!ws.state.options.wrapscan());
    run_command(&mut ws, "set");
    assert_eq!(
        Some("nowrapscan  noignorecase  nosmartcase  searchin=value  hlsearch  incsearch  defaultwidth=18".to_owned()),
        last_message(&ws)
    );
    run_command(&mut ws, "set ws!");
    assert!(ws.state.options.wrapscan());
}

#[test]
fn test_set_unknown_option_is_e518_on_the_message_line() {
    let mut ws = search_workspace();
    run_command(&mut ws, "set bogus");
    let msg = ws.state.message.as_ref().expect("message");
    assert_eq!("E518: Unknown option: bogus", msg.text);
    assert_eq!(msg.level, super::MessageLevel::Error);
    assert_eq!(Some(&Modality::Navigate), ws.state.modality_stack.last());
}

#[test]
fn test_nowrapscan_stops_at_the_bottom_with_e385() {
    let mut ws = search_workspace();
    run_command(&mut ws, "set nows");
    script().char('/').chars("x").enter().run(&mut ws).expect("run");
    script().chars("nn").run(&mut ws).expect("run");
    assert_eq!((3, 1), at(&ws));
    script().char('n').run(&mut ws).expect("run");
    assert_eq!((3, 1), at(&ws));
    assert_eq!(
        Some("E385: Search hit BOTTOM without match for: x".to_owned()),
        last_message(&ws)
    );
    // A count that runs past the end does not move at all.
    run_command(&mut ws, "set ws");
    script().char('n').run(&mut ws).expect("run");
    assert_eq!((1, 1), at(&ws));
    run_command(&mut ws, "set nows");
    script().chars("9n").run(&mut ws).expect("run");
    assert_eq!((1, 1), at(&ws));
    assert_eq!(
        Some("E385: Search hit BOTTOM without match for: x".to_owned()),
        last_message(&ws)
    );
}

#[test]
fn test_nowrapscan_stops_at_the_top_going_backward() {
    let mut ws = search_workspace();
    run_command(&mut ws, "set nows");
    script().char('?').chars("x").enter().run(&mut ws).expect("run");
    assert_eq!((1, 1), at(&ws));
    assert_eq!(
        Some("E385: Search hit TOP without match for: x".to_owned()),
        last_message(&ws)
    );
}

// ---------------------------------------------------------------------------
// Search: ignorecase, smartcase, \c and \C
// ---------------------------------------------------------------------------

/// A workspace with `apple` at A1, `Apple` at B1 and `APPLE` at C1, cursor on
/// A1.
fn case_workspace<'a>() -> Workspace<'a> {
    let book = book::Book::from_csv_reader("apple,Apple,APPLE\n".as_bytes(), None)
        .expect("csv");
    Workspace::new(book, std::path::PathBuf::from("case.sui"))
}

#[test]
fn test_search_matches_case_by_default() {
    let mut ws = case_workspace();
    script().char('/').chars("apple").enter().run(&mut ws).expect("run");
    // The only match is A1 itself, found again after wrapping.
    assert_eq!((1, 1), at(&ws));
}

#[test]
fn test_ignorecase_finds_every_case() {
    let mut ws = case_workspace();
    run_command(&mut ws, "set ic");
    script().char('/').chars("apple").enter().run(&mut ws).expect("run");
    assert_eq!((1, 2), at(&ws));
    script().char('n').run(&mut ws).expect("run");
    assert_eq!((1, 3), at(&ws));
}

#[test]
fn test_smartcase_matches_case_with_an_uppercase_letter() {
    let mut ws = case_workspace();
    run_command(&mut ws, "set ic scs");
    script().char('/').chars("Apple").enter().run(&mut ws).expect("run");
    assert_eq!((1, 2), at(&ws));
    script().char('n').run(&mut ws).expect("run");
    assert_eq!((1, 2), at(&ws));
    // All lowercase still ignores case.
    script().char('/').chars("apple").enter().run(&mut ws).expect("run");
    assert_eq!((1, 3), at(&ws));
}

#[test]
fn test_case_atoms_override_the_options() {
    let mut ws = case_workspace();
    script().char('/').chars("APPLE\\c").enter().run(&mut ws).expect("run");
    assert_eq!((1, 2), at(&ws));
    run_command(&mut ws, "set ic");
    script().char('/').chars("\\Capple").enter().run(&mut ws).expect("run");
    assert_eq!((1, 1), at(&ws));
}

#[test]
fn test_changing_ignorecase_after_a_search_changes_n() {
    let mut ws = case_workspace();
    script().char('/').chars("apple").enter().run(&mut ws).expect("run");
    assert_eq!((1, 1), at(&ws));
    run_command(&mut ws, "set ic");
    script().char('n').run(&mut ws).expect("run");
    assert_eq!((1, 2), at(&ws));
    run_command(&mut ws, "set noic");
    script().char('n').run(&mut ws).expect("run");
    assert_eq!((1, 1), at(&ws));
}


// ---------------------------------------------------------------------------
// Search: searchin
// ---------------------------------------------------------------------------

/// A workspace with `1` at A1, `=A1*2` at B1 and `$12.00` at C1, cursor on
/// A1.
fn searchin_workspace<'a>() -> Workspace<'a> {
    let mut book = book::Book::from_csv_reader("1\n".as_bytes(), None).expect("csv");
    for (col, text) in [(2, "=A1*2"), (3, "$12.00")] {
        book.update_cell(&Address { sheet: 0, row: 1, col }, text).expect("set");
    }
    book.evaluate();
    Workspace::new(book, std::path::PathBuf::from("searchin.sui"))
}

#[test]
fn test_searchin_value_matches_the_displayed_value() {
    let mut ws = searchin_workspace();
    script().char('/').chars("^2$").enter().run(&mut ws).expect("run");
    assert_eq!((1, 2), at(&ws));
    script().char('/').chars("A1").enter().run(&mut ws).expect("run");
    assert_eq!(Some("E486: Pattern not found: A1".to_owned()), last_message(&ws));
}

#[test]
fn test_searchin_raw_matches_formulas_and_stored_numbers() {
    let mut ws = searchin_workspace();
    run_command(&mut ws, "set searchin=raw");
    script().char('/').chars("A1\\*").enter().run(&mut ws).expect("run");
    assert_eq!((1, 2), at(&ws));
    script().char('/').chars("^12$").enter().run(&mut ws).expect("run");
    assert_eq!((1, 3), at(&ws));
    script().char('/').chars("\\$").enter().run(&mut ws).expect("run");
    assert_eq!(Some("E486: Pattern not found: \\$".to_owned()), last_message(&ws));
}

#[test]
fn test_changing_searchin_after_a_search_changes_n() {
    let mut ws = searchin_workspace();
    script().char('/').chars("2").enter().run(&mut ws).expect("run");
    assert_eq!((1, 2), at(&ws));
    run_command(&mut ws, "set searchin=raw");
    script().char('n').run(&mut ws).expect("run");
    // B1's raw `=A1*2` still has a 2, and so does C1's stored `12`.
    assert_eq!((1, 3), at(&ws));
    run_command(&mut ws, "set searchin=value");
    script().char('/').chars("^2$").enter().run(&mut ws).expect("run");
    assert_eq!((1, 2), at(&ws));
}

#[test]
fn test_set_searchin_bad_value_is_an_error() {
    let mut ws = searchin_workspace();
    run_command(&mut ws, "set searchin=formula");
    assert_eq!(
        Some("E474: Invalid argument: searchin=formula (one of: value, raw)".to_owned()),
        last_message(&ws)
    );
    assert_eq!(book::search::SearchIn::Value, ws.state.options.searchin());
    run_command(&mut ws, "set searchin?");
    assert_eq!(Some("searchin=value".to_owned()), last_message(&ws));
}

// ---------------------------------------------------------------------------
// Search: * and #
// ---------------------------------------------------------------------------

/// A workspace with `Paid` at A1 and B2, `Unpaid` at B1 and `paid` at A2,
/// cursor on A1.
fn star_workspace<'a>() -> Workspace<'a> {
    let book = book::Book::from_csv_reader("Paid,Unpaid\npaid,Paid\n".as_bytes(), None)
        .expect("csv");
    Workspace::new(book, std::path::PathBuf::from("star.sui"))
}

#[test]
fn test_star_finds_cells_like_this_one_and_skips_partial_matches() {
    let mut ws = star_workspace();
    script().char('*').run(&mut ws).expect("run");
    // `Unpaid` holds `Paid`, but `*` matches the whole cell.
    assert_eq!((2, 2), at(&ws));
    assert_eq!(None, last_message(&ws));
    // It is the last search now, going forward: `n` wraps back to A1.
    script().char('n').run(&mut ws).expect("run");
    assert_eq!((1, 1), at(&ws));
    assert_eq!(
        Some("search hit BOTTOM, continuing at TOP".to_owned()),
        last_message(&ws)
    );
    script().char('N').run(&mut ws).expect("run");
    assert_eq!((2, 2), at(&ws));
    let case = Default::default();
    let pattern = ws.state.search.last(case).map(|l| l.expect("valid").0.to_owned());
    assert_eq!(Some("^Paid$".to_owned()), pattern);
}

#[test]
fn test_hash_searches_backward_and_n_keeps_going_backward() {
    let mut ws = star_workspace();
    script().char('#').run(&mut ws).expect("run");
    assert_eq!((2, 2), at(&ws));
    assert_eq!(
        Some("search hit TOP, continuing at BOTTOM".to_owned()),
        last_message(&ws)
    );
    script().char('n').run(&mut ws).expect("run");
    assert_eq!((1, 1), at(&ws));
    assert_eq!(Direction::Backward, ws.state.search.direction);
}

#[test]
fn test_star_follows_ignorecase() {
    let mut ws = star_workspace();
    run_command(&mut ws, "set ignorecase");
    script().char('*').run(&mut ws).expect("run");
    assert_eq!((2, 1), at(&ws));
}

#[test]
fn test_star_takes_a_count() {
    let mut ws = search_workspace();
    script().chars("2*").run(&mut ws).expect("run");
    assert_eq!((2, 2), at(&ws));
}

#[test]
fn test_star_on_an_empty_cell_is_an_error() {
    let mut ws = star_workspace();
    script().char('/').chars("Unpaid").enter().run(&mut ws).expect("run");
    script().chars("jj").run(&mut ws).expect("run");
    assert_eq!((3, 2), at(&ws));
    for key in ['*', '#'] {
        script().char(key).run(&mut ws).expect("run");
        assert_eq!((3, 2), at(&ws));
        assert_eq!(Some("E348: No string under cursor".to_owned()), last_message(&ws));
    }
    // The last search is unchanged.
    script().char('n').run(&mut ws).expect("run");
    assert_eq!((1, 2), at(&ws));
}

#[test]
fn test_star_escapes_regex_metacharacters() {
    let text = "a.b(c)$,axb(c)$,a.b(c)$\n[x]+?*^|\\{1},x,[x]+?*^|\\{1}\n";
    let book = book::Book::from_csv_reader(text.as_bytes(), None).expect("csv");
    let mut ws = Workspace::new(book, std::path::PathBuf::from("meta.sui"));
    script().char('*').run(&mut ws).expect("run");
    assert_eq!((1, 3), at(&ws));
    script().char('j').char('*').run(&mut ws).expect("run");
    assert_eq!((2, 1), at(&ws));
    assert_eq!(None, last_message(&ws).filter(|m| !m.starts_with("search hit")));
    script().char('*').run(&mut ws).expect("run");
    assert_eq!((2, 3), at(&ws));
}

/// A workspace with `$12.00` at A1, `12` at B1, `$12.00` at C1, and
/// `=B1*2` at A2 and B2, cursor on A1.
fn star_formats_workspace<'a>() -> Workspace<'a> {
    let mut book = book::Book::from_csv_reader("".as_bytes(), None).expect("csv");
    let cells = [
        (1, 1, "$12.00"),
        (1, 2, "12"),
        (1, 3, "$12.00"),
        (2, 1, "=B1*2"),
        (2, 2, "=B1*2"),
    ];
    for (row, col, text) in cells {
        book.update_cell(&Address { sheet: 0, row, col }, text).expect("set");
    }
    book.evaluate();
    Workspace::new(book, std::path::PathBuf::from("formats.sui"))
}

#[test]
fn test_star_on_a_currency_cell_uses_the_shown_value() {
    let mut ws = star_formats_workspace();
    script().char('*').run(&mut ws).expect("run");
    // B1 shows `12`, not `$12.00`.
    assert_eq!((1, 3), at(&ws));
    let case = Default::default();
    let pattern = ws.state.search.last(case).map(|l| l.expect("valid").0.to_owned());
    assert_eq!(Some("^\\$12\\.00$".to_owned()), pattern);
}

#[test]
fn test_star_with_searchin_raw_uses_the_stored_text() {
    let mut ws = star_formats_workspace();
    run_command(&mut ws, "set searchin=raw");
    // A1 stores `12`, like B1.
    script().char('*').run(&mut ws).expect("run");
    assert_eq!((1, 2), at(&ws));
    // A formula is matched on its formula text.
    script().char('j').char('*').run(&mut ws).expect("run");
    assert_eq!((2, 1), at(&ws));
    script().char('*').run(&mut ws).expect("run");
    assert_eq!((2, 2), at(&ws));
    let case = Default::default();
    let pattern = ws.state.search.last(case).map(|l| l.expect("valid").0.to_owned());
    assert_eq!(Some("^=B1\\*2$".to_owned()), pattern);
}

#[test]
fn test_star_on_a_formula_with_searchin_value_uses_its_result() {
    let mut ws = star_formats_workspace();
    script().char('j').char('*').run(&mut ws).expect("run");
    assert_eq!((2, 2), at(&ws));
    let case = Default::default();
    let pattern = ws.state.search.last(case).map(|l| l.expect("valid").0.to_owned());
    assert_eq!(Some("^24$".to_owned()), pattern);
}

#[test]
fn test_star_and_hash_can_be_remapped() {
    let mut ws = star_workspace();
    type_command(&mut ws, "nnoremap ; *");
    type_command(&mut ws, "nunmap #");
    script().char(';').run(&mut ws).expect("run");
    assert_eq!((2, 2), at(&ws));
    script().char('#').run(&mut ws).expect("run");
    assert_eq!((2, 2), at(&ws));
    assert_eq!(Direction::Forward, ws.state.search.direction);
}

// ---------------------------------------------------------------------------
// Search: hlsearch and :noh
// ---------------------------------------------------------------------------

#[test]
fn test_parse_nohlsearch() {
    use super::cmd::{parse, Cmd};
    for text in ["noh", "nohl", "nohls", "nohlsearch", "noh "] {
        assert_eq!(parse(text), Ok(Some(Cmd::NoHighlight)), "{text}");
    }
    assert!(parse("noh x").is_err());
    assert!(parse("nohlsearchx").is_err());
    assert!(parse("nohx").is_err());
}

/// The pattern being highlighted, if any.
fn highlighted(ws: &mut Workspace) -> Option<String> {
    ws.highlight_matcher().map(|(re, _)| re.as_str().to_owned())
}

#[test]
fn test_hlsearch_highlights_after_a_search() {
    let mut ws = search_workspace();
    assert!(ws.state.options.hlsearch());
    assert_eq!(None, highlighted(&mut ws));
    script().char('/').chars("x").enter().run(&mut ws).expect("run");
    assert_eq!(Some("x".to_owned()), highlighted(&mut ws));
    // A miss still highlights, as in vim; there is just nothing to see.
    script().char('/').chars("nope").enter().run(&mut ws).expect("run");
    assert_eq!(Some("nope".to_owned()), highlighted(&mut ws));
    // An invalid pattern keeps the last one.
    script().char('/').chars("(").enter().run(&mut ws).expect("run");
    assert_eq!(Some("nope".to_owned()), highlighted(&mut ws));
}

#[test]
fn test_noh_clears_the_highlight_until_the_next_search() {
    let mut ws = search_workspace();
    script().char('/').chars("x").enter().run(&mut ws).expect("run");
    run_command(&mut ws, "noh");
    assert_eq!(None, highlighted(&mut ws));
    assert_eq!(None, last_message(&ws));
    // `:noh` keeps the pattern: `n` still searches, and highlights again.
    script().char('n').run(&mut ws).expect("run");
    assert_eq!((2, 2), at(&ws));
    assert_eq!(Some("x".to_owned()), highlighted(&mut ws));
    run_command(&mut ws, "nohlsearch");
    assert_eq!(None, highlighted(&mut ws));
    script().char('/').chars("y").enter().run(&mut ws).expect("run");
    assert_eq!(Some("y".to_owned()), highlighted(&mut ws));
    // `:noh` with no search is not an error.
    let mut fresh = search_workspace();
    run_command(&mut fresh, "noh");
    assert_eq!(None, last_message(&fresh));
}

#[test]
fn test_set_nohlsearch_turns_highlighting_off() {
    let mut ws = search_workspace();
    script().char('/').chars("x").enter().run(&mut ws).expect("run");
    run_command(&mut ws, "set nohls");
    assert!(!ws.state.options.hlsearch());
    assert_eq!(None, highlighted(&mut ws));
    // Searching does not turn it on.
    script().char('n').run(&mut ws).expect("run");
    assert_eq!(None, highlighted(&mut ws));
    run_command(&mut ws, "set hlsearch");
    assert_eq!(Some("x".to_owned()), highlighted(&mut ws));
}

#[test]
fn test_setting_hlsearch_undoes_noh() {
    let mut ws = search_workspace();
    script().char('/').chars("x").enter().run(&mut ws).expect("run");
    run_command(&mut ws, "noh");
    // As in vim, `:set hls` shows the highlight again even if it was on.
    run_command(&mut ws, "set hls");
    assert_eq!(Some("x".to_owned()), highlighted(&mut ws));
    // Other `:set` commands do not.
    run_command(&mut ws, "noh");
    run_command(&mut ws, "set ic");
    assert_eq!(None, highlighted(&mut ws));
}

// ---------------------------------------------------------------------------
// Search: incsearch
// ---------------------------------------------------------------------------

fn backspace() -> Event {
    construct_key_event(KeyCode::Backspace)
}

#[test]
fn test_incsearch_previews_each_key() {
    let mut ws = search_workspace();
    assert!(ws.state.options.incsearch());
    script().char('/').char('y').run(&mut ws).expect("run");
    assert_eq!(Some(&Modality::Search), ws.state.modality_stack.last());
    assert_eq!((1, 2), at(&ws));
    assert_eq!(Some("y".to_owned()), highlighted(&mut ws));
    // The edit box follows the cursor.
    assert_eq!(vec!["y".to_owned()], ws.text_area.lines());
    // Each key searches again from where the prompt opened.
    script().event(backspace()).char('z').run(&mut ws).expect("run");
    assert_eq!((2, 1), at(&ws));
    assert_eq!(Some("z".to_owned()), highlighted(&mut ws));
    // No match goes back to the start, quietly.
    script().char('q').run(&mut ws).expect("run");
    assert_eq!((1, 1), at(&ws));
    assert_eq!(None, last_message(&ws));
    // An empty prompt goes back to the start too, with no highlight.
    script().event(backspace()).event(backspace()).run(&mut ws).expect("run");
    assert_eq!((1, 1), at(&ws));
    assert_eq!(None, highlighted(&mut ws));
}

#[test]
fn test_incsearch_backward_previews_without_a_wrap_message() {
    let mut ws = search_workspace();
    script().char('?').char('x').run(&mut ws).expect("run");
    assert_eq!((3, 1), at(&ws));
    assert_eq!(None, last_message(&ws));
}

#[test]
fn test_incsearch_enter_keeps_the_previewed_match() {
    let mut ws = search_workspace();
    script().char('/').chars("x").enter().run(&mut ws).expect("run");
    assert_eq!(Some(&Modality::Navigate), ws.state.modality_stack.last());
    // Not the match after the previewed one.
    assert_eq!((1, 3), at(&ws));
    assert_eq!(Some("x".to_owned()), highlighted(&mut ws));
    script().char('n').run(&mut ws).expect("run");
    assert_eq!((2, 2), at(&ws));
    // A wrap still says so on Enter.
    script().char('/').chars("y").enter().run(&mut ws).expect("run");
    assert_eq!((3, 2), at(&ws));
    script().char('?').chars("z").enter().run(&mut ws).expect("run");
    assert_eq!((2, 3), at(&ws));
    script().char('/').chars("x").enter().run(&mut ws).expect("run");
    assert_eq!((3, 1), at(&ws));
    script().char('/').chars("y").enter().run(&mut ws).expect("run");
    assert_eq!((3, 2), at(&ws));
    script().char('/').chars("z").enter().run(&mut ws).expect("run");
    assert_eq!((2, 1), at(&ws));
    assert_eq!(
        Some("search hit BOTTOM, continuing at TOP".to_owned()),
        last_message(&ws)
    );
}

#[test]
fn test_incsearch_esc_restores_the_cursor_and_highlight() {
    let mut ws = search_workspace();
    script().char('/').chars("x").enter().run(&mut ws).expect("run");
    assert_eq!((1, 3), at(&ws));
    script().char('/').chars("z").run(&mut ws).expect("run");
    assert_eq!((2, 1), at(&ws));
    assert_eq!(Some("z".to_owned()), highlighted(&mut ws));
    script().esc().run(&mut ws).expect("run");
    assert_eq!(Some(&Modality::Navigate), ws.state.modality_stack.last());
    assert_eq!((1, 3), at(&ws));
    assert_eq!(vec!["x".to_owned()], ws.text_area.lines());
    assert_eq!(Some("x".to_owned()), highlighted(&mut ws));
    // `n` still repeats the search before the prompt.
    script().char('n').run(&mut ws).expect("run");
    assert_eq!((2, 2), at(&ws));
    // After `:noh` the preview highlights, and Esc hides it again.
    run_command(&mut ws, "noh");
    script().char('/').chars("y").run(&mut ws).expect("run");
    assert_eq!(Some("y".to_owned()), highlighted(&mut ws));
    script().esc().run(&mut ws).expect("run");
    assert_eq!((2, 2), at(&ws));
    assert_eq!(None, highlighted(&mut ws));
}

#[test]
fn test_incsearch_invalid_partial_pattern_is_silent_until_enter() {
    let mut ws = search_workspace();
    script().char('/').chars("x(").run(&mut ws).expect("run");
    // The last good preview stays: no error, no jump back.
    assert_eq!((1, 3), at(&ws));
    assert_eq!(None, last_message(&ws));
    assert_eq!(Some("x".to_owned()), highlighted(&mut ws));
    script().chars("|z)").run(&mut ws).expect("run");
    assert_eq!((1, 3), at(&ws));
    assert_eq!(Some("x(|z)".to_owned()), highlighted(&mut ws));
    script().event(backspace()).enter().run(&mut ws).expect("run");
    // Enter on an invalid pattern says so and leaves the cursor at the start.
    assert_eq!((1, 1), at(&ws));
    assert_eq!(Some("Invalid pattern: unclosed group".to_owned()), last_message(&ws));
    assert_eq!(None, highlighted(&mut ws));
}

#[test]
fn test_noincsearch_turns_the_preview_off() {
    let mut ws = search_workspace();
    run_command(&mut ws, "set nois");
    assert!(!ws.state.options.incsearch());
    script().char('/').chars("y").run(&mut ws).expect("run");
    assert_eq!((1, 1), at(&ws));
    assert_eq!(None, highlighted(&mut ws));
    script().enter().run(&mut ws).expect("run");
    assert_eq!((1, 2), at(&ws));
    run_command(&mut ws, "set is");
    script().char('/').chars("z").run(&mut ws).expect("run");
    assert_eq!((2, 1), at(&ws));
}

// ---------------------------------------------------------------------------
// Search: history
// ---------------------------------------------------------------------------

fn search_prompt_text(ws: &Workspace) -> String {
    use tui_prompts::State;
    ws.state.search_prompt.value().to_owned()
}

#[test]
fn test_search_history_up_and_down() {
    let mut ws = search_workspace();
    script().char('/').chars("x").enter().run(&mut ws).expect("run");
    script().char('?').chars("y").enter().run(&mut ws).expect("run");
    script().char('/').up().run(&mut ws).expect("run");
    assert_eq!(search_prompt_text(&ws), "y");
    script().up().run(&mut ws).expect("run");
    assert_eq!(search_prompt_text(&ws), "x");
    script().up().run(&mut ws).expect("run");
    assert_eq!(search_prompt_text(&ws), "x");
    script().down().run(&mut ws).expect("run");
    assert_eq!(search_prompt_text(&ws), "y");
    script().down().run(&mut ws).expect("run");
    assert_eq!(search_prompt_text(&ws), "");
}

#[test]
fn test_search_history_prefix_match_and_draft_restore() {
    let mut ws = search_workspace();
    for pattern in ["x|y", "z", "x$"] {
        script().char('/').chars(pattern).enter().run(&mut ws).expect("run");
    }
    script().char('/').chars("x").up().run(&mut ws).expect("run");
    assert_eq!(search_prompt_text(&ws), "x$");
    script().up().run(&mut ws).expect("run");
    assert_eq!(search_prompt_text(&ws), "x|y");
    script().down().down().run(&mut ws).expect("run");
    assert_eq!(search_prompt_text(&ws), "x");
}

#[test]
fn test_search_history_is_separate_from_command_history() {
    let mut ws = search_workspace();
    run_command(&mut ws, "set ic");
    script().char('/').chars("z").enter().run(&mut ws).expect("run");
    assert_eq!(ws.state.command_history.older("").as_deref(), Some("set ic"));
    assert_eq!(ws.state.command_history.older("set ic"), None);
    script().char('/').up().run(&mut ws).expect("run");
    assert_eq!(search_prompt_text(&ws), "z");
    script().up().run(&mut ws).expect("run");
    assert_eq!(search_prompt_text(&ws), "z");
}

#[test]
fn test_search_history_skips_cancelled_and_empty_patterns() {
    let mut ws = search_workspace();
    script().char('/').chars("x").enter().run(&mut ws).expect("run");
    script().char('/').chars("q").esc().run(&mut ws).expect("run");
    // An empty pattern repeats the last search but adds nothing.
    script().char('/').enter().run(&mut ws).expect("run");
    script().char('/').up().run(&mut ws).expect("run");
    assert_eq!(search_prompt_text(&ws), "x");
    script().up().run(&mut ws).expect("run");
    assert_eq!(search_prompt_text(&ws), "x");
}

#[test]
fn test_search_history_records_an_invalid_pattern() {
    let mut ws = search_workspace();
    script().char('/').chars("x(").enter().run(&mut ws).expect("run");
    script().char('/').up().run(&mut ws).expect("run");
    assert_eq!(search_prompt_text(&ws), "x(");
}

#[test]
fn test_search_history_recall_moves_the_preview_and_enter_searches() {
    let mut ws = search_workspace();
    script().char('/').chars("z").enter().run(&mut ws).expect("run");
    assert_eq!((2, 1), at(&ws));
    script().char('?').chars("y").enter().run(&mut ws).expect("run");
    assert_eq!((1, 2), at(&ws));
    script().char('/').up().up().run(&mut ws).expect("run");
    assert_eq!(search_prompt_text(&ws), "z");
    // The preview follows the recalled pattern.
    assert_eq!((2, 1), at(&ws));
    assert_eq!(Some("z".to_owned()), highlighted(&mut ws));
    script().enter().run(&mut ws).expect("run");
    assert_eq!((2, 1), at(&ws));
}

#[test]
fn test_star_and_hash_patterns_go_into_the_search_history() {
    let mut ws = star_workspace();
    script().char('*').run(&mut ws).expect("run");
    script().char('/').up().run(&mut ws).expect("run");
    assert_eq!(search_prompt_text(&ws), "^Paid$");
    script().esc().char('h').char('#').run(&mut ws).expect("run");
    script().char('/').up().run(&mut ws).expect("run");
    assert_eq!(search_prompt_text(&ws), "^paid$");
}

// ---------------------------------------------------------------------------
// Replace: :s on the current cell
// ---------------------------------------------------------------------------

/// A workspace with `banana,=SUM(C1:D1)` in row 1 and `2,3` in C1 and D1,
/// cursor on A1.
fn sub_workspace<'a>() -> Workspace<'a> {
    let mut book = book::Book::from_csv_reader("banana,,2,3\n".as_bytes(), None).expect("csv");
    book.update_cell(&Address { sheet: 0, row: 1, col: 2 }, "=SUM(C1:D1)")
        .expect("set");
    book.evaluate();
    Workspace::new(book, std::path::PathBuf::from("sub.sui"))
}

fn cell_text(ws: &Workspace, row: usize, col: usize) -> String {
    ws.book
        .get_cell_addr_contents(&Address { sheet: 0, row, col })
        .expect("contents")
}

#[test]
fn test_parse_substitute_command() {
    use super::cmd::{parse, Cmd};
    use crate::book::substitute::Scope;
    let cell = |text| Ok(Some(Cmd::Substitute { scope: Scope::Cell, text }));
    assert_eq!(parse("s/a/b/g"), cell("/a/b/g"));
    assert_eq!(parse("s#a#b#"), cell("#a#b#"));
    assert_eq!(parse("substitute/a/b/"), cell("/a/b/"));
    assert!(parse("s").is_err());
    // Other commands that start with `s` still parse as themselves.
    assert_eq!(parse("set ic"), Ok(Some(Cmd::Set("ic"))));
    assert!(matches!(parse("sort"), Ok(Some(Cmd::Sort { .. }))));
    assert_eq!(parse("system-paste"), Ok(Some(Cmd::SystemPaste)));
    assert_eq!(parse("sx/a/b/"), Ok(None));
}

#[test]
fn test_substitute_replaces_in_the_current_cell_only() {
    let mut ws = sub_workspace();
    run_command(&mut ws, "s/a/o/g");
    assert_eq!(cell_text(&ws, 1, 1), "bonono");
    assert_eq!(Some("3 substitutions on 1 cell".to_owned()), last_message(&ws));
    assert_eq!(ws.state.message.as_ref().map(|m| m.level), Some(super::MessageLevel::Info));
    assert_eq!(Some(&Modality::Navigate), ws.state.modality_stack.last());
    run_command(&mut ws, "s/o/a/");
    assert_eq!(cell_text(&ws, 1, 1), "banono");
    assert_eq!(Some("1 substitution on 1 cell".to_owned()), last_message(&ws));
}

#[test]
fn test_substitute_edits_the_formula_even_with_searchin_value() {
    let mut ws = sub_workspace();
    script().char('l').run(&mut ws).expect("run");
    run_command(&mut ws, "set searchin=value");
    run_command(&mut ws, "s/SUM/PRODUCT/");
    assert_eq!(cell_text(&ws, 1, 2), "=PRODUCT(C1:D1)");
    assert_eq!(ws.book.get_cell_addr_rendered(&Address { sheet: 0, row: 1, col: 2 }).expect("value"), "6");
}

#[test]
fn test_substitute_no_match_is_e486_and_e_silences_it() {
    let mut ws = sub_workspace();
    run_command(&mut ws, "s/z/o/");
    assert_eq!(Some("E486: Pattern not found: z".to_owned()), last_message(&ws));
    assert_eq!(ws.state.message.as_ref().map(|m| m.level), Some(super::MessageLevel::Error));
    run_command(&mut ws, "s/z/o/e");
    assert_eq!(Some("0 substitutions on 0 cells".to_owned()), last_message(&ws));
    assert_eq!(cell_text(&ws, 1, 1), "banana");
}

#[test]
fn test_substitute_is_one_undo_step_and_a_no_op_is_none() {
    let mut ws = sub_workspace();
    run_command(&mut ws, "s/a/o/g");
    run_command(&mut ws, "s/z/o/e");
    // Had the no-op been a step, this `u` would undo it and change nothing.
    script().char('u').run(&mut ws).expect("run");
    assert_eq!(cell_text(&ws, 1, 1), "banana");
    assert_eq!(cell_text(&ws, 1, 2), "=SUM(C1:D1)");
    script().char('r').run(&mut ws).expect("run");
    assert_eq!(cell_text(&ws, 1, 1), "bonono");
}

#[test]
fn test_substitute_errors_go_to_the_message_line() {
    let mut ws = sub_workspace();
    for (command, expected) in [
        ("s/a/b/x", "E488: Unknown flag of :s: x"),
        ("s/a/\\n/", "Cannot use \\n or \\r in a replacement: a cell holds one line"),
        ("s/(/b/", ""),
    ] {
        run_command(&mut ws, command);
        let msg = ws.state.message.as_ref().expect("message");
        assert_eq!(msg.level, super::MessageLevel::Error, "{}", command);
        assert!(msg.text.starts_with(expected), "{}: {}", command, msg.text);
        assert_eq!(cell_text(&ws, 1, 1), "banana");
    }
}

#[test]
fn test_substitute_array_formula_is_refused() {
    let mut book = book::Book::from_csv_reader("3,\n1,\n".as_bytes(), None).expect("csv");
    book.update_cell(&Address { sheet: 0, row: 1, col: 2 }, "=SORT(A1:A2)").expect("set");
    book.evaluate();
    let mut ws = Workspace::new(book, std::path::PathBuf::from("arr.sui"));
    script().char('l').run(&mut ws).expect("run");
    run_command(&mut ws, "s/SORT/SUM/");
    assert_eq!(Some("Cannot substitute in an array formula.".to_owned()), last_message(&ws));
    assert_eq!(cell_text(&ws, 1, 2), "=SORT(A1:A2)");
}

#[test]
fn test_substitute_uses_case_options_and_case_atoms() {
    let mut ws = sub_workspace();
    run_command(&mut ws, "s/BAN/x/");
    assert_eq!(cell_text(&ws, 1, 1), "banana");
    run_command(&mut ws, "s/\\cBAN/x/");
    assert_eq!(cell_text(&ws, 1, 1), "xana");
    run_command(&mut ws, "set ic");
    run_command(&mut ws, "s/X/b/");
    assert_eq!(cell_text(&ws, 1, 1), "bana");
}

#[test]
fn test_substitute_empty_pattern_uses_the_last_search() {
    let mut ws = sub_workspace();
    run_command(&mut ws, "s//x/");
    assert_eq!(Some("E35: No previous regular expression".to_owned()), last_message(&ws));
    script().char('/').chars("an").enter().run(&mut ws).expect("run");
    assert_eq!((1, 1), at(&ws));
    run_command(&mut ws, "s//[&]/g");
    assert_eq!(cell_text(&ws, 1, 1), "b[an][an]a");
}

#[test]
fn test_substitute_i_flag_ignores_case_over_the_options() {
    let mut ws = sub_workspace();
    run_command(&mut ws, "s/BAN/x/i");
    assert_eq!(cell_text(&ws, 1, 1), "xana");
    // `i` ignores case even when smartcase would not.
    run_command(&mut ws, "set ic scs");
    run_command(&mut ws, "s/A/o/gi");
    assert_eq!(cell_text(&ws, 1, 1), "xono");
    assert_eq!(Some("2 substitutions on 1 cell".to_owned()), last_message(&ws));
}

#[test]
fn test_substitute_big_i_flag_matches_case_over_the_options() {
    let mut ws = sub_workspace();
    run_command(&mut ws, "set ic");
    run_command(&mut ws, "s/BAN/x/I");
    assert_eq!(Some("E486: Pattern not found: BAN".to_owned()), last_message(&ws));
    assert_eq!(cell_text(&ws, 1, 1), "banana");
    run_command(&mut ws, "s/a/o/gI");
    assert_eq!(cell_text(&ws, 1, 1), "bonono");
    // `\c` in the pattern still wins over the flag.
    run_command(&mut ws, "s/\\cB/c/I");
    assert_eq!(cell_text(&ws, 1, 1), "conono");
}

#[test]
fn test_substitute_case_flags_with_an_empty_pattern_leave_the_search_alone() {
    let mut ws = grid_workspace();
    ws.book.update_cell(&Address { sheet: 0, row: 3, col: 3 }, "A").expect("set");
    script().char('/').chars("a").enter().run(&mut ws).expect("run");
    run_command(&mut ws, "%s//b/gi");
    assert_eq!(vec!["bbb", "bbb", "bbb"], grid_rows(&ws));
    // The flag was for that command only: `n` still matches case.
    ws.book.update_cell(&Address { sheet: 0, row: 3, col: 3 }, "A").expect("set");
    ws.book.update_cell(&Address { sheet: 0, row: 1, col: 1 }, "a").expect("set");
    assert_eq!((1, 2), at(&ws));
    // From B1, an `n` that ignored case would stop on C3 before A1.
    script().char('n').run(&mut ws).expect("run");
    assert_eq!((1, 1), at(&ws));
}

#[test]
fn test_substitute_n_flag_counts_and_changes_nothing() {
    let mut ws = grid_workspace();
    run_command(&mut ws, "%s/a/b/n");
    assert_eq!(Some("9 matches on 9 cells".to_owned()), last_message(&ws));
    assert_eq!(vec!["aaa", "aaa", "aaa"], grid_rows(&ws));
    ws.book.update_cell(&Address { sheet: 0, row: 1, col: 1 }, "banana").expect("set");
    run_command(&mut ws, "s/a/b/n");
    assert_eq!(Some("1 match on 1 cell".to_owned()), last_message(&ws));
    run_command(&mut ws, "s/a/b/gn");
    assert_eq!(Some("3 matches on 1 cell".to_owned()), last_message(&ws));
    run_command(&mut ws, "s/A/b/gni");
    assert_eq!(Some("3 matches on 1 cell".to_owned()), last_message(&ws));
    assert_eq!(cell_text(&ws, 1, 1), "banana");
    run_command(&mut ws, "s/z/b/n");
    assert_eq!(Some("E486: Pattern not found: z".to_owned()), last_message(&ws));
    run_command(&mut ws, "s/z/b/ne");
    assert_eq!(Some("0 matches on 0 cells".to_owned()), last_message(&ws));
}

#[test]
fn test_substitute_n_flag_is_no_undo_step() {
    let mut ws = sub_workspace();
    run_command(&mut ws, "s/a/o/g");
    run_command(&mut ws, "s/o/a/gn");
    assert_eq!(cell_text(&ws, 1, 1), "bonono");
    // Had the count been a step, this `u` would undo it and change nothing.
    script().char('u').run(&mut ws).expect("run");
    assert_eq!(cell_text(&ws, 1, 1), "banana");
}

#[test]
fn test_substitute_n_flag_counts_array_formulas() {
    let mut book = book::Book::from_csv_reader("3,\n1,\n".as_bytes(), None).expect("csv");
    book.update_cell(&Address { sheet: 0, row: 1, col: 2 }, "=SORT(A1:A2)").expect("set");
    book.evaluate();
    let mut ws = Workspace::new(book, std::path::PathBuf::from("arr.sui"));
    script().char('l').run(&mut ws).expect("run");
    run_command(&mut ws, "s/SORT/SUM/n");
    assert_eq!(Some("1 match on 1 cell".to_owned()), last_message(&ws));
    assert_eq!(cell_text(&ws, 1, 2), "=SORT(A1:A2)");
}

#[test]
fn test_substitute_goes_into_the_command_history() {
    let mut ws = sub_workspace();
    run_command(&mut ws, "s/a/o/");
    script().char(':').up().run(&mut ws).expect("run");
    assert_eq!(ws.state.command_state.value(), "s/a/o/");
}

// ---------------------------------------------------------------------------
// Replace: :%s and :'<,'>s scopes
// ---------------------------------------------------------------------------

/// A workspace with a 3 by 3 grid of `a`s, cursor on A1.
fn grid_workspace<'a>() -> Workspace<'a> {
    let book = book::Book::from_csv_reader("a,a,a\na,a,a\na,a,a\n".as_bytes(), None)
        .expect("csv");
    Workspace::new(book, std::path::PathBuf::from("grid.sui"))
}

/// The grid's cells, row by row, as one string per row.
fn grid_rows(ws: &Workspace) -> Vec<String> {
    (1..=3)
        .map(|row| (1..=3).map(|col| cell_text(ws, row, col)).collect())
        .collect()
}

#[test]
fn test_parse_substitute_scopes() {
    use super::cmd::{parse, Cmd};
    use crate::book::substitute::Scope;
    assert_eq!(
        parse("%s/a/b/g"),
        Ok(Some(Cmd::Substitute { scope: Scope::Sheet, text: "/a/b/g" }))
    );
    assert_eq!(
        parse("%substitute#a#b#"),
        Ok(Some(Cmd::Substitute { scope: Scope::Sheet, text: "#a#b#" }))
    );
    assert_eq!(
        parse("'<,'>s/a/b/"),
        Ok(Some(Cmd::Substitute { scope: Scope::Selection, text: "/a/b/" }))
    );
    assert!(parse("%s").is_err());
    assert!(parse("'<,'>s").is_err());
    // Only `:s` takes `%`.
    assert!(parse("%sort").is_err());
    assert!(parse("%w").is_err());
    // Other commands take the selection range `:` pre-fills in Range
    // Select, and ignore it: they act on the selection there already.
    assert_eq!(parse("'<,'>w"), Ok(Some(Cmd::Write(None))));
    assert!(matches!(parse("'<,'>sort"), Ok(Some(Cmd::Sort { .. }))));
    assert_eq!(parse("'<,'>sx/a/b/"), Ok(None));
}

#[test]
fn test_substitute_percent_replaces_in_the_whole_sheet() {
    let mut ws = grid_workspace();
    run_command(&mut ws, "%s/a/b/");
    assert_eq!(vec!["bbb", "bbb", "bbb"], grid_rows(&ws));
    assert_eq!(Some("9 substitutions on 9 cells".to_owned()), last_message(&ws));
    assert_eq!((1, 1), at(&ws));
}

#[test]
fn test_substitute_percent_is_one_undo_step_and_undo_goes_to_the_first_change() {
    let mut ws = grid_workspace();
    ws.book.update_cell(&Address { sheet: 0, row: 1, col: 1 }, "x").expect("set");
    script().chars("jj").run(&mut ws).expect("run");
    run_command(&mut ws, "%s/a/b/");
    assert_eq!(vec!["xbb", "bbb", "bbb"], grid_rows(&ws));
    assert_eq!((3, 1), at(&ws));
    script().char('u').run(&mut ws).expect("run");
    assert_eq!(vec!["xaa", "aaa", "aaa"], grid_rows(&ws));
    // B1 is the first cell the command changed.
    assert_eq!((1, 2), at(&ws));
}

#[test]
fn test_colon_in_range_select_prefills_the_selection_range() {
    let mut ws = grid_workspace();
    script().chars("vjl:").run(&mut ws).expect("run");
    assert_eq!(Some(&Modality::Command), ws.state.modality_stack.last());
    assert_eq!("'<,'>", command_text(&ws));
    script().chars("s/a/b/").enter().run(&mut ws).expect("run");
    assert_eq!(vec!["bba", "bba", "aaa"], grid_rows(&ws));
    assert_eq!(Some("4 substitutions on 4 cells".to_owned()), last_message(&ws));
    // As in vim, the command ends the selection.
    assert_eq!(Some(&Modality::Navigate), ws.state.modality_stack.last());
    assert_eq!((1, 1), at(&ws));
}

#[test]
fn test_colon_in_range_select_before_a_start_is_set_does_not_prefill() {
    let mut ws = grid_workspace();
    script().alt('r').char(':').run(&mut ws).expect("run");
    assert_eq!(Some(&Modality::Command), ws.state.modality_stack.last());
    assert_eq!("", command_text(&ws));
}

#[test]
fn test_substitute_selection_uses_the_last_selection_after_it_ends() {
    let mut ws = grid_workspace();
    // Select B2:C3, then leave Range Select with Esc.
    script().chars("jlvjl").esc().run(&mut ws).expect("run");
    assert_eq!(Some(&Modality::Navigate), ws.state.modality_stack.last());
    run_command(&mut ws, "'<,'>s/a/b/");
    assert_eq!(vec!["aaa", "abb", "abb"], grid_rows(&ws));
    // A later selection replaces it.
    script().chars("hvj").char(' ').run(&mut ws).expect("run");
    run_command(&mut ws, "'<,'>s/a/c/");
    assert_eq!(vec!["aaa", "cbb", "cbb"], grid_rows(&ws));
}

#[test]
fn test_substitute_selection_is_one_undo_step() {
    let mut ws = grid_workspace();
    script().chars("vjjl:").run(&mut ws).expect("run");
    script().chars("s/a/b/").enter().run(&mut ws).expect("run");
    assert_eq!(vec!["bba", "bba", "bba"], grid_rows(&ws));
    script().char('u').run(&mut ws).expect("run");
    assert_eq!(vec!["aaa", "aaa", "aaa"], grid_rows(&ws));
}

#[test]
fn test_substitute_selection_without_one_is_e20() {
    let mut ws = grid_workspace();
    run_command(&mut ws, "'<,'>s/a/b/");
    assert_eq!(Some("E20: Mark not set".to_owned()), last_message(&ws));
    assert_eq!(ws.state.message.as_ref().map(|m| m.level), Some(super::MessageLevel::Error));
    assert_eq!(vec!["aaa", "aaa", "aaa"], grid_rows(&ws));
}

#[test]
fn test_substitute_selection_on_another_sheet_is_e20() {
    let mut ws = grid_workspace();
    script().chars("vj").esc().run(&mut ws).expect("run");
    run_command(&mut ws, "new-sheet other");
    run_command(&mut ws, "select-sheet other");
    assert_eq!(1, ws.book.location.sheet);
    run_command(&mut ws, "'<,'>s/a/b/");
    assert_eq!(Some("E20: Mark not set".to_owned()), last_message(&ws));
    assert_eq!(vec!["aaa", "aaa", "aaa"], grid_rows(&ws));
}

#[test]
fn test_substitute_selection_with_an_array_formula_changes_nothing() {
    let mut book = book::Book::from_csv_reader("3,\n1,\n".as_bytes(), None).expect("csv");
    book.update_cell(&Address { sheet: 0, row: 1, col: 2 }, "=SORT(A1:A2)").expect("set");
    book.evaluate();
    let mut ws = Workspace::new(book, std::path::PathBuf::from("arr.sui"));
    script().chars("vjl:").run(&mut ws).expect("run");
    script().chars("s/[0-9A-Z]/x/").enter().run(&mut ws).expect("run");
    assert_eq!(Some("Cannot substitute in an array formula.".to_owned()), last_message(&ws));
    assert_eq!(cell_text(&ws, 1, 1), "3");
    assert_eq!(cell_text(&ws, 1, 2), "=SORT(A1:A2)");
}

#[test]
fn test_substitute_percent_skips_the_hidden_part_of_a_merge() {
    let mut model = ironcalc::base::Model::new_empty("test", "en", "America/New_York", "en")
        .expect("model");
    model.workbook.worksheets[0].merge_cells.push("A1:B1".to_string());
    let mut book = book::Book::from_model(model);
    for col in 1..=3 {
        book.update_cell(&Address { sheet: 0, row: 1, col }, "a").expect("set");
    }
    let mut ws = Workspace::new(book, std::path::PathBuf::from("merge.sui"));
    run_command(&mut ws, "%s/a/b/");
    assert_eq!(Some("2 substitutions on 2 cells".to_owned()), last_message(&ws));
    assert_eq!(cell_text(&ws, 1, 1), "b");
    assert_eq!(cell_text(&ws, 1, 2), "a");
    assert_eq!(cell_text(&ws, 1, 3), "b");
}

#[test]
fn test_substitute_refreshes_the_current_cell_contents_box() {
    let mut ws = sub_workspace();
    run_command(&mut ws, "%s/a/o/g");
    assert_eq!(ws.text_area.lines().join("\n"), "bonono");
}

// ---------------------------------------------------------------------------
// Replace: the c flag
// ---------------------------------------------------------------------------

fn confirm_prompt(ws: &Workspace) -> Option<String> {
    ws.state.confirm.as_ref().map(|c| c.prompt.clone())
}

#[test]
fn test_confirm_opens_the_prompt_on_the_first_match() {
    let mut ws = grid_workspace();
    script().chars("jl").run(&mut ws).expect("run");
    run_command(&mut ws, "%s/a/b/c");
    assert_eq!(Some(&Modality::Confirm), ws.state.modality_stack.last());
    assert_eq!(Some("replace with b (y/n/a/q/l)?".to_owned()), confirm_prompt(&ws));
    // Row-major from the top of the sheet, not from the cursor.
    assert_eq!((1, 1), at(&ws));
    // The match is highlighted with `:s`'s pattern, even with nohlsearch.
    run_confirm_keys_then(&mut ws, "q");
    run_command(&mut ws, "set nohlsearch");
    run_command(&mut ws, "%s/a/b/c");
    assert_eq!(Some("a".to_owned()), highlighted(&mut ws));
    run_confirm_keys_then(&mut ws, "q");
    assert_eq!(None, highlighted(&mut ws));
}

fn run_confirm_keys_then(ws: &mut Workspace, keys: &str) {
    script().chars(keys).run(ws).expect("run");
}

#[test]
fn test_confirm_y_and_n_walk_the_matches() {
    let mut ws = grid_workspace();
    run_command(&mut ws, "%s/a/b/c");
    script().chars("yn").run(&mut ws).expect("run");
    assert_eq!((1, 3), at(&ws));
    assert_eq!(vec!["baa", "aaa", "aaa"], grid_rows(&ws));
    script().chars("nnny").run(&mut ws).expect("run");
    assert_eq!((3, 1), at(&ws));
    assert_eq!(vec!["baa", "aab", "aaa"], grid_rows(&ws));
    script().chars("nn").run(&mut ws).expect("run");
    assert_eq!(Some(&Modality::Confirm), ws.state.modality_stack.last());
    script().char('n').run(&mut ws).expect("run");
    // Past the last match the prompt ends on the last changed cell.
    assert_eq!(Some(&Modality::Navigate), ws.state.modality_stack.last());
    assert_eq!((2, 3), at(&ws));
    assert_eq!(Some("2 substitutions on 2 cells".to_owned()), last_message(&ws));
    assert_eq!(ws.text_area.lines().join("\n"), "b");
}

#[test]
fn test_confirm_a_replaces_this_and_the_rest() {
    let mut ws = grid_workspace();
    run_command(&mut ws, "%s/a/b/c");
    script().chars("nnna").run(&mut ws).expect("run");
    assert_eq!(vec!["aaa", "bbb", "bbb"], grid_rows(&ws));
    assert_eq!(Some(&Modality::Navigate), ws.state.modality_stack.last());
    assert_eq!((3, 3), at(&ws));
    assert_eq!(Some("6 substitutions on 6 cells".to_owned()), last_message(&ws));
}

#[test]
fn test_confirm_l_replaces_this_and_stops() {
    let mut ws = grid_workspace();
    run_command(&mut ws, "%s/a/b/c");
    script().chars("nl").run(&mut ws).expect("run");
    assert_eq!(vec!["aba", "aaa", "aaa"], grid_rows(&ws));
    assert_eq!(Some(&Modality::Navigate), ws.state.modality_stack.last());
    assert_eq!((1, 2), at(&ws));
}

#[test]
fn test_confirm_q_and_esc_stop_and_go_back_when_nothing_changed() {
    for stop in ["q", "esc"] {
        let mut ws = grid_workspace();
        script().chars("jjl").run(&mut ws).expect("run");
        run_command(&mut ws, "%s/a/b/c");
        let keys = script().chars("nn");
        let keys = if stop == "q" { keys.char('q') } else { keys.esc() };
        keys.run(&mut ws).expect("run");
        assert_eq!(Some(&Modality::Navigate), ws.state.modality_stack.last(), "{stop}");
        assert_eq!((3, 2), at(&ws), "{stop}");
        assert_eq!(vec!["aaa", "aaa", "aaa"], grid_rows(&ws), "{stop}");
        assert_eq!(None, last_message(&ws), "{stop}");
        // Nothing changed, so there is no undo step.
        script().char('u').run(&mut ws).expect("run");
        assert_eq!((3, 2), at(&ws), "{stop}");
    }
}

#[test]
fn test_confirm_q_after_a_change_goes_to_the_last_change() {
    let mut ws = grid_workspace();
    run_command(&mut ws, "%s/a/b/c");
    script().chars("nynnq").run(&mut ws).expect("run");
    assert_eq!((1, 2), at(&ws));
    assert_eq!(vec!["aba", "aaa", "aaa"], grid_rows(&ws));
}

#[test]
fn test_confirm_other_keys_are_ignored() {
    let mut ws = grid_workspace();
    run_command(&mut ws, "%s/a/b/c");
    script().chars("xjk:").run(&mut ws).expect("run");
    assert_eq!(Some(&Modality::Confirm), ws.state.modality_stack.last());
    assert_eq!((1, 1), at(&ws));
    assert_eq!(vec!["aaa", "aaa", "aaa"], grid_rows(&ws));
}

#[test]
fn test_confirm_session_is_one_undo_step() {
    let mut ws = grid_workspace();
    script().chars("jj").run(&mut ws).expect("run");
    run_command(&mut ws, "%s/a/b/c");
    script().chars("nynyyq").run(&mut ws).expect("run");
    assert_eq!(vec!["aba", "bba", "aaa"], grid_rows(&ws));
    script().char('u').run(&mut ws).expect("run");
    assert_eq!(vec!["aaa", "aaa", "aaa"], grid_rows(&ws));
    // Undo goes to the first changed cell.
    assert_eq!((1, 2), at(&ws));
    script().char('r').run(&mut ws).expect("run");
    assert_eq!(vec!["aba", "bba", "aaa"], grid_rows(&ws));
}

#[test]
fn test_confirm_with_g_asks_about_each_match_in_a_cell() {
    let book = book::Book::from_csv_reader("aaa,a\n".as_bytes(), None).expect("csv");
    let mut ws = Workspace::new(book, std::path::PathBuf::from("g.sui"));
    run_command(&mut ws, "%s/a/b/gc");
    script().chars("yny").run(&mut ws).expect("run");
    assert_eq!(Some(&Modality::Confirm), ws.state.modality_stack.last());
    assert_eq!((1, 2), at(&ws));
    script().char('n').run(&mut ws).expect("run");
    assert_eq!(cell_text(&ws, 1, 1), "bab");
    assert_eq!(cell_text(&ws, 1, 2), "a");
    assert_eq!(Some("2 substitutions on 1 cell".to_owned()), last_message(&ws));
    assert_eq!((1, 1), at(&ws));
}

#[test]
fn test_confirm_does_not_match_the_replacement_again() {
    let book = book::Book::from_csv_reader("aa\n".as_bytes(), None).expect("csv");
    let mut ws = Workspace::new(book, std::path::PathBuf::from("loop.sui"));
    run_command(&mut ws, "s/a/aa/gc");
    script().chars("yy").run(&mut ws).expect("run");
    assert_eq!(Some(&Modality::Navigate), ws.state.modality_stack.last());
    assert_eq!(cell_text(&ws, 1, 1), "aaaa");
}

#[test]
fn test_confirm_with_the_selection_scope() {
    let mut ws = grid_workspace();
    script().chars("jlvjl:").run(&mut ws).expect("run");
    script().chars("s/a/b/c").enter().run(&mut ws).expect("run");
    assert_eq!(Some(&Modality::Confirm), ws.state.modality_stack.last());
    assert_eq!((2, 2), at(&ws));
    script().chars("yna").run(&mut ws).expect("run");
    assert_eq!(vec!["aaa", "aba", "abb"], grid_rows(&ws));
    assert_eq!(Some(&Modality::Navigate), ws.state.modality_stack.last());
    assert_eq!((3, 3), at(&ws));
}

#[test]
fn test_confirm_with_no_match_is_e486() {
    let mut ws = grid_workspace();
    run_command(&mut ws, "%s/z/b/c");
    assert_eq!(Some(&Modality::Navigate), ws.state.modality_stack.last());
    assert_eq!(Some("E486: Pattern not found: z".to_owned()), last_message(&ws));
    assert!(ws.state.confirm.is_none());
    run_command(&mut ws, "%s/z/b/ce");
    assert_eq!(Some(&Modality::Navigate), ws.state.modality_stack.last());
}

#[test]
fn test_confirm_with_n_only_counts() {
    let mut ws = grid_workspace();
    run_command(&mut ws, "%s/a/b/cn");
    assert_eq!(Some(&Modality::Navigate), ws.state.modality_stack.last());
    assert_eq!(Some("9 matches on 9 cells".to_owned()), last_message(&ws));
}

#[test]
fn test_confirm_prompt_is_drawn_and_scrolls_to_the_match() {
    use ratatui::{backend::TestBackend, Terminal};
    let mut book = book::Book::from_csv_reader("x\n".as_bytes(), None).expect("csv");
    book.update_cell(&Address { sheet: 0, row: 40, col: 1 }, "far").expect("set");
    book.evaluate();
    let mut ws = Workspace::new(book, std::path::PathBuf::from("far.sui"));
    run_command(&mut ws, "%s/far/near/c");
    assert_eq!((40, 1), at(&ws));
    let mut term = Terminal::new(TestBackend::new(60, 16)).expect("terminal");
    term.draw(|f| super::render::draw(f, &mut ws)).expect("draw");
    let buf = term.backend().buffer().clone();
    let rows: Vec<String> = (0..buf.area.height)
        .map(|y| (0..buf.area.width).map(|x| buf[(x, y)].symbol().to_owned()).collect())
        .collect();
    assert!(rows.iter().any(|r| r.contains("replace with near (y/n/a/q/l)?")), "{rows:#?}");
    assert!(rows.iter().any(|r| r.contains("far")), "{rows:#?}");
    assert!(rows.last().unwrap().contains("confirm"), "{rows:#?}");
}

// ---------------------------------------------------------------------------
// Freeze
// ---------------------------------------------------------------------------

fn cmd_line(ws: &mut Workspace, text: &str) {
    run(ws, script().char(':').chars(text).enter());
}

#[test]
fn test_cmd_freeze_parses_rows_and_optional_columns() {
    assert_eq!(parse("freeze 1"), Ok(Some(Cmd::Freeze { rows: 1, cols: 0 })));
    assert_eq!(parse("freeze  2   3"), Ok(Some(Cmd::Freeze { rows: 2, cols: 3 })));
    assert_eq!(parse("freeze 0 0"), Ok(Some(Cmd::Freeze { rows: 0, cols: 0 })));
    assert_eq!(parse("unfreeze"), Ok(Some(Cmd::Unfreeze)));
}

#[test]
fn test_cmd_freeze_rejects_bad_counts() {
    for input in [
        "freeze -1",
        "freeze 1.5",
        "freeze x",
        "freeze 1 x",
        "freeze 1 2 3",
        "freeze1",
        "freeze 99999999999999999999999",
        "unfreeze 1",
    ] {
        assert!(parse(input).is_err(), "{input:?} should be an error");
    }
}

#[test]
fn test_freeze_sets_clears_and_reports() {
    let mut ws = new_workspace();
    cmd_line(&mut ws, "freeze 1");
    assert_eq!(1, ws.book.frozen_rows(0));
    assert_eq!(Some("Froze rows 1-1, columns none".to_string()), message_text(&ws));
    cmd_line(&mut ws, "unfreeze");
    assert_eq!(0, ws.book.frozen_rows(0));
    cmd_line(&mut ws, "freeze 3");
    cmd_line(&mut ws, "freeze 0 0");
    assert_eq!(0, ws.book.frozen_rows(0));
}

#[test]
fn test_freeze_errors_leave_the_freeze_alone() {
    let mut ws = new_workspace();
    cmd_line(&mut ws, "freeze 2");
    for bad in ["freeze -1", "freeze 1.5", "freeze x", "freeze 2000000"] {
        cmd_line(&mut ws, bad);
        run(&mut ws, script().event(crossterm::event::Event::Key(KeyEvent::new(
            KeyCode::Esc,
            KeyModifiers::NONE,
        ))));
        assert_eq!(2, ws.book.frozen_rows(0), "{bad}");
    }
}

#[test]
fn test_freeze_is_per_sheet() {
    let mut ws = new_workspace();
    cmd_line(&mut ws, "new-sheet two");
    cmd_line(&mut ws, "select-sheet two");
    cmd_line(&mut ws, "freeze 2");
    assert_eq!(2, ws.book.frozen_rows(ws.book.location.sheet));
    assert_eq!(0, ws.book.frozen_rows(0));
}

#[test]
fn test_freeze_marks_dirty_and_undo_leaves_it() {
    let mut ws = new_workspace();
    run(&mut ws, script().chars("efoo").enter());
    cmd_line(&mut ws, "freeze 1");
    assert!(ws.book.dirty);
    cmd_line(&mut ws, "undo");
    assert_eq!(cell(&ws, 1, 1), "");
    assert_eq!(1, ws.book.frozen_rows(0), "undo must not touch the freeze");
    assert!(ws.book.dirty, "still unsaved");
    cmd_line(&mut ws, "unfreeze");
    assert!(ws.book.dirty);
}

#[test]
fn test_cmd_bare_freeze_parses() {
    assert_eq!(parse("freeze"), Ok(Some(Cmd::FreezeAtCursor)));
    assert_eq!(parse("freeze  "), Ok(Some(Cmd::FreezeAtCursor)));
}

#[test]
fn test_bare_freeze_pins_above_and_left_of_cursor() {
    let mut ws = new_workspace();
    run(&mut ws, script().chars("lljjjj"));
    cmd_line(&mut ws, "freeze");
    assert_eq!(4, ws.book.frozen_rows(0));
    assert_eq!(2, ws.book.frozen_columns(0));
    assert_eq!(Some("Froze rows 1-4, columns A-B".to_string()), message_text(&ws));
    assert!(ws.book.dirty);
}

#[test]
fn test_bare_freeze_counts_from_origin_when_scrolled() {
    let mut ws = new_workspace();
    run(&mut ws, script().chars("lll").chars(&"j".repeat(60)));
    cmd_line(&mut ws, "freeze");
    assert_eq!(60, ws.book.frozen_rows(0));
    assert_eq!(3, ws.book.frozen_columns(0));
}

#[test]
fn test_bare_freeze_on_a1_changes_nothing() {
    let mut ws = new_workspace();
    cmd_line(&mut ws, "freeze");
    assert!(!ws.book.dirty);
    assert_eq!(0, ws.book.frozen_rows(0));
    assert_eq!(
        Some("Nothing to freeze above or left of A1".to_string()),
        message_text(&ws)
    );
}

#[test]
fn test_bare_freeze_on_a1_keeps_an_existing_freeze() {
    let mut ws = new_workspace();
    cmd_line(&mut ws, "freeze 2");
    assert_eq!(2, ws.book.frozen_rows(0));
    cmd_line(&mut ws, "freeze");
    assert_eq!(2, ws.book.frozen_rows(0));
}

#[test]
fn test_bare_freeze_same_as_existing_is_not_dirty() {
    let mut ws = new_workspace();
    run(&mut ws, script().chars("j"));
    cmd_line(&mut ws, "freeze");
    ws.book.dirty = false;
    cmd_line(&mut ws, "freeze");
    assert!(!ws.book.dirty);
}

fn draw_lines(ws: &mut Workspace, w: u16, h: u16) -> Vec<String> {
    use ratatui::{backend::TestBackend, Terminal};
    let mut term = Terminal::new(TestBackend::new(w, h)).expect("terminal");
    term.draw(|f| super::render::draw(f, ws)).expect("draw");
    let buf = term.backend().buffer().clone();
    (0..h)
        .map(|y| (0..w).map(|x| buf[(x, y)].symbol().to_owned()).collect())
        .collect()
}

#[test]
fn test_freeze_larger_than_screen_clips_rows_and_reports() {
    let mut ws = new_workspace();
    cmd_line(&mut ws, "freeze 30");
    let rows = draw_lines(&mut ws, 60, 24);
    assert_eq!(Some("Freeze clipped to fit the screen".to_string()), message_text(&ws));
    assert_eq!(30, ws.book.frozen_rows(0), "stored count stays as typed");
    // The cursor can still move and scroll.
    run(&mut ws, script().chars(&"j".repeat(40)));
    let rows2 = draw_lines(&mut ws, 60, 24);
    assert!(rows2.iter().any(|r| r.contains("│41")), "{rows2:#?}\n{rows:#?}");
    assert_eq!(30, ws.book.frozen_rows(0));
}

#[test]
fn test_freeze_larger_than_screen_clips_columns() {
    let mut ws = new_workspace();
    cmd_line(&mut ws, "freeze 0 20");
    draw_lines(&mut ws, 40, 24);
    assert_eq!(Some("Freeze clipped to fit the screen".to_string()), message_text(&ws));
    assert_eq!(20, ws.book.frozen_columns(0));
    run(&mut ws, script().chars(&"l".repeat(25)));
    let rows = draw_lines(&mut ws, 40, 24);
    assert!(rows.iter().any(|r| r.contains('Z')), "{rows:#?}");
}

#[test]
fn test_freeze_clip_goes_away_when_the_terminal_grows() {
    let mut ws = new_workspace();
    cmd_line(&mut ws, "freeze 10");
    draw_lines(&mut ws, 60, 14);
    assert!(ws.state.viewport_state.freeze_clipped);
    let rows = draw_lines(&mut ws, 60, 30);
    assert!(!ws.state.viewport_state.freeze_clipped);
    assert!(rows.iter().any(|r| r.contains("│10")), "{rows:#?}");
}

#[test]
fn test_freeze_clip_saves_the_typed_count() {
    let mut ws = new_workspace();
    cmd_line(&mut ws, "freeze 30 2");
    draw_lines(&mut ws, 60, 14);
    let dir = std::env::temp_dir().join(format!("sheetsui-clip-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("clip.xlsx");
    ws.book.save_to_xlsx(path.to_str().unwrap()).expect("save");
    let loaded = book::Book::new_from_xlsx(path.to_str().unwrap()).expect("load");
    assert_eq!(30, loaded.frozen_rows(0));
    assert_eq!(2, loaded.frozen_columns(0));
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn test_delete_rows_cmd_forms() {
    assert_eq!(Cmd::DeleteRows(1), parse("delete-rows").unwrap().unwrap());
    assert_eq!(Cmd::DeleteRows(3), parse("delete-rows 3").unwrap().unwrap());
    assert_eq!(Cmd::DeleteRows(2), parse("dr 2").unwrap().unwrap());
    assert!(parse("delete-rows x").is_err());
    assert!(parse("delete-rows 1.5").is_err());
    assert!(parse("delete-rows -1").is_err());
    assert!(parse("delete-rowsx").is_err());
}

#[test]
fn test_delete_rows_command_deletes_from_the_cursor_row() {
    let mut ws = sort_workspace(&[&["1"], &["2"], &["3"], &["4"]]);
    script().char('j').run(&mut ws).expect("move");
    run_command(&mut ws, "delete-rows 2");
    assert_eq!(sheet_column(&ws, 1, 1..=3), vec!["1", "4", ""]);
    assert_eq!(2, ws.book.location.row);
    assert!(ws.book.undo().expect("undo"));
    assert_eq!(sheet_column(&ws, 1, 1..=4), vec!["1", "2", "3", "4"]);
}

#[test]
fn test_delete_rows_command_errors_are_a_dialog_and_change_nothing() {
    let mut ws = sort_workspace(&[&["1"], &["2"]]);
    run_command(&mut ws, "delete-rows 0");
    assert_eq!(Some(&Modality::Dialog), ws.state.modality_stack.last());
    assert_eq!(sheet_column(&ws, 1, 1..=2), vec!["1", "2"]);
    // The last undo step is still the setup edit, not a delete.
    assert!(ws.book.undo().expect("undo"));
    assert_eq!(sheet_column(&ws, 1, 1..=2), vec!["1", ""]);
}

#[test]
fn test_delete_rows_command_in_range_select_ignores_the_count() {
    let mut ws = sort_workspace(&[&["1"], &["2"], &["3"], &["4"], &["5"]]);
    script().char('j').char('v').char('j').run(&mut ws).expect("select");
    run_command(&mut ws, "delete-rows 9");
    assert_eq!(sheet_column(&ws, 1, 1..=3), vec!["1", "4", "5"]);
    assert_eq!(Some(&Modality::Navigate), ws.state.modality_stack.last());
    assert_eq!(2, ws.book.location.row);
}

#[test]
fn test_delete_cols_cmd_forms() {
    assert_eq!(Cmd::DeleteColumns(1), parse("delete-cols").unwrap().unwrap());
    assert_eq!(Cmd::DeleteColumns(3), parse("delete-cols 3").unwrap().unwrap());
    assert_eq!(Cmd::DeleteColumns(2), parse("dc 2").unwrap().unwrap());
    assert!(parse("delete-cols x").is_err());
    assert!(parse("delete-cols 1.5").is_err());
    assert!(parse("delete-cols -1").is_err());
    assert!(parse("delete-colsx").is_err());
}

#[test]
fn test_delete_cols_command_deletes_from_the_cursor_column() {
    let mut ws = sort_workspace(&[&["1", "2", "3", "4"]]);
    script().char('l').run(&mut ws).expect("move");
    run_command(&mut ws, "delete-cols 2");
    assert_eq!(vec!["1", "4", ""], (1..=3).map(|c| ws.book.get_cell_addr_rendered(&Address { row: 1, col: c, sheet: 0 }).unwrap()).collect::<Vec<_>>());
    assert_eq!(2, ws.book.location.col);
    assert!(ws.book.undo().expect("undo"));
    assert_eq!("3", ws.book.get_cell_addr_rendered(&Address { row: 1, col: 3, sheet: 0 }).unwrap());
}

#[test]
fn test_delete_cols_command_errors_are_a_dialog_and_change_nothing() {
    let mut ws = sort_workspace(&[&["1", "2"]]);
    run_command(&mut ws, "delete-cols 0");
    assert_eq!(Some(&Modality::Dialog), ws.state.modality_stack.last());
    assert_eq!("2", ws.book.get_cell_addr_rendered(&Address { row: 1, col: 2, sheet: 0 }).unwrap());
}

#[test]
fn test_delete_cols_command_in_range_select_ignores_the_count() {
    let mut ws = sort_workspace(&[&["1", "2", "3", "4", "5"]]);
    script().char('l').char('v').char('l').run(&mut ws).expect("select");
    run_command(&mut ws, "delete-cols 9");
    assert_eq!("4", ws.book.get_cell_addr_rendered(&Address { row: 1, col: 2, sheet: 0 }).unwrap());
    assert_eq!(Some(&Modality::Navigate), ws.state.modality_stack.last());
    assert_eq!(2, ws.book.location.col);
}

#[test]
fn test_dd_deletes_a_row_and_3dd_three() {
    let mut ws = sort_workspace(&[&["1"], &["2"], &["3"], &["4"], &["5"]]);
    script().char('j').chars("dd").run(&mut ws).expect("dd");
    assert_eq!(sheet_column(&ws, 1, 1..=4), vec!["1", "3", "4", "5"]);
    assert_eq!(2, ws.book.location.row);
    script().chars("3dd").run(&mut ws).expect("3dd");
    assert_eq!(sheet_column(&ws, 1, 1..=2), vec!["1", ""]);
    assert!(ws.state.numeric_prefix.is_empty());
}

#[test]
fn test_u_after_dd_restores_the_row_in_one_step() {
    let mut ws = sort_workspace(&[&["1"], &["2"], &["3"]]);
    script().char('j').chars("2dd").char('u').run(&mut ws).expect("keys");
    assert_eq!(sheet_column(&ws, 1, 1..=3), vec!["1", "2", "3"]);
}

#[test]
fn test_dc_and_2dc_delete_columns() {
    let mut ws = sort_workspace(&[&["a", "b", "c", "d"]]);
    script().char('l').chars("dc").run(&mut ws).expect("dc");
    assert_eq!("a", sheet_column(&ws, 1, 1..=1)[0]);
    assert_eq!("c", sheet_column(&ws, 2, 1..=1)[0]);
    script().chars("2dc").run(&mut ws).expect("2dc");
    assert_eq!("", sheet_column(&ws, 2, 1..=1)[0]);
    assert_eq!("a", sheet_column(&ws, 1, 1..=1)[0]);
}

#[test]
fn test_a_lone_d_waits_and_esc_cancels_it() {
    let mut ws = sort_workspace(&[&["1"], &["2"]]);
    script().char('d').run(&mut ws).expect("d");
    assert_eq!(1, ws.state.pending_keys.len());
    assert_eq!(sheet_column(&ws, 1, 1..=2), vec!["1", "2"]);
    script().esc().run(&mut ws).expect("esc");
    assert!(ws.state.pending_keys.is_empty());
    assert_eq!(sheet_column(&ws, 1, 1..=2), vec!["1", "2"]);
}

#[test]
fn test_range_select_d_deletes_rows_and_big_d_columns() {
    let mut ws = sort_workspace(&[&["1", "a"], &["2", "b"], &["3", "c"], &["4", "d"]]);
    script().char('j').char('v').char('j').char('2').char('d').run(&mut ws).expect("d");
    assert_eq!(sheet_column(&ws, 1, 1..=2), vec!["1", "4"]);
    assert_eq!(Some(&Modality::Navigate), ws.state.modality_stack.last());
    script().char('v').char('D').run(&mut ws).expect("D");
    assert_eq!(sheet_column(&ws, 1, 1..=2), vec!["a", "d"]);
    assert_eq!(Some(&Modality::Navigate), ws.state.modality_stack.last());
}

#[test]
fn test_dd_can_be_remapped_and_unmapped() {
    let mut ws = sort_workspace(&[&["1"], &["2"], &["3"]]);
    type_command(&mut ws, "nnoremap X dd");
    script().char('X').run(&mut ws).expect("X");
    assert_eq!(sheet_column(&ws, 1, 1..=2), vec!["2", "3"]);
    type_command(&mut ws, "nunmap dd");
    script().chars("dd").run(&mut ws).expect("dd");
    assert_eq!(sheet_column(&ws, 1, 1..=2), vec!["2", "3"]);
}

#[test]
fn test_help_lists_the_delete_keys() {
    use crate::ui::keys::{builtin, help, map::MapMode};
    let ws = new_workspace();
    let nav = help::group_text(builtin::navigation(), ws.state.keymap.table(MapMode::Navigate), "modify")
        .expect("modify group");
    assert!(nav.contains("`dd`: delete the current row"), "{nav}");
    assert!(nav.contains("`dc`: delete the current column"), "{nav}");
    let range = help::group_text(builtin::range_select(), ws.state.keymap.table(MapMode::RangeSelect), "selection")
        .expect("selection group");
    assert!(range.contains("`d`: delete the rows the range touches"), "{range}");
    assert!(range.contains("`D`: delete the columns the range touches"), "{range}");
}

#[test]
fn test_fill_down_cmd_forms() {
    assert_eq!(Cmd::FillDown(1), parse("fill-down").unwrap().unwrap());
    assert_eq!(Cmd::FillDown(5), parse("fill-down 5").unwrap().unwrap());
    assert!(parse("fill-down x").is_err());
    assert!(parse("fill-down -1").is_err());
    assert!(parse("fill-downx").is_err());
}

#[test]
fn test_ctrl_d_fills_the_cell_above_down() {
    let mut ws = sort_workspace(&[&["1", "=A1+1"], &["2"]]);
    script().char('j').char('l').run(&mut ws).expect("move");
    script().ctrl('d').run(&mut ws).expect("fill");
    assert_eq!(
        "=A2+1",
        ws.book.get_cell_addr_contents(&Address { sheet: 0, row: 2, col: 2 }).unwrap()
    );
    assert!(ws.book.dirty);
}

#[test]
fn test_prefix_ctrl_d_and_fill_down_cmd_fill_count_cells_and_cursor_stays() {
    let mut ws = sort_workspace(&[&["a"], &[""]]);
    script().char('j').run(&mut ws).expect("move");
    script().char('3').ctrl('d').run(&mut ws).expect("fill");
    assert_eq!(sheet_column(&ws, 1, 1..=5), vec!["a", "a", "a", "a", ""]);
    assert_eq!(2, ws.book.location.row);
    assert!(ws.book.undo().expect("undo"));
    assert_eq!(sheet_column(&ws, 1, 1..=4), vec!["a", "", "", ""]);

    run_command(&mut ws, "fill-down 2");
    assert_eq!(sheet_column(&ws, 1, 1..=4), vec!["a", "a", "a", ""]);
    run_command(&mut ws, "fill-down");
    assert_eq!(2, ws.book.location.row);
}

#[test]
fn test_fill_down_on_row_one_is_a_dialog_and_changes_nothing() {
    let mut ws = sort_workspace(&[&["a"]]);
    ws.book.dirty = false;
    script().ctrl('d').run(&mut ws).expect("fill");
    assert_eq!(Some(&Modality::Dialog), ws.state.modality_stack.last());
    assert!(!ws.book.dirty);
}

#[test]
fn test_fill_right_cmd_forms() {
    assert_eq!(Cmd::FillRight(1), parse("fill-right").unwrap().unwrap());
    assert_eq!(Cmd::FillRight(5), parse("fill-right 5").unwrap().unwrap());
    assert!(parse("fill-right x").is_err());
    assert!(parse("fill-rightx").is_err());
}

#[test]
fn test_ctrl_r_fills_the_cell_to_the_left_right() {
    let mut ws = sort_workspace(&[&["1"], &["=A1+1"]]);
    script().char('j').char('l').run(&mut ws).expect("move");
    script().ctrl('r').run(&mut ws).expect("fill");
    assert_eq!(
        "=B1+1",
        ws.book.get_cell_addr_contents(&Address { sheet: 0, row: 2, col: 2 }).unwrap()
    );
    assert!(ws.book.dirty);
    assert_eq!(Some(&Modality::Navigate), ws.state.modality_stack.last());
}

#[test]
fn test_prefix_ctrl_r_and_fill_right_cmd_fill_count_cells_and_cursor_stays() {
    let mut ws = sort_workspace(&[&["a", "", "", "", ""]]);
    script().char('l').run(&mut ws).expect("move");
    script().char('3').ctrl('r').run(&mut ws).expect("fill");
    let row = |ws: &Workspace| -> Vec<String> {
        (1..=5)
            .map(|c| ws.book.get_cell_addr_contents(&Address { sheet: 0, row: 1, col: c }).unwrap())
            .collect()
    };
    assert_eq!(row(&ws), vec!["a", "a", "a", "a", ""]);
    assert_eq!(2, ws.book.location.col);
    assert!(ws.book.undo().expect("undo"));
    assert_eq!(row(&ws), vec!["a", "", "", "", ""]);
    run_command(&mut ws, "fill-right 2");
    assert_eq!(row(&ws), vec!["a", "a", "a", "", ""]);
}

#[test]
fn test_fill_right_on_column_a_is_a_dialog_and_changes_nothing() {
    let mut ws = sort_workspace(&[&["a"]]);
    ws.book.dirty = false;
    script().ctrl('r').run(&mut ws).expect("fill");
    assert_eq!(Some(&Modality::Dialog), ws.state.modality_stack.last());
    assert!(!ws.book.dirty);
}

#[test]
fn test_alt_r_enters_range_select_and_ctrl_r_does_not() {
    let mut ws = new_workspace();
    script().alt('r').run(&mut ws).expect("run");
    assert_eq!(Some(&Modality::RangeSelect), ws.state.modality_stack.last());
    let mut ws = new_workspace();
    script().ctrl('r').run(&mut ws).expect("run");
    assert_ne!(Some(&Modality::RangeSelect), ws.state.modality_stack.last());
}

// -------------------------------------------------------------------------
// Fill a selection

fn grid(ws: &Workspace, rows: std::ops::RangeInclusive<usize>, cols: std::ops::RangeInclusive<usize>) -> Vec<Vec<String>> {
    rows.map(|row| {
        cols.clone()
            .map(|col| ws.book.get_cell_addr_contents(&Address { sheet: 0, row, col }).unwrap())
            .collect()
    })
    .collect()
}

/// Select B2:D5 in range select mode, started from Navigate.
fn select_b2_d5(ws: &mut Workspace) {
    script()
        .char('j').char('l')
        .alt('r').char(' ')
        .chars("jjjll")
        .run(ws)
        .expect("select");
}

fn block_workspace<'a>() -> Workspace<'a> {
    sort_workspace(&[&["", "", "", ""], &["", "=A2+1", "x", "y"], &["", "", "", ""], &["", "", "", ""], &["", "", "", ""]])
}

#[test]
fn test_ctrl_d_in_range_select_fills_each_column_from_its_top_cell() {
    let mut ws = block_workspace();
    select_b2_d5(&mut ws);
    script().ctrl('d').run(&mut ws).expect("fill");
    assert_eq!(
        grid(&ws, 2..=5, 2..=4),
        vec![
            vec!["=A2+1", "x", "y"],
            vec!["=A3+1", "x", "y"],
            vec!["=A4+1", "x", "y"],
            vec!["=A5+1", "x", "y"],
        ]
    );
    assert_eq!(Some(&Modality::RangeSelect), ws.state.modality_stack.last());
    assert_eq!(
        Some((Address { sheet: 0, row: 2, col: 2 }, Address { sheet: 0, row: 5, col: 4 })),
        ws.state.range_select.get_range()
    );
    // One undo step.
    assert!(ws.book.undo().expect("undo"));
    assert_eq!(grid(&ws, 3..=5, 2..=4), vec![vec![""; 3]; 3]);
}

#[test]
fn test_ctrl_d_then_ctrl_r_and_back_stamp_the_top_left_cell_with_formats() {
    for order in [['d', 'r'], ['r', 'd']] {
        let mut ws = sort_workspace(&[&["", ""], &["", "7"]]);
        let area = ironcalc::base::expressions::types::Area { sheet: 0, row: 2, column: 2, width: 1, height: 1 };
        ws.book.set_cell_style(&[("font.b", "true")], &area).expect("bold");
        script().char('j').char('l').alt('r').char(' ').chars("jlll").run(&mut ws).expect("select");
        for k in order {
            script().ctrl(k).run(&mut ws).expect("fill");
        }
        for row in 2..=3 {
            for col in 2..=5 {
                let addr = Address { sheet: 0, row, col };
                assert_eq!("7", ws.book.get_cell_addr_contents(&addr).unwrap(), "{order:?} {row},{col}");
                assert_eq!(Some(true), ws.book.get_cell_style(&addr).map(|s| s.font.b));
            }
        }
        assert_eq!(Some(&Modality::RangeSelect), ws.state.modality_stack.last());
    }
}

#[test]
fn test_fill_one_row_or_column_selection_uses_the_neighbour() {
    let mut ws = sort_workspace(&[&["a", "b"], &["", ""]]);
    script().char('j').alt('r').char(' ').char('l').run(&mut ws).expect("select");
    script().ctrl('d').run(&mut ws).expect("fill");
    assert_eq!(grid(&ws, 2..=2, 1..=2), vec![vec!["a", "b"]]);

    let mut ws = sort_workspace(&[&["a", ""], &["b", ""]]);
    script().char('l').alt('r').char(' ').char('j').run(&mut ws).expect("select");
    script().ctrl('r').run(&mut ws).expect("fill");
    assert_eq!(grid(&ws, 1..=2, 1..=2), vec![vec!["a", "a"], vec!["b", "b"]]);
}

#[test]
fn test_range_select_fill_ignores_a_prefix() {
    let mut ws = block_workspace();
    select_b2_d5(&mut ws);
    script().char('2').ctrl('d').run(&mut ws).expect("fill");
    assert_eq!(sheet_column(&ws, 3, 2..=5), vec!["x"; 4]);
}

#[test]
fn test_fill_command_with_selection_prefix_uses_selection_and_ignores_count() {
    let mut ws = block_workspace();
    select_b2_d5(&mut ws);
    // `:` pre-fills `'<,'>`; the count is ignored under a selection.
    script().char(':').chars("fill-down 1").enter().run(&mut ws).expect("cmd");
    assert_eq!(sheet_column(&ws, 3, 2..=5), vec!["x"; 4]);
    assert_eq!(Some(&Modality::RangeSelect), ws.state.modality_stack.last());
    assert_eq!(Cmd::FillDown(5), parse("'<,'>fill-down 5").unwrap().unwrap());
}

#[test]
fn test_range_select_fill_errors_are_a_dialog_and_change_nothing() {
    let mut ws = sort_workspace(&[&["a"]]);
    ws.book.dirty = false;
    script().alt('r').char(' ').run(&mut ws).expect("select");
    script().ctrl('d').run(&mut ws).expect("fill");
    assert_eq!(Some(&Modality::Dialog), ws.state.modality_stack.last());
    assert!(!ws.book.dirty);
}

#[test]
fn test_x_in_range_select_still_stamps_the_start_cell() {
    let mut ws = sort_workspace(&[&["", "", ""], &["", "5", "9"]]);
    script().char('j').char('l').alt('r').char(' ').chars("jl").run(&mut ws).expect("select");
    script().char('x').run(&mut ws).expect("extend");
    assert_eq!(grid(&ws, 2..=3, 2..=3), vec![vec!["5", "5"], vec!["5", "5"]]);
    assert_eq!(Some(&Modality::Navigate), ws.state.modality_stack.last());
}

#[test]
fn test_fill_series_cmd_forms_and_usage_errors() {
    use crate::book::FillDirection::*;
    assert_eq!(Cmd::FillSeries(Down, 3), parse("fill-series down 3").unwrap().unwrap());
    assert_eq!(Cmd::FillSeries(Up, 1), parse("fill-series up 1").unwrap().unwrap());
    assert_eq!(Cmd::FillSeries(Left, 2), parse("'<,'>fill-series left 2").unwrap().unwrap());
    assert_eq!(Cmd::FillSeries(Right, 2), parse("fill-series right 2").unwrap().unwrap());
    for bad in ["fill-series", "fill-series down", "fill-series sideways 3", "fill-series down x", "fill-series down 0", "fill-series down 2 3", "fill-seriesx"] {
        let err = parse(bad).expect_err(bad);
        assert!(err.contains("Usage"), "{bad}: {err}");
    }
}

#[test]
fn test_fill_series_cmd_uses_cursor_or_selection_as_seeds() {
    let mut ws = sort_workspace(&[&["1", "5"], &["2", "7"], &["", ""], &["", ""], &["", ""]]);
    script().char('j').run(&mut ws).expect("move");
    run_command(&mut ws, "fill-series down 1");
    assert_eq!(grid(&ws, 1..=3, 1..=1), vec![vec!["1"], vec!["2"], vec!["2"]]);
    let mut ws = sort_workspace(&[&["1", "5"], &["2", "7"], &["", ""], &["", ""], &["", ""]]);
    script().alt('r').char(' ').chars("jl").run(&mut ws).expect("select");
    script().char(':').chars("fill-series down 2").enter().run(&mut ws).expect("cmd");
    assert_eq!(grid(&ws, 3..=4, 1..=2), vec![vec!["3", "9"], vec!["4", "11"]]);
    let mut ws = sort_workspace(&[&["1", "2"]]);
    run_command(&mut ws, "fill-series down");
    assert_eq!(Some(&Modality::Dialog), ws.state.modality_stack.last());
}

#[test]
fn test_config_flag_file_and_keys_flag_both_load() {
    let dir = temp_config_dir("config-flag");
    std::fs::write(dir.join("config"), "set ic\n").unwrap();
    let cfg = dir.join("mine");
    std::fs::write(&cfg, "set scs\nnnoremap a l\n").unwrap();
    let keys = dir.join("mykeys");
    std::fs::write(&keys, "nnoremap b h\n").unwrap();
    let mut ws = new_workspace();
    ws.init_config_from(Some(&cfg), Some(dir.clone()));
    ws.init_keys_from(Some(&keys), Some(dir.clone()));
    assert!(ws.state.options.smartcase());
    assert!(!ws.state.options.ignorecase());
    assert_eq!(2, ws.state.keymap.nav.mappings().len());
    assert_eq!(None, message_text(&ws));
}

#[test]
fn test_missing_config_flag_file_is_counted_as_a_warning() {
    let dir = temp_config_dir("config-flag-missing");
    let mut ws = new_workspace();
    ws.init_config_from(Some(&dir.join("nope")), Some(dir));
    assert_eq!(
        Some("1 warning: see :config".to_owned()),
        message_text(&ws)
    );
}

#[test]
fn test_parse_config_cmd() {
    assert_eq!(parse("config"), Ok(Some(Cmd::Config)));
    assert!(parse("config x").is_err());
    assert!(parse("configs").is_err());
}

#[test]
fn test_config_cmd_lists_warnings_from_all_three_sources() {
    let dir = temp_config_dir("config-cmd");
    std::fs::write(dir.join("config"), "write x\nset ic\n").unwrap();
    std::fs::write(dir.join("keys"), "nnoremap <Nope> j\n").unwrap();
    std::fs::write(dir.join("themes/bad.theme"), "style nope fg red\n").unwrap();
    let mut ws = new_workspace();
    ws.init_config_from(None, Some(dir.clone()));
    ws.init_theme_from(Some("bad"), Some(dir.clone()));
    ws.init_keys_from(None, Some(dir));
    assert_eq!(Some("3 warnings: see :config".to_owned()), message_text(&ws));
    run_command(&mut ws, "config");
    assert_eq!(&Modality::Dialog, ws.state.modality());
    let text = popup_text(&ws);
    assert!(text.contains("config:1:"), "{text}");
    assert!(text.contains("keys:1:"), "{text}");
    assert!(text.contains("bad:1:"), "{text}");
}

#[test]
fn test_config_cmd_says_so_when_there_are_no_warnings() {
    let mut ws = new_workspace();
    run_command(&mut ws, "config");
    assert_eq!(&Modality::Dialog, ws.state.modality());
    assert!(popup_text(&ws).contains("No warnings"));
}

#[test]
fn test_config_cmd_keeps_startup_theme_warnings_after_a_theme_switch() {
    let dir = temp_config_dir("config-cmd-switch");
    std::fs::write(dir.join("themes/bad.theme"), "style nope fg red\n").unwrap();
    let mut ws = new_workspace();
    ws.init_theme_from(Some("bad"), Some(dir));
    run_command(&mut ws, "theme light");
    run_command(&mut ws, "config");
    assert!(popup_text(&ws).contains("bad:1:"));
}

#[test]
fn test_nmap_and_theme_dialogs_stay_filtered() {
    let dir = temp_config_dir("config-cmd-filter");
    std::fs::write(dir.join("keys"), "nnoremap <Nope> j\n").unwrap();
    std::fs::write(dir.join("themes/bad.theme"), "style nope fg red\n").unwrap();
    let mut ws = new_workspace();
    ws.init_theme_from(Some("bad"), Some(dir.clone()));
    ws.init_keys_from(None, Some(dir));
    run_command(&mut ws, "nmap");
    let text = popup_text(&ws);
    assert!(text.contains("keys:1:") && !text.contains("bad:1:"), "{text}");
    ws.state.popup = None;
    ws.state.modality_stack.pop();
    run_command(&mut ws, "theme");
    let text = popup_text(&ws);
    assert!(text.contains("bad:1:") && !text.contains("keys:1:"), "{text}");
}

#[test]
fn test_col_width_cmd_forms_and_usage_errors() {
    assert_eq!(Cmd::ColWidth { width: 20, count: 1 }, parse("col-width 20").unwrap().unwrap());
    assert_eq!(Cmd::ColWidth { width: 1, count: 5 }, parse("col-width 1 5").unwrap().unwrap());
    assert_eq!(Cmd::ColWidth { width: 255, count: 1 }, parse("col-width 255").unwrap().unwrap());
    for bad in ["col-width", "col-width 0", "col-width 256", "col-width x", "col-width 1.5", "col-width -3", "col-width 20 0", "col-width 20 x", "col-width 20 2 3", "col-widthx"] {
        let err = parse(bad).expect_err(bad);
        assert!(err.contains("Usage: col-width <1-255|default> [count]"), "{bad}: {err}");
    }
}

#[test]
fn test_col_width_sets_columns_and_says_so() {
    let mut ws = new_workspace();
    let col = ws.book.location.col;
    run_command(&mut ws, "col-width 20");
    assert_eq!(20, ws.book.get_col_size(col).unwrap());
    assert_eq!(Some("Column A width 20".to_owned()), last_message(&ws));
    run_command(&mut ws, "col-width 7 3");
    for c in col..col + 3 {
        assert_eq!(7, ws.book.get_col_size(c).unwrap());
    }
    assert_eq!(Some("Columns A:C width 7".to_owned()), last_message(&ws));
    run_command(&mut ws, "col-width 255");
    assert_eq!(255, ws.book.get_col_size(col).unwrap());
    run_command(&mut ws, "col-width 1");
    assert_eq!(1, ws.book.get_col_size(col).unwrap());
}

#[test]
fn test_col_width_errors_change_nothing() {
    let mut ws = new_workspace();
    let before = ws.book.get_col_size(1).unwrap();
    for bad in ["col-width", "col-width 0", "col-width 256", "col-width x", "col-width 9 0"] {
        let mut ws = new_workspace();
        run_command(&mut ws, bad);
        assert_eq!(before, ws.book.get_col_size(1).unwrap(), "{bad}");
        assert_eq!(Some(&Modality::Dialog), ws.state.modality_stack.last(), "{bad}");
    }
    // A late failure at the count: start at the last column.
    ws.book.location.col = crate::book::LAST_COLUMN as usize;
    let last = ws.book.get_col_size(crate::book::LAST_COLUMN as usize).unwrap();
    run_command(&mut ws, "col-width 9 2");
    assert_eq!(last, ws.book.get_col_size(crate::book::LAST_COLUMN as usize).unwrap());
    assert!(ws.state.message.as_ref().unwrap().text.contains("Usage"));
    // The last column alone still works.
    run_command(&mut ws, "col-width 9");
    assert_eq!(9, ws.book.get_col_size(crate::book::LAST_COLUMN as usize).unwrap());
}

#[test]
fn test_col_width_is_one_undo_step_and_redoes() {
    let mut ws = new_workspace();
    let before = ws.book.get_col_size(1).unwrap();
    run_command(&mut ws, "col-width 9 4");
    run(&mut ws, script().char('u'));
    for c in 1..=4 {
        assert_eq!(before, ws.book.get_col_size(c).unwrap());
    }
    run_command(&mut ws, "redo");
    for c in 1..=4 {
        assert_eq!(9, ws.book.get_col_size(c).unwrap());
    }
}

#[test]
fn test_autofit_cols_cmd_forms_and_usage_errors() {
    assert_eq!(Cmd::AutofitCols(1), parse("autofit-cols").unwrap().unwrap());
    assert_eq!(Cmd::AutofitCols(3), parse("autofit-cols 3").unwrap().unwrap());
    for bad in ["autofit-cols 0", "autofit-cols x", "autofit-cols 1 2", "autofit-colsx", "autofit-cols -1"] {
        let err = parse(bad).expect_err(bad);
        assert!(err.contains("Usage: autofit-cols [count]"), "{bad}: {err}");
    }
}

#[test]
fn test_autofit_cols_fits_columns_and_says_so() {
    let mut ws = sort_workspace(&[&["hello world", "", "x"]]);
    run_command(&mut ws, "autofit-cols");
    assert_eq!(11, ws.book.get_col_size(1).unwrap());
    assert_eq!(Some("Fitted column A".to_owned()), last_message(&ws));
    run_command(&mut ws, "col-width 40 3");
    run_command(&mut ws, "autofit-cols 3");
    assert_eq!(11, ws.book.get_col_size(1).unwrap());
    assert_eq!(18, ws.book.get_col_size(2).unwrap());
    assert_eq!(3, ws.book.get_col_size(3).unwrap());
    assert_eq!(Some("Fitted 3 columns (1 empty, reset to 18)".to_owned()), last_message(&ws));
    run(&mut ws, script().char('u'));
    assert_eq!(40, ws.book.get_col_size(2).unwrap());
    // A lone empty column.
    ws.book.location.col = 2;
    run_command(&mut ws, "autofit-cols");
    assert_eq!(Some("Column B is empty, reset to 18".to_owned()), last_message(&ws));
}

#[test]
fn test_autofit_cols_errors_change_nothing() {
    let mut ws = sort_workspace(&[&["hello world"]]);
    let before = ws.book.get_col_size(1).unwrap();
    run_command(&mut ws, "autofit-cols 0");
    assert_eq!(before, ws.book.get_col_size(1).unwrap());
    assert_eq!(Some(&Modality::Dialog), ws.state.modality_stack.last());
    let mut ws = sort_workspace(&[&["hello world"]]);
    ws.book.location.col = crate::book::LAST_COLUMN as usize;
    run_command(&mut ws, "autofit-cols 2");
    assert!(last_message(&ws).unwrap().contains("Usage"));
    assert_eq!(before, ws.book.get_col_size(crate::book::LAST_COLUMN as usize).unwrap());
}

fn width_workspace<'a>() -> Workspace<'a> {
    // Row 1 is short, row 3 is long, in columns A to C.
    sort_workspace(&[
        &["ab", "cd", "ef"],
        &["abcdef", "cdefgh", "efghij"],
        &["abcdefghijkl", "cdefghijklmn", "efghijklmnop"],
    ])
}

#[test]
fn test_gw_fits_the_cursor_column_and_3gw_fits_three() {
    let mut ws = width_workspace();
    run(&mut ws, script().chars("gw"));
    assert_eq!(12, ws.book.get_col_size(1).unwrap());
    assert_eq!(18, ws.book.get_col_size(2).unwrap());
    assert_eq!(Some("Fitted column A".to_owned()), last_message(&ws));
    run(&mut ws, script().chars("3gw"));
    for c in 1..=3 {
        assert_eq!(12, ws.book.get_col_size(c).unwrap());
    }
    assert_eq!(Some("Fitted 3 columns".to_owned()), last_message(&ws));
    // One undo step for the three columns.
    run(&mut ws, script().char('u'));
    assert_eq!(12, ws.book.get_col_size(1).unwrap());
    assert_eq!(18, ws.book.get_col_size(2).unwrap());
}

#[test]
fn test_gw_past_the_last_column_is_an_error_that_changes_nothing() {
    let mut ws = width_workspace();
    ws.book.location.col = crate::book::LAST_COLUMN as usize;
    run(&mut ws, script().chars("2gw"));
    assert!(last_message(&ws).unwrap().contains("Usage"));
}

#[test]
fn test_w_in_range_select_fits_touched_columns_over_the_selected_rows_only() {
    let mut ws = width_workspace();
    // Select A1:B2: the long row 3 is not measured.
    run(&mut ws, script().chars("vjl"));
    run(&mut ws, script().char('w'));
    assert_eq!(6, ws.book.get_col_size(1).unwrap());
    assert_eq!(6, ws.book.get_col_size(2).unwrap());
    assert_eq!(18, ws.book.get_col_size(3).unwrap());
    assert_eq!(Some(&Modality::RangeSelect), ws.state.modality_stack.last());
    assert_eq!(Some("Fitted 2 columns".to_owned()), last_message(&ws));
}

#[test]
fn test_w_in_range_select_ignores_a_count() {
    let mut ws = width_workspace();
    run(&mut ws, script().chars("v"));
    run(&mut ws, script().chars("3w"));
    // Only A1 is selected, so only column A is fitted, to row 1 alone.
    assert_eq!(3, ws.book.get_col_size(1).unwrap());
    assert_eq!(18, ws.book.get_col_size(2).unwrap());
}

#[test]
fn test_w_over_a_whole_column_matches_navigation_mode() {
    let mut ws = width_workspace();
    run(&mut ws, script().chars("vjj"));
    run(&mut ws, script().char('w'));
    assert_eq!(12, ws.book.get_col_size(1).unwrap());
}

#[test]
fn test_col_commands_in_range_select_apply_to_touched_columns_and_ignore_count() {
    let mut ws = width_workspace();
    run(&mut ws, script().chars("vl"));
    run_command(&mut ws, "col-width 30 5");
    assert_eq!(30, ws.book.get_col_size(1).unwrap());
    assert_eq!(30, ws.book.get_col_size(2).unwrap());
    assert_eq!(18, ws.book.get_col_size(3).unwrap());
    assert_eq!(Some("Columns A:B width 30".to_owned()), last_message(&ws));
    run_command(&mut ws, "autofit-cols 5");
    // Row 1 only: "ab" and "cd" are clamped up to 3.
    assert_eq!(3, ws.book.get_col_size(1).unwrap());
    assert_eq!(3, ws.book.get_col_size(2).unwrap());
    assert_eq!(18, ws.book.get_col_size(3).unwrap());
}

#[test]
fn test_gw_and_w_can_be_remapped_and_unmapped() {
    let mut ws = width_workspace();
    map_nav(&mut ws, "<Space>f", "gw");
    run(&mut ws, script().chars(" f"));
    assert_eq!(12, ws.book.get_col_size(1).unwrap());
    let mut ws = width_workspace();
    type_command(&mut ws, "nunmap gw");
    run(&mut ws, script().chars("gw"));
    assert_eq!(18, ws.book.get_col_size(1).unwrap());
    let mut ws = width_workspace();
    type_command(&mut ws, "vunmap w");
    run(&mut ws, script().chars("vw"));
    assert_eq!(18, ws.book.get_col_size(1).unwrap());
}

#[test]
fn test_set_defaultwidth_moves_unset_columns_but_not_set_ones() {
    let mut ws = new_workspace();
    run_command(&mut ws, "col-width 30 2");
    run_command(&mut ws, "set dw=24");
    assert_eq!(24, ws.book.get_col_size(3).unwrap());
    assert_eq!(30, ws.book.get_col_size(1).unwrap());
    assert_eq!(30, ws.book.get_col_size(2).unwrap());
    run_command(&mut ws, "set defaultwidth=10");
    assert_eq!(10, ws.book.get_col_size(3).unwrap());
    assert_eq!(30, ws.book.get_col_size(1).unwrap());
}

#[test]
fn test_set_defaultwidth_bad_values_change_nothing() {
    let mut ws = new_workspace();
    run_command(&mut ws, "set dw=24");
    for bad in ["dw=0", "dw=256", "dw=abc"] {
        run_command(&mut ws, &format!("set {bad}"));
        assert!(last_message(&ws).unwrap().starts_with("E474"), "{bad}");
        assert_eq!(24, ws.book.get_col_size(3).unwrap(), "{bad}");
    }
    run_command(&mut ws, "set dw?");
    assert_eq!(Some("defaultwidth=24".to_owned()), last_message(&ws));
}

#[test]
fn test_config_file_sets_the_default_width() {
    let dir = temp_config_dir("config-dw");
    std::fs::write(dir.join("config"), "set dw=24\n").unwrap();
    let mut ws = new_workspace();
    ws.init_config_from(None, Some(dir));
    assert_eq!(24, ws.book.get_col_size(5).unwrap());
}

#[test]
fn test_col_width_default_parses() {
    assert_eq!(Cmd::ColWidthDefault { count: 1 }, parse("col-width default").unwrap().unwrap());
    assert_eq!(Cmd::ColWidthDefault { count: 3 }, parse("col-width default 3").unwrap().unwrap());
    for bad in ["col-width default 0", "col-width default x", "col-width default 1 2", "col-width defaultx"] {
        let err = parse(bad).expect_err(bad);
        assert!(err.contains("Usage: col-width"), "{bad}: {err}");
    }
}

#[test]
fn test_col_width_default_clears_saved_widths_in_one_undo_step() {
    let mut ws = new_workspace();
    run_command(&mut ws, "set dw=12");
    run_command(&mut ws, "col-width 30 3");
    run_command(&mut ws, "col-width default 3");
    for c in 1..=3 {
        assert_eq!(12, ws.book.get_col_size(c).unwrap());
    }
    assert_eq!(Some("Columns A:C width default (12)".to_owned()), last_message(&ws));
    run_command(&mut ws, "col-width default");
    assert_eq!(Some("Column A width default (12)".to_owned()), last_message(&ws));
    run_command(&mut ws, "col-width 30");
    run(&mut ws, script().char('u'));
    run(&mut ws, script().char('u'));
    run(&mut ws, script().char('u'));
    for c in 1..=3 {
        assert_eq!(30, ws.book.get_col_size(c).unwrap());
    }
}

#[test]
fn test_autofit_empty_column_resets_to_the_option() {
    let mut ws = sort_workspace(&[&["hello world"]]);
    run_command(&mut ws, "set dw=24");
    run_command(&mut ws, "col-width 40 2");
    ws.book.location.col = 2;
    run_command(&mut ws, "autofit-cols");
    assert_eq!(24, ws.book.get_col_size(2).unwrap());
    assert_eq!(Some("Column B is empty, reset to 24".to_owned()), last_message(&ws));
}

#[test]
fn test_wide_column_is_drawn_clipped_and_cursor_stays_visible() {
    let mut ws = new_workspace();
    run(&mut ws, script().chars("ihello").enter());
    cmd_line(&mut ws, "col-width 255");
    let rows = draw_lines(&mut ws, 40, 12);
    assert_eq!(
        Some("Column is wider than the screen, drawn clipped".to_string()),
        message_text(&ws)
    );
    assert!(rows.iter().any(|r| r.contains("hello")), "{rows:#?}");
    assert_eq!(255, ws.book.get_col_size(1).unwrap(), "saved width unchanged");
    // Moving on to the next column keeps it on screen too.
    run(&mut ws, script().char('l'));
    let rows = draw_lines(&mut ws, 40, 12);
    assert!(rows.iter().any(|r| r.contains('B')), "{rows:#?}");
}

#[test]
fn test_wide_column_next_to_frozen_column_still_fits() {
    let mut ws = new_workspace();
    cmd_line(&mut ws, "freeze 0 1");
    run(&mut ws, script().char('l'));
    cmd_line(&mut ws, "col-width 255");
    let rows = draw_lines(&mut ws, 40, 12);
    assert!(rows.iter().any(|r| r.contains('A')) && rows.iter().any(|r| r.contains('B')), "{rows:#?}");
}

#[test]
fn test_ctrl_l_stops_at_255_and_ctrl_h_at_1() {
    let mut ws = new_workspace();
    cmd_line(&mut ws, "col-width 254");
    run(&mut ws, script().ctrl('l').ctrl('l').ctrl('l'));
    assert_eq!(255, ws.book.get_col_size(1).unwrap());
    cmd_line(&mut ws, "col-width 1");
    run(&mut ws, script().ctrl('h'));
    assert_eq!(1, ws.book.get_col_size(1).unwrap());
}

fn fmt_of(ws: &Workspace, row: usize, col: usize) -> String {
    ws.book
        .get_number_format(&Address { sheet: ws.book.location.sheet, row, col })
        .expect("format")
}

#[test]
fn test_format_cmd_parse() {
    assert_eq!(Cmd::Format(None), parse("format").unwrap().unwrap());
    assert_eq!(Cmd::Format(None), parse("format  ").unwrap().unwrap());
    assert_eq!(Cmd::Format(Some("percent")), parse("format percent").unwrap().unwrap());
    assert_eq!(
        Cmd::Format(Some("0.00 \"USD\"")),
        parse("format 0.00 \"USD\"").unwrap().unwrap()
    );
    assert_eq!(Err("Usage: format [<preset|code>]"), parse("formatx"));
}

#[test]
fn test_format_cmd_cursor_cell_preset_and_message() {
    let mut ws = new_workspace();
    run_command(&mut ws, "format Currency");
    assert_eq!("$#,##0.00", fmt_of(&ws, 1, 1));
    assert_eq!("General", fmt_of(&ws, 1, 2));
    assert_eq!(Some("Format currency: 1 cell".to_owned()), last_message(&ws));
}

#[test]
fn test_format_cmd_raw_code_with_spaces_and_quotes() {
    let mut ws = new_workspace();
    run_command(&mut ws, "format 0.00 \"USD\"");
    assert_eq!("0.00 \"USD\"", fmt_of(&ws, 1, 1));
    assert_eq!(Some("Format 0.00 \"USD\": 1 cell".to_owned()), last_message(&ws));
}

#[test]
fn test_format_cmd_bad_code_is_error_and_changes_nothing() {
    let mut ws = new_workspace();
    script().char('v').chars("jjll").char(':').chars("format \"abc").enter()
        .run(&mut ws).expect("script");
    assert_eq!(Some("Invalid number format: \"abc".to_owned()), last_message(&ws));
    assert_eq!(super::MessageLevel::Error, ws.state.message.as_ref().unwrap().level);
    for r in 1..=3 {
        for c in 1..=3 {
            assert_eq!("General", fmt_of(&ws, r, c));
        }
    }
}

#[test]
fn test_format_cmd_range_select_exact_cells_one_undo() {
    let mut ws = new_workspace();
    script().char('v').chars("jjll").char(':').chars("format percent").enter()
        .run(&mut ws).expect("script");
    for r in 1..=4 {
        for c in 1..=4 {
            let want = if r <= 3 && c <= 3 { "0.00%" } else { "General" };
            assert_eq!(want, fmt_of(&ws, r, c), "{r}:{c}");
        }
    }
    assert_eq!(Some("Format percent: 9 cells".to_owned()), last_message(&ws));
    run_command(&mut ws, "undo");
    for r in 1..=3 {
        for c in 1..=3 {
            assert_eq!("General", fmt_of(&ws, r, c), "{r}:{c}");
        }
    }
}

#[test]
fn test_format_cmd_no_argument_shows_format_and_changes_nothing() {
    let mut ws = new_workspace();
    run_command(&mut ws, "format");
    assert_eq!(Some("Format: General".to_owned()), last_message(&ws));
    run_command(&mut ws, "format percent");
    run_command(&mut ws, "format");
    assert_eq!(Some("Format: 0.00%".to_owned()), last_message(&ws));
    assert_eq!("0.00%", fmt_of(&ws, 1, 1));
}

#[test]
fn test_format_cmd_never_changes_col_width() {
    let mut ws = new_workspace();
    let before = ws.book.get_col_size(1).unwrap();
    run_command(&mut ws, "format currency");
    assert_eq!(before, ws.book.get_col_size(1).unwrap());
}

#[test]
fn test_format_span_cmd_parse() {
    assert_eq!(Cmd::FormatCols(None, "percent"), parse("format-cols percent").unwrap().unwrap());
    assert_eq!(Cmd::FormatCols(Some(3), "percent"), parse("format-cols 3 percent").unwrap().unwrap());
    assert_eq!(Cmd::FormatCols(None, "0"), parse("format-cols 0").unwrap().unwrap());
    assert_eq!(Cmd::FormatCols(Some(3), "0"), parse("format-cols 3 0").unwrap().unwrap());
    assert_eq!(
        Cmd::FormatRows(Some(2), "0.00 \"USD\""),
        parse("format-rows 2 0.00 \"USD\"").unwrap().unwrap()
    );
    assert_eq!(Cmd::FormatRows(None, "0.000"), parse("format-rows 0.000").unwrap().unwrap());
    assert_eq!(Err("Usage: format-cols [count] <preset|code>"), parse("format-cols"));
    assert_eq!(Err("Usage: format-rows [count] <preset|code>"), parse("format-rows  "));
    assert_eq!(Err("Usage: format-rows [count] <preset|code>"), parse("format-rowsx"));
}

#[test]
fn test_format_cols_cursor_and_count_one_undo() {
    let mut ws = new_workspace();
    run_command(&mut ws, "format-cols 0.000");
    assert_eq!("0.000", fmt_of(&ws, 1, 1));
    assert_eq!("0.000", fmt_of(&ws, 500, 1));
    assert_eq!("General", fmt_of(&ws, 1, 2));
    assert_eq!(Some("Format 0.000: column A".to_owned()), last_message(&ws));
    run_command(&mut ws, "format-cols 3 Percent");
    for c in 1..=3 {
        assert_eq!("0.00%", fmt_of(&ws, 7, c), "{c}");
    }
    assert_eq!("General", fmt_of(&ws, 7, 4));
    assert_eq!(Some("Format percent: columns A:C".to_owned()), last_message(&ws));
    run_command(&mut ws, "undo");
    assert_eq!("0.000", fmt_of(&ws, 7, 1));
    assert_eq!("General", fmt_of(&ws, 7, 2));
}

#[test]
fn test_format_rows_cursor_and_count() {
    let mut ws = new_workspace();
    run_command(&mut ws, "format-rows 2 date");
    assert_eq!("yyyy-mm-dd", fmt_of(&ws, 1, 1));
    assert_eq!("yyyy-mm-dd", fmt_of(&ws, 2, 300));
    assert_eq!("General", fmt_of(&ws, 3, 1));
    assert_eq!(Some("Format date: rows 1:2".to_owned()), last_message(&ws));
}

#[test]
fn test_format_span_errors_change_nothing() {
    let mut ws = new_workspace();
    for cmd in ["format-cols 0 percent", "format-cols 16385 percent", "format-rows 0 percent"] {
        run_command(&mut ws, cmd);
        assert_eq!(super::MessageLevel::Error, ws.state.message.as_ref().unwrap().level, "{cmd}");
        assert_eq!("General", fmt_of(&ws, 1, 1), "{cmd}");
    }
    run_command(&mut ws, "format-cols \"abc");
    assert_eq!(Some("Invalid number format: \"abc".to_owned()), last_message(&ws));
    assert_eq!("General", fmt_of(&ws, 1, 1));
}

#[test]
fn test_format_cols_range_select_ignores_count() {
    let mut ws = new_workspace();
    script().chars("l").char('v').chars("jjl").char(':').chars("format-cols 9 percent").enter()
        .run(&mut ws).expect("script");
    assert_eq!("General", fmt_of(&ws, 1, 1));
    assert_eq!("0.00%", fmt_of(&ws, 1, 2));
    assert_eq!("0.00%", fmt_of(&ws, 900, 3));
    assert_eq!("General", fmt_of(&ws, 1, 4));
    assert_eq!(Some("Format percent: columns B:C".to_owned()), last_message(&ws));
}

#[test]
fn test_format_rows_range_select_and_width_unchanged() {
    let mut ws = new_workspace();
    let width = ws.book.get_col_size(1).unwrap();
    script().char('j').char('v').chars("jl").char(':').chars("format-rows currency").enter()
        .run(&mut ws).expect("script");
    assert_eq!("General", fmt_of(&ws, 1, 1));
    assert_eq!("$#,##0.00", fmt_of(&ws, 2, 50));
    assert_eq!("$#,##0.00", fmt_of(&ws, 3, 1));
    assert_eq!("General", fmt_of(&ws, 4, 1));
    assert_eq!(width, ws.book.get_col_size(1).unwrap());
}

#[test]
fn test_decimals_cmd_parse() {
    assert_eq!(Cmd::Decimals(DecimalsChange::Add(1)), parse("decimals +1").unwrap().unwrap());
    assert_eq!(Cmd::Decimals(DecimalsChange::Add(-2)), parse("decimals -2").unwrap().unwrap());
    assert_eq!(Cmd::Decimals(DecimalsChange::Set(3)), parse("decimals 3").unwrap().unwrap());
    assert_eq!(Cmd::Decimals(DecimalsChange::Set(0)), parse("decimals 0").unwrap().unwrap());
    let usage = Err("Usage: decimals <+n|-n|n>");
    for bad in ["decimals", "decimals x", "decimals +", "decimals 1.5", "decimals 16", "decimals +16", "decimals -16", "decimals 1 2", "decimalsx"] {
        assert_eq!(usage, parse(bad), "{bad}");
    }
}

#[test]
fn test_decimals_cmd_cursor_cell_and_message() {
    let mut ws = new_workspace();
    run_command(&mut ws, "decimals +1");
    assert_eq!("0.0", fmt_of(&ws, 1, 1));
    assert_eq!("General", fmt_of(&ws, 1, 2));
    assert_eq!(Some("Decimals +1: 1 cell".to_owned()), last_message(&ws));
    run_command(&mut ws, "decimals 3");
    assert_eq!("0.000", fmt_of(&ws, 1, 1));
    assert_eq!(Some("Decimals 3: 1 cell".to_owned()), last_message(&ws));
    run_command(&mut ws, "decimals -1");
    assert_eq!("0.00", fmt_of(&ws, 1, 1));
    assert_eq!(Some("Decimals -1: 1 cell".to_owned()), last_message(&ws));
}

#[test]
fn test_decimals_cmd_range_select_mixed_one_format_one_undo() {
    let mut ws = new_workspace();
    run_command(&mut ws, "format percent");
    script().char('v').chars("jjll").char(':').chars("decimals +1").enter()
        .run(&mut ws).expect("script");
    for r in 1..=3 {
        for c in 1..=3 {
            assert_eq!("0.000%", fmt_of(&ws, r, c), "{r}:{c}");
        }
    }
    assert_eq!("General", fmt_of(&ws, 4, 4));
    assert_eq!(Some("Decimals +1: 9 cells".to_owned()), last_message(&ws));
    run_command(&mut ws, "undo");
    assert_eq!("0.00%", fmt_of(&ws, 1, 1));
    assert_eq!("General", fmt_of(&ws, 2, 2));
}

#[test]
fn test_decimals_cmd_refusal_is_error_and_changes_nothing() {
    let mut ws = new_workspace();
    run_command(&mut ws, "format date");
    script().char('v').chars("jj").char(':').chars("decimals +1").enter()
        .run(&mut ws).expect("script");
    assert_eq!(Some("Cannot change decimals of yyyy-mm-dd".to_owned()), last_message(&ws));
    assert_eq!(super::MessageLevel::Error, ws.state.message.as_ref().unwrap().level);
    assert_eq!("yyyy-mm-dd", fmt_of(&ws, 1, 1));
    assert_eq!("General", fmt_of(&ws, 2, 1));
}

#[test]
fn test_decimals_cmd_bad_argument_shows_usage() {
    let mut ws = new_workspace();
    run_command(&mut ws, "decimals 99");
    assert_eq!(Some(&Modality::Dialog), ws.state.modality_stack.last());
    assert!(popup_text(&ws).contains("Usage: decimals <+n|-n|n>"));
    assert_eq!("General", fmt_of(&ws, 1, 1));
}

#[test]
fn test_alt_preset_keys_format_cursor_cell() {
    let cases = [
        ('1', "0.00"), ('2', "hh:mm:ss"), ('3', "yyyy-mm-dd"), ('4', "$#,##0.00"),
        ('5', "0.00%"), ('6', "0.00E+00"), ('0', "General"),
    ];
    for (key, code) in cases {
        let mut ws = new_workspace();
        run_command(&mut ws, "format percent");
        script().alt(key).run(&mut ws).expect("script");
        assert_eq!(code, fmt_of(&ws, 1, 1), "<A-{key}>");
        assert_eq!("General", fmt_of(&ws, 2, 1));
    }
    let mut ws = new_workspace();
    script().alt('4').run(&mut ws).expect("script");
    assert_eq!(Some("Format currency: 1 cell".to_owned()), last_message(&ws));
}

#[test]
fn test_alt_preset_key_in_range_select_formats_selection_in_one_undo() {
    let mut ws = new_workspace();
    script().char('v').chars("jjll").alt('5').run(&mut ws).expect("script");
    for r in 1..=3 {
        for c in 1..=3 {
            assert_eq!("0.00%", fmt_of(&ws, r, c), "{r}:{c}");
        }
    }
    assert_eq!("General", fmt_of(&ws, 4, 4));
    assert_eq!(Some("Format percent: 9 cells".to_owned()), last_message(&ws));
    run_command(&mut ws, "undo");
    assert_eq!("General", fmt_of(&ws, 1, 1));
}

#[test]
fn test_alt_decimal_keys_in_both_modes() {
    let mut ws = new_workspace();
    script().alt('.').run(&mut ws).expect("script");
    assert_eq!("0.0", fmt_of(&ws, 1, 1));
    assert_eq!(Some("Decimals +1: 1 cell".to_owned()), last_message(&ws));
    script().alt('.').alt(',').alt(',').run(&mut ws).expect("script");
    assert_eq!("0", fmt_of(&ws, 1, 1));
    script().char('v').char('j').alt('.').run(&mut ws).expect("script");
    assert_eq!("0.0", fmt_of(&ws, 1, 1));
    assert_eq!("0.0", fmt_of(&ws, 2, 1));
    assert_eq!(Some("Decimals +1: 2 cells".to_owned()), last_message(&ws));
}

#[test]
fn test_alt_decimal_key_refusal_changes_nothing() {
    let mut ws = new_workspace();
    script().alt('3').alt('.').run(&mut ws).expect("script");
    assert_eq!("yyyy-mm-dd", fmt_of(&ws, 1, 1));
    assert_eq!(Some("Cannot change decimals of yyyy-mm-dd".to_owned()), last_message(&ws));
    assert_eq!(super::MessageLevel::Error, ws.state.message.as_ref().unwrap().level);
}

#[test]
fn test_plain_digit_is_still_a_numeric_prefix_beside_alt_digit() {
    let mut ws = new_workspace();
    script().chars("3j").run(&mut ws).expect("script");
    assert_eq!(4, ws.book.location.row);
    assert_eq!("General", fmt_of(&ws, 1, 1));
}

#[test]
fn test_alt_format_keys_can_be_remapped_and_unmapped() {
    let mut ws = new_workspace();
    type_command(&mut ws, "nunmap <A-5>");
    script().alt('5').run(&mut ws).expect("script");
    assert_eq!("General", fmt_of(&ws, 1, 1));
    type_command(&mut ws, "nnoremap Q <A-4>");
    script().char('Q').run(&mut ws).expect("script");
    assert_eq!("$#,##0.00", fmt_of(&ws, 1, 1));
}

#[test]
fn test_alt_format_keys_are_in_the_help() {
    let mut ws = new_workspace();
    script().alt('h').run(&mut ws).expect("script");
    let help = popup_text(&ws);
    for k in ["<A-1>", "<A-5>", "<A-0>", "<A-.>", "<A-,>"] {
        assert!(help.contains(k), "{k} missing from help");
    }
}

fn has_style(ws: &Workspace, row: usize, col: usize, flag: StyleFlag) -> bool {
    ws.book.cell_has_style(&Address { sheet: ws.book.location.sheet, row, col }, flag)
}

#[test]
fn test_style_cmd_parse() {
    assert_eq!(Cmd::Style("bold underline"), parse("style bold underline").unwrap().unwrap());
    assert_eq!(Cmd::Style("bold"), parse("bold").unwrap().unwrap());
    assert_eq!(Cmd::Style("italic"), parse("italic").unwrap().unwrap());
    assert_eq!(Cmd::Style("underline"), parse("underline").unwrap().unwrap());
    assert_eq!(Cmd::Style("strike"), parse("strike").unwrap().unwrap());
    assert_eq!(Err("Usage: style <attr>..."), parse("style"));
    assert_eq!(Err("Usage: style <attr>..."), parse("style   "));
    assert_eq!(Err("Usage: bold"), parse("bold now"));
    // Not these commands: other words that start the same.
    assert!(parse("bolder").is_err() || parse("bolder").unwrap().is_none());
}

#[test]
fn test_parse_style_attrs() {
    assert_eq!(
        Ok(vec![
            StyleAttr::Toggle(StyleFlag::Bold),
            StyleAttr::Toggle(StyleFlag::Italic),
            StyleAttr::Toggle(StyleFlag::Underline),
            StyleAttr::Toggle(StyleFlag::Strike),
        ]),
        parse_style_attrs("bold italic underline strike"),
    );
    assert_eq!(
        Ok(vec![
            StyleAttr::Set(StyleFlag::Bold, false),
            StyleAttr::Set(StyleFlag::Italic, false),
            StyleAttr::Set(StyleFlag::Underline, false),
            StyleAttr::Set(StyleFlag::Strike, false),
        ]),
        parse_style_attrs("nobold  noitalic\tnounderline nostrike"),
    );
    assert_eq!(Err("Unknown style: shiny".to_string()), parse_style_attrs("bold shiny"));
    assert_eq!(Err("Unknown style: Bold".to_string()), parse_style_attrs("Bold"));
    assert_eq!(Err("Repeated style: bold".to_string()), parse_style_attrs("bold nobold"));
    assert_eq!(Err("Repeated style: bold".to_string()), parse_style_attrs("bold bold"));
    assert_eq!(Err("Usage: style <attr>...".to_string()), parse_style_attrs(" "));
}

#[test]
fn test_style_cmd_cursor_cell_and_message() {
    let mut ws = new_workspace();
    run_command(&mut ws, "bold");
    assert!(has_style(&ws, 1, 1, StyleFlag::Bold));
    assert!(!has_style(&ws, 2, 1, StyleFlag::Bold));
    assert_eq!(Some("Bold: 1 cell".to_owned()), last_message(&ws));
    run_command(&mut ws, "bold");
    assert!(!has_style(&ws, 1, 1, StyleFlag::Bold));
    assert_eq!(Some("Not bold: 1 cell".to_owned()), last_message(&ws));
    run_command(&mut ws, "style italic underline");
    assert!(has_style(&ws, 1, 1, StyleFlag::Italic) && has_style(&ws, 1, 1, StyleFlag::Underline));
    assert_eq!(Some("Style italic underline: 1 cell".to_owned()), last_message(&ws));
    run_command(&mut ws, "strike");
    assert!(has_style(&ws, 1, 1, StyleFlag::Strike));
    run_command(&mut ws, "style nostrike noitalic");
    assert!(!has_style(&ws, 1, 1, StyleFlag::Strike) && !has_style(&ws, 1, 1, StyleFlag::Italic));
    assert_eq!(Some("Style nostrike noitalic: 1 cell".to_owned()), last_message(&ws));
}

#[test]
fn test_style_cmd_range_select_decided_by_first_cell_one_undo() {
    let mut ws = new_workspace();
    // Bold the second cell only; the first is plain, so the selection gains bold.
    script().char('j').chars(":bold").enter().char('k').run(&mut ws).expect("script");
    script().char('v').chars("jj").char(':').chars("bold").enter()
        .run(&mut ws).expect("script");
    for r in 1..=3 {
        assert!(has_style(&ws, r, 1, StyleFlag::Bold), "{r}");
    }
    assert!(!has_style(&ws, 4, 1, StyleFlag::Bold));
    assert_eq!(Some("Bold: 3 cells".to_owned()), last_message(&ws));
    run_command(&mut ws, "undo");
    assert!(!has_style(&ws, 1, 1, StyleFlag::Bold));
    assert!(has_style(&ws, 2, 1, StyleFlag::Bold));
}

#[test]
fn test_style_cmd_unknown_word_changes_nothing() {
    let mut ws = new_workspace();
    run_command(&mut ws, "style bold shiny");
    assert_eq!(Some("Unknown style: shiny".to_owned()), last_message(&ws));
    assert_eq!(super::MessageLevel::Error, ws.state.message.as_ref().unwrap().level);
    assert!(!has_style(&ws, 1, 1, StyleFlag::Bold));
    run_command(&mut ws, "style");
    assert_eq!(Some(&Modality::Dialog), ws.state.modality_stack.last());
    assert!(popup_text(&ws).contains("Usage: style <attr>..."));
}

#[test]
fn test_style_span_cmd_parse() {
    assert_eq!(Cmd::StyleCols(None, "bold"), parse("style-cols bold").unwrap().unwrap());
    assert_eq!(Cmd::StyleCols(Some(3), "bold"), parse("style-cols 3 bold").unwrap().unwrap());
    assert_eq!(Cmd::StyleRows(Some(2), "bold nounderline"), parse("style-rows 2 bold nounderline").unwrap().unwrap());
    assert_eq!(Cmd::StyleRows(None, "italic"), parse("style-rows italic").unwrap().unwrap());
    assert_eq!(Err("Usage: style-cols [count] <attr>..."), parse("style-cols"));
    assert_eq!(Err("Usage: style-cols [count] <attr>..."), parse("style-cols 3"));
    assert_eq!(Err("Usage: style-rows [count] <attr>..."), parse("style-rows  "));
    assert_eq!(Err("Usage: style-rows [count] <attr>..."), parse("style-rowsx"));
}

#[test]
fn test_style_cols_cursor_count_message_one_undo() {
    let mut ws = new_workspace();
    run_command(&mut ws, "style-cols bold");
    assert!(has_style(&ws, 1, 1, StyleFlag::Bold) && has_style(&ws, 500, 1, StyleFlag::Bold));
    assert!(!has_style(&ws, 1, 2, StyleFlag::Bold));
    assert_eq!(Some("Bold: column A".to_owned()), last_message(&ws));
    run_command(&mut ws, "style-cols 3 italic");
    for c in 1..=3 {
        assert!(has_style(&ws, 7, c, StyleFlag::Italic), "{c}");
    }
    assert!(!has_style(&ws, 7, 4, StyleFlag::Italic));
    assert_eq!(Some("Italic: columns A:C".to_owned()), last_message(&ws));
    run_command(&mut ws, "undo");
    assert!(!has_style(&ws, 7, 1, StyleFlag::Italic));
    assert!(has_style(&ws, 7, 1, StyleFlag::Bold));
}

#[test]
fn test_style_cols_toggle_decided_by_cursor_cell_and_keeps_format() {
    let mut ws = new_workspace();
    run_command(&mut ws, "format-cols percent");
    // Bold only the cursor cell A1: the column toggle then clears bold.
    run_command(&mut ws, "bold");
    run_command(&mut ws, "style-cols bold");
    assert!(!has_style(&ws, 1, 1, StyleFlag::Bold));
    assert_eq!("0.00%", fmt_of(&ws, 9, 1));
}

#[test]
fn test_style_rows_cursor_and_count() {
    let mut ws = new_workspace();
    run_command(&mut ws, "style-rows 2 underline");
    assert!(has_style(&ws, 1, 1, StyleFlag::Underline) && has_style(&ws, 2, 300, StyleFlag::Underline));
    assert!(!has_style(&ws, 3, 1, StyleFlag::Underline));
    assert_eq!(Some("Underline: rows 1:2".to_owned()), last_message(&ws));
}

#[test]
fn test_style_span_errors_change_nothing() {
    let mut ws = new_workspace();
    for cmd in ["style-cols 0 bold", "style-cols 16385 bold", "style-rows 0 bold", "style-rows 1048577 bold"] {
        run_command(&mut ws, cmd);
        assert_eq!(super::MessageLevel::Error, ws.state.message.as_ref().unwrap().level, "{cmd}");
        assert!(!has_style(&ws, 1, 1, StyleFlag::Bold), "{cmd}");
    }
    run_command(&mut ws, "style-cols bold shiny");
    assert_eq!(Some("Unknown style: shiny".to_owned()), last_message(&ws));
    assert!(!has_style(&ws, 1, 1, StyleFlag::Bold));
}

#[test]
fn test_style_cols_range_select_ignores_count() {
    let mut ws = new_workspace();
    script().chars("l").char('v').chars("jjl").char(':').chars("style-cols 9 bold").enter()
        .run(&mut ws).expect("script");
    assert!(!has_style(&ws, 1, 1, StyleFlag::Bold));
    assert!(has_style(&ws, 1, 2, StyleFlag::Bold) && has_style(&ws, 900, 3, StyleFlag::Bold));
    assert!(!has_style(&ws, 1, 4, StyleFlag::Bold));
    assert_eq!(Some("Bold: columns B:C".to_owned()), last_message(&ws));
}

fn cell_style_of(ws: &Workspace, row: usize, col: usize) -> ironcalc::base::types::Style {
    ws.book
        .get_cell_style(&Address { sheet: ws.book.location.sheet, row, col })
        .expect("style")
}

fn halign_of(ws: &Workspace, row: usize, col: usize) -> Option<ironcalc::base::types::HorizontalAlignment> {
    cell_style_of(ws, row, col).alignment.map(|a| a.horizontal)
}

#[test]
fn test_align_and_font_color_cmd_parse() {
    assert_eq!(Cmd::Align("center"), parse("align center").unwrap().unwrap());
    assert_eq!(Cmd::Align("general"), parse("align general").unwrap().unwrap());
    assert_eq!(Err("Usage: align <left|center|right|general>"), parse("align"));
    assert_eq!(Err("Usage: align <left|center|right|general>"), parse("align middle"));
    assert_eq!(Cmd::FontColor("red"), parse("font-color red").unwrap().unwrap());
    assert_eq!(Err("Usage: font-color <color>"), parse("font-color"));
}

#[test]
fn test_parse_style_attrs_align_and_color() {
    use crate::book::HAlign;
    assert_eq!(
        Ok(vec![StyleAttr::Align(HAlign::Center), StyleAttr::Color(Some("#800000".to_string())), StyleAttr::Toggle(StyleFlag::Bold)]),
        parse_style_attrs("center color=red bold"),
    );
    assert_eq!(Ok(vec![StyleAttr::Color(None)]), parse_style_attrs("nocolor"));
    assert_eq!(Ok(vec![StyleAttr::Align(HAlign::General)]), parse_style_attrs("general"));
    assert_eq!(Err("Repeated style: alignment".to_string()), parse_style_attrs("left right"));
    assert_eq!(Err("Repeated style: color".to_string()), parse_style_attrs("color=red nocolor"));
    assert_eq!(Err("Repeated style: color".to_string()), parse_style_attrs("color=red color=blue"));
    assert!(parse_style_attrs("color=nonsense").is_err());
}

#[test]
fn test_align_cmd_sets_clears_and_undoes() {
    use ironcalc::base::types::HorizontalAlignment as H;
    let mut ws = new_workspace();
    run_command(&mut ws, "align center");
    assert_eq!(Some(H::Center), halign_of(&ws, 1, 1));
    assert_eq!(Some("Align center: 1 cell".to_owned()), last_message(&ws));
    run_command(&mut ws, "align right");
    assert_eq!(Some(H::Right), halign_of(&ws, 1, 1));
    run_command(&mut ws, "align general");
    assert_eq!(Some(H::General), halign_of(&ws, 1, 1));
    run_command(&mut ws, "undo");
    assert_eq!(Some(H::Right), halign_of(&ws, 1, 1));
    run_command(&mut ws, "style left bold");
    assert_eq!(Some(H::Left), halign_of(&ws, 1, 1));
    assert!(has_style(&ws, 1, 1, StyleFlag::Bold));
}

#[test]
fn test_align_cmd_range_select_and_cols() {
    use ironcalc::base::types::HorizontalAlignment as H;
    let mut ws = new_workspace();
    script().char('v').chars("jl").char(':').chars("align right").enter().run(&mut ws).expect("script");
    for (r, c) in [(1, 1), (1, 2), (2, 1), (2, 2)] {
        assert_eq!(Some(H::Right), halign_of(&ws, r, c));
    }
    assert_eq!(Some("Align right: 4 cells".to_owned()), last_message(&ws));
    assert_eq!(None, halign_of(&ws, 3, 3));
    run_command(&mut ws, "style-cols 2 center");
    assert_eq!(Some(H::Center), halign_of(&ws, 50, 2));
}

#[test]
fn test_font_color_cmd_sets_and_rejects_bad_color() {
    use ironcalc::base::types::Color;
    let mut ws = new_workspace();
    run_command(&mut ws, "font-color red");
    assert_eq!(Color::Rgb("#800000".to_string()), cell_style_of(&ws, 1, 1).font.color);
    assert_eq!(Some("Font color #800000: 1 cell".to_owned()), last_message(&ws));
    run_command(&mut ws, "style nocolor");
    assert_eq!(Color::None, cell_style_of(&ws, 1, 1).font.color);
    run_command(&mut ws, "style color=#00ff00 bold");
    assert_eq!(Color::Rgb("#00ff00".to_string()), cell_style_of(&ws, 1, 1).font.color);
    run_command(&mut ws, "font-color notacolor");
    assert_eq!(super::MessageLevel::Error, ws.state.message.as_ref().unwrap().level);
    assert_eq!(Color::Rgb("#00ff00".to_string()), cell_style_of(&ws, 1, 1).font.color);
    // A bad word after a good color changes nothing.
    run_command(&mut ws, "style color=blue shiny");
    assert_eq!(Color::Rgb("#00ff00".to_string()), cell_style_of(&ws, 1, 1).font.color);
}

#[test]
fn test_clear_style_cmd_parse() {
    assert_eq!(Cmd::ClearStyle, parse("clear-style").unwrap().unwrap());
    assert_eq!(Cmd::ClearStyleCols(None), parse("clear-style-cols").unwrap().unwrap());
    assert_eq!(Cmd::ClearStyleCols(Some(2)), parse("clear-style-cols 2").unwrap().unwrap());
    assert_eq!(Cmd::ClearStyleRows(None), parse("clear-style-rows").unwrap().unwrap());
    assert_eq!(Cmd::ClearStyleRows(Some(3)), parse("clear-style-rows 3").unwrap().unwrap());
    assert_eq!(Err("Usage: clear-style"), parse("clear-style now"));
    assert_eq!(Err("Usage: clear-style-cols [count]"), parse("clear-style-cols x"));
    assert_eq!(Err("Usage: clear-style-rows [count]"), parse("clear-style-rows 1 2"));
}

#[test]
fn test_clear_style_cmd_cursor_cell_message_and_undo() {
    let mut ws = new_workspace();
    run_command(&mut ws, "style bold underline center color=red");
    run_command(&mut ws, "format percent");
    run_command(&mut ws, "clear-style");
    assert!(!has_style(&ws, 1, 1, StyleFlag::Bold) && !has_style(&ws, 1, 1, StyleFlag::Underline));
    assert_eq!(None, halign_of(&ws, 1, 1));
    assert_eq!("General", fmt_of(&ws, 1, 1));
    assert_eq!(Some("Cleared style: 1 cell".to_owned()), last_message(&ws));
    run_command(&mut ws, "undo");
    assert!(has_style(&ws, 1, 1, StyleFlag::Bold));
    assert_eq!("0.00%", fmt_of(&ws, 1, 1));
}

#[test]
fn test_clear_style_cols_and_rows_cmds() {
    let mut ws = new_workspace();
    run_command(&mut ws, "style-cols 3 bold");
    run_command(&mut ws, "clear-style-cols 2");
    assert!(!has_style(&ws, 40, 1, StyleFlag::Bold) && !has_style(&ws, 40, 2, StyleFlag::Bold));
    assert!(has_style(&ws, 40, 3, StyleFlag::Bold));
    assert_eq!(Some("Cleared style: columns A:B".to_owned()), last_message(&ws));
    run_command(&mut ws, "style-rows 2 italic");
    run_command(&mut ws, "clear-style-rows");
    assert!(!has_style(&ws, 1, 50, StyleFlag::Italic));
    assert!(has_style(&ws, 2, 50, StyleFlag::Italic));
    assert_eq!(Some("Cleared style: row 1".to_owned()), last_message(&ws));
    run_command(&mut ws, "clear-style-cols 0");
    assert_eq!(super::MessageLevel::Error, ws.state.message.as_ref().unwrap().level);
}
