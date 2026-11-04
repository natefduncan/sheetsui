//! Ui rendering logic
use std::{
    path::{Path, PathBuf},
    process::ExitCode,
    str::FromStr,
};

use crate::book::{self, search::Direction, AddressRange, Book, SortKey};

use anyhow::{anyhow, Result};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ironcalc::base::expressions::{types::Area, utils::number_to_column};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Flex, Layout},
    style::{Modifier, Style},
    widgets::Block,
};
use tui_prompts::{State, Status, TextPrompt, TextState};
use ratatui_textarea::{CursorMove, TextArea};

mod cmd;
mod help;
mod history;
mod options;
// Parts are not used until the later stages of planning/keys-plan.md.
#[allow(dead_code)]
pub mod keys;
pub mod render;
mod search;
pub mod theme;
#[cfg(test)]
mod test;

use cmd::{Cmd, CmdSortKey};
use keys::{
    builtin::{NavHandler, RangeHandler, Table},
    map::{resolve, resolve_timeout, KeyMap, MapCommand, Resolution, RhsItem, Target, UserTable},
    Key,
};
pub use history::CommandHistory;
use options::Options;
use search::{CaseOptions, SearchState};
use theme::{lookup, parse::Warning, Slot, Theme};
use render::{markdown::Markdown, viewport::ViewportState};

/// Default file name used when creating a new empty workbook.
const DEFAULT_WORKBOOK_NAME: &str = "Untitled.sui";

#[derive(Default, Debug, PartialEq, Clone)]
pub enum Modality {
    #[default]
    Navigate,
    CellEdit,
    Command,
    /// The `/` prompt.
    Search,
    Dialog,
    RangeSelect,
    Quit,
}

#[derive(Debug, Default)]
pub struct RangeSelection {
    pub original_location: Option<Address>,
    pub start: Option<Address>,
    pub end: Option<Address>,
}

impl RangeSelection {
    pub fn get_range(&self) -> Option<(Address, Address)> {
        if let (Some(start), Some(end)) = (&self.start, &self.end) {
            return Some((
                Address {
                    sheet: start.sheet,
                    row: std::cmp::min(start.row, end.row),
                    col: std::cmp::min(start.col, end.col),
                },
                Address {
                    sheet: end.sheet,
                    row: std::cmp::max(start.row, end.row),
                    col: std::cmp::max(start.col, end.col),
                },
            ));
        }
        None
    }

    pub fn reset_range_selection(&mut self) {
        self.start = None;
        self.end = None;
    }
}

#[derive(Debug)]
pub enum ClipboardContents {
    Cell(String),
    Range(Vec<Vec<String>>),
}

/// How a message line is styled.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageLevel {
    Info,
    Error,
}

/// A one-line message, shown in the outer block's bottom title until the next
/// key press. Only theme feedback uses it for now.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message {
    pub level: MessageLevel,
    pub text: String,
}

#[derive(Debug)]
pub struct AppState<'ws> {
    pub modality_stack: Vec<Modality>,
    pub viewport_state: ViewportState,
    pub command_state: TextState<'ws>,
    pub command_history: CommandHistory,
    /// Patterns searched for with `/`, `?`, `*` and `#`.
    pub search_history: CommandHistory,
    /// The text of the `/` prompt.
    pub search_prompt: TextState<'ws>,
    /// The last search, for `n`.
    pub search: SearchState,
    /// The `:set` options, for this session only.
    pub options: Options,
    pub numeric_prefix: Vec<char>,
    /// Keys typed so far that are the start of a longer key sequence.
    pub pending_keys: Vec<Key>,
    /// The user's key mappings and the clock on a half-typed sequence.
    pub keymap: KeyMap,
    pub range_select: RangeSelection,
    pub dialog_scroll: u16,
    pub theme: Theme,
    /// Warnings from the last theme load.
    pub theme_warnings: Vec<Warning>,
    /// The user themes directory. `None` means there are only built-ins.
    pub themes_dir: Option<PathBuf>,
    pub message: Option<Message>,
    dirty: bool,
    popup: Option<Markdown>,
    clipboard: Option<ClipboardContents>,
}

impl<'ws> Default for AppState<'ws> {
    fn default() -> Self {
        AppState {
            modality_stack: vec![Modality::default()],
            viewport_state: Default::default(),
            command_state: Default::default(),
            command_history: Default::default(),
            search_history: Default::default(),
            search_prompt: Default::default(),
            search: Default::default(),
            options: Default::default(),
            numeric_prefix: Default::default(),
            pending_keys: Default::default(),
            keymap: Default::default(),
            range_select: Default::default(),
            dialog_scroll: 0,
            theme: Theme::default(),
            theme_warnings: Vec::new(),
            themes_dir: None,
            message: None,
            dirty: false,
            popup: Default::default(),
            clipboard: Default::default(),
        }
    }
}
impl<'ws> AppState<'ws> {
    pub fn modality(&'ws self) -> &'ws Modality {
        self.modality_stack.last().unwrap()
    }

    pub fn pop_modality(&mut self) {
        if self.modality_stack.len() > 1 {
            self.modality_stack.pop();
        }
    }

    pub fn get_n_prefix(&self) -> usize {
        let prefix = self
            .numeric_prefix
            .iter()
            .map(|c| c.to_digit(10).unwrap())
            .try_fold(0_usize, |acc, n| {
                acc.checked_mul(10)?.checked_add(n as usize)
            })
            .unwrap_or(1);
        if prefix == 0 {
            return 1;
        }
        prefix
    }

    pub fn reset_n_prefix(&mut self) {
        self.numeric_prefix.clear();
    }
}

// TODO(jwall): Should we just be using `Area` for this?.
/// The Address in a Table.
#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Clone)]
pub struct Address {
    pub sheet: u32,
    pub row: usize,
    pub col: usize,
}

impl Address {
    pub fn new(row: usize, col: usize) -> Self {
        Self { sheet: 0, row, col }
    }

    pub fn to_range_part(&self) -> String {
        let count = if self.col == 26 {
            1
        } else {
            (self.col / 26) + 1
        };
        format!(
            "{}{}",
            render::viewport::COLNAMES[(self.col - 1) % 26].repeat(count),
            self.row
        )
    }
}

impl Default for Address {
    fn default() -> Self {
        Address::new(1, 1)
    }
}

/// A workspace defining our UI state.
pub struct Workspace<'ws> {
    name: PathBuf,
    book: Book,
    pub(crate) state: AppState<'ws>,
    text_area: TextArea<'ws>,
}

fn parse_csv_rows(text: &str) -> Result<Vec<Vec<String>>, anyhow::Error> {
    let reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(text.as_bytes());
    let mut rows = Vec::new();
    for rec in reader.into_byte_records() {
        let record = rec?;
        let mut row = Vec::with_capacity(record.len());
        for i in 0..record.len() {
            row.push(
                String::from_utf8_lossy(
                    record.get(i).expect("Unexpected failure to get cell row"),
                )
                .to_string(),
            );
        }
        rows.push(row);
    }
    Ok(rows)
}

impl<'ws> Workspace<'ws> {
    /// Constructs a new Workspace from an `Book` with a path for the name.
    pub fn new(book: Book, name: PathBuf) -> Self {
        let mut ws = Self {
            book,
            name,
            state: AppState::default(),
            text_area: reset_text_area("".to_owned(), &Theme::default()),
        };
        ws.handle_movement_change();
        ws
    }

    pub fn new_empty(_locale: &str, _tz: &str) -> Result<Self> {
        let mut book = Book::default();
        book.dirty = false;
        Ok(Self::new(book, PathBuf::from_str(DEFAULT_WORKBOOK_NAME).unwrap()))
    }

    /// Loads a workspace from a path.
    pub fn load(path: &Path, locale: &str, tz: &str) -> Result<Self> {
        let book = load_book(path, locale, tz)?;
        Ok(Workspace::new(book, path.to_path_buf()))
    }

    /// Loads a new `Book` into a `Workspace` from a path.
    pub fn load_into<P: Into<PathBuf>>(&mut self, path: P) -> Result<()> {
        let path: PathBuf = path.into();
        // FIXME(zaphar): This should be managed better.
        let book = load_book(&path, "en", "America/New_York")?;
        self.book = book;
        self.name = path;
        Ok(())
    }

    pub fn selected_range_to_string(&self) -> String {
        let state = &self.state;
        if let Some((start, end)) = state.range_select.get_range() {
            let a1 = format!("{}:{}", start.to_range_part(), end.to_range_part());
            if let Some(ref start_addr) = state.range_select.start {
                if start_addr.sheet != self.book.location.sheet {
                    return format!(
                        "{}!{}",
                        self.book
                            .get_sheet_name_by_idx(start_addr.sheet as usize)
                            .expect("No such sheet index"),
                        a1
                    );
                }
            }
            return a1;
        }
        String::new()
    }

    /// Move a row down in the current sheet.
    pub fn move_down(&mut self) -> Result<()> {
        let mut loc = self.book.location.clone();
        if loc.row < (book::LAST_ROW as usize) {
            loc.row += 1;
            self.book.move_to(&loc)?;
        }
        Ok(())
    }

    /// Move to the top row without changing columns
    pub fn move_to_top(&mut self) -> Result<()> {
        self.book.move_to(&Address {
            sheet: self.book.location.sheet,
            row: 1,
            col: self.book.location.col,
        })?;
        Ok(())
    }

    /// Move a row up in the current sheet.
    pub fn move_up(&mut self) -> Result<()> {
        let mut loc = self.book.location.clone();
        if loc.row > 1 {
            loc.row -= 1;
            self.book.move_to(&loc)?;
        }
        Ok(())
    }

    /// Move a column to the left in the current sheet.
    pub fn move_left(&mut self) -> Result<()> {
        let mut loc = self.book.location.clone();
        if loc.col > 1 {
            loc.col -= 1;
            self.book.move_to(&loc)?;
        }
        Ok(())
    }

    /// Move a column to the left in the current sheet.
    pub fn move_right(&mut self) -> Result<()> {
        let mut loc = self.book.location.clone();
        if loc.col < (book::LAST_COLUMN as usize) {
            loc.col += 1;
            self.book.move_to(&loc)?;
        }
        Ok(())
    }

    /// Handle input in our ui loop.
    pub fn handle_input(&mut self, evt: Event) -> Result<Option<ExitCode>> {
        if let Event::Key(key) = evt {
            self.state.message = None;
            // Everything one key changes undoes as one action.
            self.book.begin_action();
            let result = self.dispatch_key(key);
            self.book.end_action();
            return result;
        }
        Ok(None)
    }

    /// Sends a key to the handler for the current mode.
    fn dispatch_key(&mut self, key: KeyEvent) -> Result<Option<ExitCode>> {
        match self.state.modality() {
            Modality::Navigate => self.handle_navigation_input(key),
            Modality::CellEdit => self.handle_edit_input(key),
            Modality::Command => self.handle_command_input(key),
            Modality::Search => self.handle_search_input(key),
            Modality::Dialog => self.handle_dialog_input(key),
            Modality::RangeSelect => self.handle_range_select_input(key),
            Modality::Quit => self.handle_quit_dialog(key),
        }
    }

    /// When the keys typed so far give up waiting for more, if they are
    /// waiting on a timeout.
    pub fn key_deadline(&self) -> Option<std::time::Instant> {
        self.state.keymap.deadline
    }

    /// The wait for the rest of a key sequence ran out. Runs the longest user
    /// mapping the pending keys start, or drops them if there is none.
    pub fn handle_key_timeout(&mut self) -> Result<Option<ExitCode>> {
        if self.state.keymap.deadline.is_none() {
            return Ok(None);
        }
        let table = match self.state.modality() {
            Modality::Navigate => Some(&self.state.keymap.nav),
            Modality::RangeSelect => Some(&self.state.keymap.range),
            _ => None,
        };
        let target = table.and_then(|t| resolve_timeout(t, &self.state.pending_keys));
        self.finish_keys();
        let Some(target) = target else {
            return Ok(None);
        };
        self.book.begin_action();
        let result = self.run_user_target(target);
        self.book.end_action();
        result
    }

    /// The pending keys are used up: run, dropped or cancelled.
    fn finish_keys(&mut self) {
        self.state.pending_keys.clear();
        self.state.keymap.clear_timeout();
    }

    /// Acts on what the lookup made of the pending keys.
    fn apply_resolution<H>(
        &mut self,
        resolution: Resolution<H>,
        run: impl FnOnce(&mut Self, H) -> Result<Option<ExitCode>>,
    ) -> Result<Option<ExitCode>> {
        match resolution {
            Resolution::Builtin(handler) => {
                self.finish_keys();
                run(self, handler)
            }
            Resolution::User(target) => {
                self.finish_keys();
                self.run_user_target(target)
            }
            Resolution::Wait { timeout } => {
                if timeout {
                    self.state.keymap.start_timeout();
                } else {
                    self.state.keymap.clear_timeout();
                }
                Ok(None)
            }
            Resolution::Drop => {
                self.finish_keys();
                Ok(None)
            }
        }
    }

    /// Looks up the pending keys in the user mappings for a mode, then in the
    /// built-in table. A replay skips the user mappings.
    fn resolve_pending<H: Copy>(
        &self,
        user: fn(&KeyMap) -> &UserTable,
        builtin: &Table<H>,
    ) -> Resolution<H> {
        let none = UserTable::default();
        let user = if self.state.keymap.replaying {
            &none
        } else {
            user(&self.state.keymap)
        };
        resolve(user, builtin, &self.state.pending_keys)
    }

    fn run_user_target(&mut self, target: Target) -> Result<Option<ExitCode>> {
        match target {
            Target::Replay(items) => self.replay(&items),
            Target::Unmapped => Ok(None),
        }
    }

    /// Replays a mapping as if its keys were typed, once for each count of the
    /// numeric prefix. User mappings do not apply to the keys, only the
    /// built-ins. A step that fails drops the rest and shows the error.
    fn replay(&mut self, items: &[RhsItem]) -> Result<Option<ExitCode>> {
        let times = self.state.get_n_prefix();
        self.state.reset_n_prefix();
        let was_replaying = std::mem::replace(&mut self.state.keymap.replaying, true);
        let mut exit = None;
        'replay: for _ in 0..times {
            for item in items {
                let step = match item {
                    RhsItem::Key(key) => self.dispatch_key(KeyEvent::new(key.code, key.mods)),
                    RhsItem::Cmd(text) => self.run_mapped_command(text),
                };
                match step {
                    Ok(None) => {}
                    Ok(code) => {
                        exit = code;
                        break 'replay;
                    }
                    Err(e) => {
                        self.set_message(MessageLevel::Error, e.to_string());
                        break 'replay;
                    }
                }
            }
        }
        self.state.keymap.replaying = was_replaying;
        Ok(exit)
    }

    /// Runs the text of a `<Cmd>…<CR>` as a command, without command mode. A
    /// command that does not parse is an error here, not a dialog, so the
    /// replay can stop.
    fn run_mapped_command(&mut self, text: &str) -> Result<Option<ExitCode>> {
        if keys::command::parse(text).map_err(|e| anyhow!(e))?.is_none() {
            match cmd::parse(text) {
                Err(msg) => return Err(anyhow!(msg)),
                Ok(None) => return Err(anyhow!("Unrecognized command {text}")),
                Ok(Some(_)) => {}
            }
        }
        self.handle_command(text.to_owned())
    }

    fn undo(&mut self) -> Result<()> {
        if self.book.undo()? {
            self.handle_movement_change();
        }
        Ok(())
    }

    fn redo(&mut self) -> Result<()> {
        if self.book.redo()? {
            self.handle_movement_change();
        }
        Ok(())
    }

    fn render_help_text(&self) -> Markdown {
        // TODO(zaphar): We should be sourcing these from our actual help documentation.
        // Ideally we would also render the markdown content properly.
        // https://github.com/zaphar/sheetsui/issues/22
        match self.state.modality() {
            Modality::Navigate => help::to_widget("navigate", &self.state.theme, &self.state.keymap),
            Modality::CellEdit => help::to_widget("edit", &self.state.theme, &self.state.keymap),
            Modality::Command => help::to_widget("command", &self.state.theme, &self.state.keymap),
            Modality::Search => help::to_widget("navigate", &self.state.theme, &self.state.keymap),
            Modality::RangeSelect => help::to_widget("visual", &self.state.theme, &self.state.keymap),
            _ => help::to_widget("", &self.state.theme, &self.state.keymap),
        }
    }

    fn handle_command_input(&mut self, key: event::KeyEvent) -> Result<Option<ExitCode>> {
        if key.kind == KeyEventKind::Press {
            match key.code {
                KeyCode::Enter => return self.exit_command_mode(),
                KeyCode::Esc => return self.cancel_command_mode(),
                KeyCode::Char('h') if key.modifiers == KeyModifiers::ALT => {
                    self.enter_dialog_mode(self.render_help_text());
                    return Ok(None);
                }
                KeyCode::Up => {
                    let shown = self.state.command_state.value().to_owned();
                    if let Some(text) = self.state.command_history.older(&shown) {
                        self.set_command_text(text);
                    }
                    return Ok(None);
                }
                KeyCode::Down => {
                    let shown = self.state.command_state.value().to_owned();
                    if let Some(text) = self.state.command_history.newer(&shown) {
                        self.set_command_text(text);
                    }
                    return Ok(None);
                }
                _ => {
                    // An edit makes the text shown the prefix for the next Up.
                    self.state.command_history.reset();
                }
            }
        }
        self.state.command_state.handle_key_event(key);
        Ok(None)
    }

    fn handle_search_input(&mut self, key: event::KeyEvent) -> Result<Option<ExitCode>> {
        if key.kind == KeyEventKind::Press {
            match key.code {
                KeyCode::Enter => return self.exit_search_mode(),
                KeyCode::Esc => return self.cancel_search_mode(),
                KeyCode::Up => {
                    let shown = self.state.search_prompt.value().to_owned();
                    if let Some(text) = self.state.search_history.older(&shown) {
                        self.set_search_text(text);
                    }
                    return Ok(None);
                }
                KeyCode::Down => {
                    let shown = self.state.search_prompt.value().to_owned();
                    if let Some(text) = self.state.search_history.newer(&shown) {
                        self.set_search_text(text);
                    }
                    return Ok(None);
                }
                // An edit makes the text shown the prefix for the next Up.
                _ => self.state.search_history.reset(),
            }
        }
        self.state.search_prompt.handle_key_event(key);
        self.update_preview();
        Ok(None)
    }

    fn handle_quit_dialog(&mut self, key: event::KeyEvent) -> Result<Option<ExitCode>> {
        if key.kind == KeyEventKind::Press {
            match key.code {
                KeyCode::Esc | KeyCode::Char('n') | KeyCode::Char('N') => {
                    self.exit_quit_mode()?;
                    return Ok(Some(ExitCode::SUCCESS));
                }
                KeyCode::Char('y') | KeyCode::Char('Y') => {
                    // We have been asked to save the file first.
                    self.save_file()?;
                    self.exit_quit_mode()?;
                    return Ok(Some(ExitCode::SUCCESS));
                }
                _ => return Ok(None),
            }
        }
        Ok(None)
    }

    fn handle_dialog_input(&mut self, key: event::KeyEvent) -> Result<Option<ExitCode>> {
        if key.kind == KeyEventKind::Press {
            match key.code {
                KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q') => self.exit_dialog_mode()?,
                KeyCode::Char('h') if key.modifiers == KeyModifiers::ALT => {
                    self.exit_dialog_mode()?
                }
                KeyCode::Char('j') | KeyCode::Down => {
                    self.state.dialog_scroll = self.state.dialog_scroll.saturating_add(1);
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    self.state.dialog_scroll = self.state.dialog_scroll.saturating_sub(1);
                }
                code => {
                    if let Some(widget) = &self.state.popup {
                        widget.handle_input(code);
                    }
                }
            }
        }
        Ok(None)
    }

    fn handle_edit_input(&mut self, key: event::KeyEvent) -> Result<Option<ExitCode>> {
        if key.kind == KeyEventKind::Press {
            match key.code {
                KeyCode::Char('h') if key.modifiers == KeyModifiers::ALT => {
                    self.enter_dialog_mode(self.render_help_text());
                    return Ok(None);
                }
                KeyCode::Char('r') if key.modifiers == KeyModifiers::CONTROL => {
                    self.enter_range_select_mode(false);
                    return Ok(None);
                }
                KeyCode::Char('p') if key.modifiers == KeyModifiers::CONTROL => {
                    self.text_area
                        .set_yank_text(self.selected_range_to_string());
                    self.text_area.paste();
                    self.state.dirty = true;
                    return Ok(None);
                }
                // Return so the key does not go into the next edit's text.
                KeyCode::Enter => {
                    self.exit_edit_mode(true)?;
                    return Ok(None);
                }
                KeyCode::Esc => {
                    self.exit_edit_mode(false)?;
                    return Ok(None);
                }
                _ => {
                    // NOOP
                }
            }
        }
        // TODO(zaphar): Some specialized editing keybinds
        // * Select All
        // * Copy
        // * Paste
        if self.text_area.input(key) {
            self.state.dirty = true;
        }
        Ok(None)
    }

    fn handle_command(&mut self, cmd_text: String) -> Result<Option<ExitCode>> {
        if cmd_text.is_empty() {
            return Ok(None);
        }
        match keys::command::parse(&cmd_text) {
            Ok(Some(MapCommand::List(mode))) => {
                let text = keys::map::listing_text(mode, &self.state.keymap);
                self.enter_dialog_mode(Markdown::from_str_themed(&text, &self.state.theme));
                return Ok(None);
            }
            Ok(Some(command)) => {
                if let Err(msg) = self.state.keymap.apply(command) {
                    self.set_message(MessageLevel::Error, msg);
                }
                return Ok(None);
            }
            Ok(None) => {}
            Err(msg) => {
                self.enter_dialog_mode(Markdown::from_str(&msg));
                return Ok(None);
            }
        }
        match cmd::parse(&cmd_text) {
            Ok(Some(Cmd::Edit(path))) => {
                self.load_into(path)?;
                Ok(None)
            }
            Ok(Some(Cmd::Help(maybe_topic))) => {
                self.enter_dialog_mode(help::to_widget(maybe_topic.unwrap_or(""), &self.state.theme, &self.state.keymap));
                Ok(None)
            }
            Ok(Some(Cmd::Theme(maybe_name))) => {
                match maybe_name {
                    Some(name) => self.switch_theme(name),
                    None => self.enter_dialog_mode(self.theme_listing()),
                }
                Ok(None)
            }
            Ok(Some(Cmd::Set(args))) => {
                match self.state.options.set(args) {
                    Ok(Some(text)) => self.set_message(MessageLevel::Info, text),
                    Ok(None) => {}
                    Err(text) => self.set_message(MessageLevel::Error, text),
                }
                // As in vim, setting `hlsearch` shows the highlight that
                // `:noh` hid, even if the option was already on.
                let sets_hls = args
                    .split_whitespace()
                    .any(|a| matches!(a.trim_end_matches('!'), "hls" | "hlsearch"));
                if sets_hls && self.state.options.hlsearch() {
                    self.state.search.highlight = true;
                }
                Ok(None)
            }
            Ok(Some(Cmd::NoHighlight)) => {
                self.state.search.highlight = false;
                Ok(None)
            }
            Ok(Some(Cmd::Write(maybe_path))) => {
                if let Some(path) = maybe_path {
                    self.save_to(path)?;
                } else {
                    self.save_file()?;
                }
                Ok(None)
            }
            Ok(Some(Cmd::ExportCsv(path))) => {
                self.book
                    .save_sheet_to_csv(self.book.location.sheet, path)?;
                Ok(None)
            }
            Ok(Some(Cmd::InsertColumns(count))) => {
                self.book.insert_columns(self.book.location.col, count)?;
                self.book.evaluate();
                Ok(None)
            }
            Ok(Some(Cmd::InsertRows(count))) => {
                self.book.insert_rows(self.book.location.row, count)?;
                self.book.evaluate();
                Ok(None)
            }
            Ok(Some(Cmd::RenameSheet(idx, name))) => {
                match idx {
                    Some(idx) => {
                        self.book.set_sheet_name(idx as u32, name)?;
                    }
                    _ => {
                        self.book.set_sheet_name(self.book.location.sheet, name)?;
                    }
                }
                Ok(None)
            }
            Ok(Some(Cmd::NewSheet(name))) => {
                self.book.new_sheet(name)?;
                Ok(None)
            }
            Ok(Some(Cmd::SelectSheet(name))) => {
                self.book.select_sheet_by_name(name);
                Ok(None)
            }
            Ok(Some(Cmd::Quit)) => self.quit_app(),
            Ok(Some(Cmd::Undo)) => {
                self.undo()?;
                Ok(None)
            }
            Ok(Some(Cmd::Redo)) => {
                self.redo()?;
                Ok(None)
            }
            Ok(Some(Cmd::ColorRows(count, color))) => {
                let row_count = count.unwrap_or(1);
                let row = self.book.location.row;
                for r in row..(row + row_count) {
                    self.book.set_row_style(
                        &[("fill.color", &color)],
                        self.book.location.sheet,
                        r,
                    )?;
                }
                Ok(None)
            }
            Ok(Some(Cmd::ColorColumns(count, color))) => {
                let col_count = count.unwrap_or(1);
                let col = self.book.location.col;
                for c in col..(col + col_count) {
                    self.book.set_col_style(
                        &[("fill.color", &color)],
                        self.book.location.sheet,
                        c,
                    )?;
                }
                Ok(None)
            }
            Ok(Some(Cmd::ColorCell(color))) => {
                let sheet = self.book.location.sheet;
                let area = if let Some((start, end)) = self.state.range_select.get_range() {
                    Area {
                        sheet,
                        row: start.row as i32,
                        column: start.col as i32,
                        width: (end.col - start.col + 1) as i32,
                        height: (end.row - start.row + 1) as i32,
                    }
                } else {
                    let address = self.book.location.clone();
                    Area {
                        sheet,
                        row: address.row as i32,
                        column: address.col as i32,
                        width: 1,
                        height: 1,
                    }
                };
                self.book
                    .set_cell_style(&[("fill.color", &color)], &area)?;
                Ok(None)
            }
            Ok(Some(Cmd::SystemPaste)) => {
                let rows = self.get_rows_from_system_clipboard()?;
                self.state.clipboard = Some(ClipboardContents::Range(rows));
                self.paste_range()?;
                Ok(None)
            }
            Ok(Some(Cmd::Sort { skip, keys })) => {
                // A sort error is the user's to fix, so show it, do not exit.
                if let Err(e) = self.sort_rows(skip, &keys) {
                    self.enter_dialog_mode(Markdown::from_str(&e.to_string()));
                }
                Ok(None)
            }
            Ok(None) => {
                self.enter_dialog_mode(Markdown::from_str(&format!(
                    "Unrecognized commmand {}",
                    cmd_text
                )));
                Ok(None)
            }
            Err(msg) => {
                self.enter_dialog_mode(Markdown::from_str(msg));
                Ok(None)
            }
        }
    }

    fn handle_numeric_prefix(&mut self, digit: char) {
        self.state.numeric_prefix.push(digit);
    }

    /// Takes a digit as part of the numeric prefix, if no other keys are
    /// pending. Returns whether it did.
    fn take_numeric_prefix(&mut self, key: &Key) -> bool {
        if !self.state.pending_keys.is_empty() {
            return false;
        }
        match key.as_char() {
            Some(d) if d.is_ascii_digit() => {
                self.handle_numeric_prefix(d);
                true
            }
            _ => false,
        }
    }

    fn handle_range_select_input(&mut self, key: event::KeyEvent) -> Result<Option<ExitCode>> {
        if key.kind != KeyEventKind::Press {
            return Ok(None);
        }
        let key = Key::from_event(&key);
        if key == Key::plain(KeyCode::Esc) {
            if !self.state.numeric_prefix.is_empty() || !self.state.pending_keys.is_empty() {
                self.state.reset_n_prefix();
                self.finish_keys();
            } else {
                self.state.range_select.start = None;
                self.state.range_select.end = None;
                self.exit_range_select_mode()?;
            }
            self.state.keymap.clear_timeout();
            return Ok(None);
        }
        if self.take_numeric_prefix(&key) {
            return Ok(None);
        }
        self.state.pending_keys.push(key);
        let resolution = self.resolve_pending(|k| &k.range, keys::builtin::range_select());
        self.apply_resolution(resolution, Self::run_range_handler)
    }

    fn run_range_handler(&mut self, handler: RangeHandler) -> Result<Option<ExitCode>> {
        match handler {
            RangeHandler::Help => {
                self.enter_dialog_mode(self.render_help_text());
            }
            RangeHandler::ClearRangeAll => {
                if let Some((start, end)) = self.state.range_select.get_range() {
                    self.book.clear_cell_range_all(start, end)?;
                }
            }
            RangeHandler::ClearRange => {
                if let Some((start, end)) = self.state.range_select.get_range() {
                    self.book.clear_cell_range(start, end)?;
                }
            }
            RangeHandler::SortDesc => {
                self.sort_range_selection(false)?;
            }
            RangeHandler::SortAsc => {
                self.sort_range_selection(true)?;
            }
            RangeHandler::MoveLeft => {
                self.run_with_prefix(|ws: &mut Workspace<'_>| -> Result<()> {
                    ws.move_left()?;
                    Ok(())
                })?;
                self.maybe_update_range_end();
            }
            RangeHandler::MoveDown => {
                self.run_with_prefix(|ws: &mut Workspace<'_>| -> Result<()> {
                    ws.move_down()?;
                    Ok(())
                })?;
                self.maybe_update_range_end();
            }
            RangeHandler::MoveUp => {
                self.run_with_prefix(|ws: &mut Workspace<'_>| -> Result<()> {
                    ws.move_up()?;
                    Ok(())
                })?;
                self.maybe_update_range_end();
            }
            RangeHandler::MoveRight => {
                self.run_with_prefix(|ws: &mut Workspace<'_>| -> Result<()> {
                    ws.move_right()?;
                    Ok(())
                })?;
                self.maybe_update_range_end();
            }
            RangeHandler::Select => {
                if self.update_range_selection()? {
                    self.exit_range_select_mode()?;
                }
            }
            RangeHandler::NextSheet => {
                self.state.range_select.reset_range_selection();
                self.run_with_prefix(|ws: &mut Workspace<'_>| -> Result<()> {
                    ws.book.select_next_sheet();
                    Ok(())
                })?;
            }
            RangeHandler::PrevSheet => {
                self.state.range_select.reset_range_selection();
                self.run_with_prefix(|ws: &mut Workspace<'_>| -> Result<()> {
                    ws.book.select_prev_sheet();
                    Ok(())
                })?;
            }
            RangeHandler::CopyFormatted => {
                self.copy_range(true)?;
                self.exit_range_select_mode()?;
            }
            RangeHandler::CopyRaw => {
                self.copy_range(false)?;
                self.exit_range_select_mode()?;
            }
            RangeHandler::Extend => {
                if let (Some(from), Some(to)) = (
                    self.state.range_select.start.as_ref(),
                    self.state.range_select.end.as_ref(),
                ) {
                    self.book.extend_to(from, to)?;
                }
                self.exit_range_select_mode()?;
            }
            RangeHandler::EnterCommand => {
                self.enter_command_mode();
            }
        }
        Ok(None)
    }

    fn copy_range(&mut self, formatted: bool) -> Result<(), anyhow::Error> {
        use arboard::Clipboard;
        self.update_range_selection()?;
        match &self.state.range_select.get_range() {
            Some((start, end)) => {
                let mut rows = Vec::new();
                for row in (AddressRange { start, end }).as_rows() {
                    let mut cols = Vec::new();
                    for cell in row {
                        cols.push(if formatted {
                            self.book.get_cell_addr_rendered(&cell)?
                        } else {
                            self.book.get_cell_addr_contents(&cell)?
                        });
                    }
                    rows.push(cols);
                }
                let mut cb = Clipboard::new()?;
                let (html, csv) = self
                    .book
                    .range_to_clipboard_content(AddressRange { start, end })?;
                cb.set_html(html, Some(csv))?;
                self.state.clipboard = Some(ClipboardContents::Range(rows));
            }
            None => {
                self.copy_cell_to_clipboard(formatted)?;
            }
        }
        Ok(())
    }

    /// Sort the rows of the last range selection, or of the used area when
    /// there is no selection, keeping the first `skip` rows in place.
    fn sort_rows(&mut self, skip: usize, keys: &[CmdSortKey]) -> Result<()> {
        let (first_row, last_row, columns) = match self.state.range_select.get_range() {
            Some((start, end)) => (start.row, end.row, Some(start.col..=end.col)),
            None => (1, self.book.get_size()?.0, None),
        };
        let row_count = (last_row + 1).saturating_sub(first_row);
        if row_count == 0 {
            return Ok(());
        }
        if skip > 0 && skip >= row_count {
            return Err(anyhow!(
                "All {} rows are skipped; nothing to sort.",
                row_count
            ));
        }
        let keys = keys
            .iter()
            .map(|k| {
                let col = k.col.unwrap_or(self.book.location.col);
                if let Some(columns) = &columns {
                    if !columns.contains(&col) {
                        return Err(anyhow!(
                            "Key column {} is outside the selected range.",
                            number_to_column(col as i32).unwrap_or_default()
                        ));
                    }
                }
                Ok(SortKey {
                    col,
                    ascending: k.ascending,
                })
            })
            .collect::<Result<Vec<_>>>()?;
        self.book
            .sort_rows(self.book.location.sheet, first_row + skip, last_row, &keys)
    }

    /// Sort the rows from the range start to the cursor by the cursor's
    /// column, then leave Range Select mode.
    ///
    /// Does nothing if no range start is set, or if Range Select was not
    /// started from Navigate mode: leaving it from CellEdit pastes the range
    /// into the cell being edited.
    fn sort_range_selection(&mut self, ascending: bool) -> Result<()> {
        let Some(start) = self.state.range_select.start.clone() else {
            return Ok(());
        };
        let stack = &self.state.modality_stack;
        if stack.len() < 2 || stack[stack.len() - 2] != Modality::Navigate {
            return Ok(());
        }
        let cursor = self.book.location.clone();
        let result = self.book.sort_rows(
            cursor.sheet,
            std::cmp::min(start.row, cursor.row),
            std::cmp::max(start.row, cursor.row),
            &[SortKey {
                col: cursor.col,
                ascending,
            }],
        );
        self.exit_range_select_mode()?;
        // A sort error is the user's to fix, so show it, do not exit.
        if let Err(e) = result {
            self.enter_dialog_mode(Markdown::from_str(&e.to_string()));
        }
        Ok(())
    }

    fn get_rows_from_system_clipboard(&mut self) -> Result<Vec<Vec<String>>, anyhow::Error> {
        use arboard::Clipboard;
        let mut cb = Clipboard::new()?;
        let txt = cb.get_text().map_err(|e| anyhow!(e))?;
        parse_csv_rows(&txt)
    }

    fn copy_cell_to_clipboard(&mut self, formatted: bool) -> Result<(), anyhow::Error> {
        use arboard::Clipboard;
        let contents = if formatted {
            self.book.get_current_cell_rendered()?
        } else {
            self.book.get_current_cell_contents()?
        };
        let mut cb = Clipboard::new()?;
        cb.set_text(&contents)?;
        self.state.clipboard = Some(ClipboardContents::Cell(contents));
        Ok(())
    }

    fn update_range_selection(&mut self) -> Result<bool, anyhow::Error> {
        Ok(if self.state.range_select.start.is_none() {
            self.state.range_select.start = Some(self.book.location.clone());
            self.state.range_select.end = Some(self.book.location.clone());
            false
        } else {
            self.state.range_select.end = Some(self.book.location.clone());
            true
        })
    }

    fn maybe_update_range_end(&mut self) {
        if self.state.range_select.start.is_some() {
            self.state.range_select.end = Some(self.book.location.clone());
        }
    }

    fn handle_navigation_input(&mut self, key: event::KeyEvent) -> Result<Option<ExitCode>> {
        if key.kind != KeyEventKind::Press {
            return Ok(None);
        }
        let key = Key::from_event(&key);
        if key == Key::plain(KeyCode::Esc) {
            self.state.reset_n_prefix();
            self.finish_keys();
            return Ok(None);
        }
        if self.take_numeric_prefix(&key) {
            return Ok(None);
        }
        self.state.pending_keys.push(key);
        let resolution = self.resolve_pending(|k| &k.nav, keys::builtin::navigation());
        self.apply_resolution(resolution, Self::run_nav_handler)
    }

    fn run_nav_handler(&mut self, handler: NavHandler) -> Result<Option<ExitCode>> {
        match handler {
            NavHandler::ToggleBold => {
                let address = self.book.location.clone();
                let style = self.book.get_cell_style(&address).map(|s| s.font.b);
                self.toggle_bool_style(style, "font.b", &address)?;
            }
            NavHandler::ToggleItalic => {
                let address = self.book.location.clone();
                let style = self.book.get_cell_style(&address).map(|s| s.font.i);
                self.toggle_bool_style(style, "font.i", &address)?;
            }
            NavHandler::EnterEdit => {
                self.enter_edit_mode();
            }
            NavHandler::EnterCommand => {
                self.enter_command_mode();
            }
            NavHandler::Save => {
                self.save_file()?;
            }
            NavHandler::Substitute => {
                self.book.clear_current_cell()?;
                self.text_area = reset_text_area(String::new(), &self.state.theme);
                self.enter_edit_mode();
            }
            NavHandler::EnterRangeSelect => {
                self.enter_range_select_mode(false);
            }
            NavHandler::CopyCell => {
                self.copy_cell_to_clipboard(false)?;
            }
            NavHandler::CopyCellFormatted => {
                self.copy_cell_to_clipboard(true)?;
            }
            NavHandler::EnterVisual => {
                self.enter_range_select_mode(true);
            }
            NavHandler::Paste => {
                self.paste_range()?;
            }
            NavHandler::Help => {
                self.enter_dialog_mode(self.render_help_text());
            }
            NavHandler::NextSheet => {
                self.run_with_prefix(|ws: &mut Workspace<'_>| -> Result<()> {
                    ws.book.select_next_sheet();
                    Ok(())
                })?;
            }
            NavHandler::PrevSheet => {
                self.run_with_prefix(|ws: &mut Workspace<'_>| -> Result<()> {
                    ws.book.select_prev_sheet();
                    Ok(())
                })?;
            }
            NavHandler::ClearCell => {
                self.book.clear_current_cell()?;
            }
            NavHandler::ClearCellAll => {
                self.book.clear_current_cell_all()?;
            }
            NavHandler::ColWider => {
                self.run_with_prefix(|ws: &mut Workspace<'_>| -> Result<()> {
                    let Address {
                        sheet: _,
                        row: _,
                        col,
                    } = &ws.book.location;
                    ws.book
                        .set_col_size(*col, ws.book.get_col_size(*col)? + 1)?;
                    Ok(())
                })?;
            }
            NavHandler::ColNarrower => {
                self.run_with_prefix(|ws: &mut Workspace<'_>| -> Result<()> {
                    let Address {
                        sheet: _,
                        row: _,
                        col,
                    } = &ws.book.location;
                    let curr_size = ws.book.get_col_size(*col)?;
                    if curr_size > 1 {
                        ws.book.set_col_size(*col, curr_size - 1)?;
                    }
                    Ok(())
                })?;
            }
            NavHandler::Quit => {
                return self.quit_app();
            }
            NavHandler::MoveDown => {
                self.run_with_prefix(|ws: &mut Workspace<'_>| -> Result<()> {
                    ws.move_down()?;
                    ws.handle_movement_change();
                    Ok(())
                })?;
            }
            NavHandler::MoveUp => {
                self.run_with_prefix(|ws: &mut Workspace<'_>| -> Result<()> {
                    ws.move_up()?;
                    ws.handle_movement_change();
                    Ok(())
                })?;
            }
            NavHandler::MoveLeft => {
                self.run_with_prefix(|ws: &mut Workspace<'_>| -> Result<()> {
                    ws.move_left()?;
                    ws.handle_movement_change();
                    Ok(())
                })?;
            }
            NavHandler::MoveRight => {
                self.run_with_prefix(|ws: &mut Workspace<'_>| -> Result<()> {
                    ws.move_right()?;
                    ws.handle_movement_change();
                    Ok(())
                })?;
            }
            NavHandler::MoveTop => {
                self.move_to_top()?;
            }
            NavHandler::InsertRowBelow => {
                self.book.insert_rows(self.book.location.row + 1, 1)?;
                self.move_down()?;
                self.handle_movement_change();
                self.enter_edit_mode();
            }
            NavHandler::InsertRowAbove => {
                self.book.insert_rows(self.book.location.row, 1)?;
                self.move_up()?;
                self.handle_movement_change();
                self.enter_edit_mode();
            }
            NavHandler::SearchForward => {
                self.state.reset_n_prefix();
                self.enter_search_mode(Direction::Forward);
            }
            NavHandler::SearchBackward => {
                self.state.reset_n_prefix();
                self.enter_search_mode(Direction::Backward);
            }
            NavHandler::SearchNext => {
                let count = self.state.get_n_prefix();
                self.state.reset_n_prefix();
                self.search_next(self.state.search.direction, count);
            }
            NavHandler::SearchPrev => {
                let count = self.state.get_n_prefix();
                self.state.reset_n_prefix();
                self.search_next(self.state.search.direction.reverse(), count);
            }
            NavHandler::SearchCellForward => {
                let count = self.state.get_n_prefix();
                self.state.reset_n_prefix();
                self.search_cell(Direction::Forward, count);
            }
            NavHandler::SearchCellBackward => {
                let count = self.state.get_n_prefix();
                self.state.reset_n_prefix();
                self.search_cell(Direction::Backward, count);
            }
            NavHandler::Undo => {
                self.run_with_prefix(|ws: &mut Workspace<'_>| -> Result<()> { ws.undo() })?;
            }
            NavHandler::Redo => {
                self.run_with_prefix(|ws: &mut Workspace<'_>| -> Result<()> { ws.redo() })?;
            }
        }
        Ok(None)
    }

    fn toggle_bool_style(
        &mut self,
        current_val: Option<bool>,
        path: &str,
        address: &Address,
    ) -> Result<(), anyhow::Error> {
        let value = if let Some(b_val) = current_val {
            if b_val {
                "false"
            } else {
                "true"
            }
        } else {
            "true"
        };
        self.book.set_cell_style(
            &[(path, value)],
            &Area {
                sheet: address.sheet,
                row: address.row as i32,
                column: address.col as i32,
                width: 1,
                height: 1,
            },
        )?;
        Ok(())
    }

    fn paste_range(&mut self) -> Result<(), anyhow::Error> {
        match &self.state.clipboard {
            Some(ClipboardContents::Cell(contents)) => {
                self.book.edit_current_cell(contents)?;
                self.book.evaluate();
            }
            Some(ClipboardContents::Range(ref rows)) => {
                let Address { sheet, row, col } = self.book.location.clone();
                for (ri, columns) in rows.iter().enumerate() {
                    for (ci, cell) in columns.iter().enumerate() {
                        self.book.update_cell(
                            &Address {
                                sheet,
                                row: ri + row,
                                col: ci + col,
                            },
                            cell.clone(),
                        )?;
                    }
                }
                self.book.evaluate();
            }
            None => {
                let rows = self.get_rows_from_system_clipboard()?;
                let Address { sheet, row, col } = self.book.location.clone();
                for (ri, columns) in rows.iter().enumerate() {
                    for (ci, cell) in columns.iter().enumerate() {
                        self.book.update_cell(
                            &Address {
                                sheet,
                                row: ri + row,
                                col: ci + col,
                            },
                            cell.clone(),
                        )?;
                    }
                }
                self.book.evaluate();
            }
        }
        self.state.clipboard = None;
        Ok(())
    }

    fn run_with_prefix(
        &mut self,
        action: impl Fn(&mut Workspace<'_>) -> std::result::Result<(), anyhow::Error>,
    ) -> Result<(), anyhow::Error> {
        for _ in 1..=self.state.get_n_prefix() {
            action(self)?;
        }
        self.state.reset_n_prefix();
        Ok(())
    }

    fn enter_quit_mode(&mut self) -> bool {
        if self.book.dirty {
            self.state.modality_stack.push(Modality::Quit);
            return true;
        }
        false
    }

    fn enter_command_mode(&mut self) {
        self.state.modality_stack.push(Modality::Command);
        self.state.command_state.truncate();
        self.state.command_history.reset();
        *self.state.command_state.status_mut() = Status::Pending;
        self.state.command_state.focus();
    }

    /// Opens the `/` or `?` prompt. The direction is the last search's only
    /// once the user presses Enter.
    fn enter_search_mode(&mut self, direction: Direction) {
        self.state.search.prompt = direction;
        self.state.search.preview = self.state.options.incsearch().then(|| search::Preview {
            start: self.book.location.clone(),
            view: self.state.viewport_state.clone(),
            matcher: None,
        });
        self.state.modality_stack.push(Modality::Search);
        self.state.search_prompt.truncate();
        self.state.search_history.reset();
        *self.state.search_prompt.status_mut() = Status::Pending;
        self.state.search_prompt.focus();
    }

    /// Puts a pattern from the search history in the prompt, and previews it.
    fn set_search_text(&mut self, text: String) {
        *self.state.search_prompt.value_mut() = text;
        self.state.search_prompt.move_end();
        self.update_preview();
    }

    /// Enter in the `/` or `?` prompt: searches for the pattern in the
    /// prompt's direction. An empty pattern repeats the last one.
    fn exit_search_mode(&mut self) -> Result<Option<ExitCode>> {
        // Search from where the prompt opened, so the preview's match is the
        // one Enter finds, wrap message and all.
        self.end_preview();
        let pattern = self.state.search_prompt.value().to_owned();
        self.state.search_prompt.blur();
        *self.state.search_prompt.status_mut() = Status::Done;
        self.state.pop_modality();
        if !pattern.is_empty() {
            // Record a pattern that fails too, so a typo can be fixed with Up.
            self.state.search_history.add(&pattern);
            let case = self.case_options();
            if let Err(msg) = self.state.search.set_pattern(&pattern, case) {
                self.set_message(MessageLevel::Error, msg);
                return Ok(None);
            }
        }
        // With no last pattern the direction does not matter: `search_next`
        // says E35 either way.
        let direction = self.state.search.prompt;
        self.state.search.direction = direction;
        self.search_next(direction, 1);
        Ok(None)
    }

    /// Esc in the `/` prompt: leaves it without searching, and puts the
    /// cursor and highlight back as they were before the preview.
    fn cancel_search_mode(&mut self) -> Result<Option<ExitCode>> {
        self.end_preview();
        self.state.search_prompt.blur();
        *self.state.search_prompt.status_mut() = Status::Aborted;
        self.state.pop_modality();
        Ok(None)
    }

    /// With `incsearch`, moves the cursor to the first match of what the
    /// prompt holds, searching from where the prompt opened. Nothing to
    /// match puts it back there. A pattern that does not compile yet is
    /// skipped quietly: the preview stays as it was.
    fn update_preview(&mut self) {
        let case = self.case_options();
        let within = self.state.options.searchin();
        let wrapscan = self.state.options.wrapscan();
        let direction = self.state.search.prompt;
        let Some(preview) = self.state.search.preview.as_mut() else {
            return;
        };
        let pattern = self.state.search_prompt.value();
        preview.matcher = if pattern.is_empty() {
            None
        } else {
            match search::compile(pattern, case) {
                Ok(matcher) => Some(matcher),
                Err(_) => return,
            }
        };
        let start = &preview.start;
        let target = match &preview.matcher {
            Some(matcher) => {
                match self.book.search(start.sheet, matcher, within, start, direction, wrapscan) {
                    Ok(Some(hit)) => hit.address,
                    _ => start.clone(),
                }
            }
            None => start.clone(),
        };
        if target != self.book.location {
            // `move_to` cannot fail; it only sets the row and column.
            let _ = self.book.move_to(&target);
            self.handle_movement_change();
        }
    }

    /// Closes the `incsearch` preview, if there is one, and puts the cursor
    /// and scroll position back where the prompt opened.
    fn end_preview(&mut self) {
        let Some(preview) = self.state.search.preview.take() else {
            return;
        };
        self.state.viewport_state = preview.view;
        if preview.start != self.book.location {
            let _ = self.book.move_to(&preview.start);
            self.handle_movement_change();
        }
    }

    /// Moves to the `count`th next match of the last search, going in
    /// `direction`. A miss or a wrap is reported on the message line; a miss
    /// leaves the cursor where it was.
    fn search_next(&mut self, direction: Direction, count: usize) {
        // Every search, `n` included, shows the highlight `:noh` hid.
        self.state.search.highlight = true;
        let case = self.case_options();
        let (pattern, matcher) = match self.state.search.last(case) {
            Some(Ok(last)) => last,
            Some(Err(msg)) => {
                self.set_message(MessageLevel::Error, msg);
                return;
            }
            None => {
                self.set_message(
                    MessageLevel::Error,
                    "E35: No previous regular expression".to_owned(),
                );
                return;
            }
        };
        let wrapscan = self.state.options.wrapscan();
        let within = self.state.options.searchin();
        let mut from = self.book.location.clone();
        let mut found = Ok(None);
        let mut wrapped = false;
        let mut first = None;
        let mut remaining = count.max(1);
        let mut step = 0;
        while remaining > 0 {
            found = self
                .book
                .search(from.sheet, matcher, within, &from, direction, wrapscan);
            let Ok(Some(hit)) = &found else { break };
            remaining -= 1;
            step += 1;
            wrapped |= hit.wrapped;
            from = hit.address.clone();
            // Back at the first match: the matches repeat every `step - 1`,
            // so a huge count skips the whole laps instead of walking them.
            match &first {
                None => first = Some(hit.address.clone()),
                Some(start) if *start == hit.address && step > 1 => remaining %= step - 1,
                Some(_) => {}
            }
        }
        match found {
            Ok(Some(hit)) => {
                // `move_to` cannot fail; it only sets the row and column.
                let _ = self.book.move_to(&hit.address);
                self.handle_movement_change();
                if wrapped {
                    let text = match direction {
                        Direction::Forward => "search hit BOTTOM, continuing at TOP",
                        Direction::Backward => "search hit TOP, continuing at BOTTOM",
                    };
                    self.set_message(MessageLevel::Info, text.to_owned());
                }
            }
            Ok(None) if !wrapscan => {
                let end = match direction {
                    Direction::Forward => "BOTTOM",
                    Direction::Backward => "TOP",
                };
                let text = format!("E385: Search hit {end} without match for: {pattern}");
                self.set_message(MessageLevel::Error, text);
            }
            Ok(None) => {
                let text = format!("E486: Pattern not found: {pattern}");
                self.set_message(MessageLevel::Error, text);
            }
            Err(e) => self.set_message(MessageLevel::Error, e.to_string()),
        }
    }

    /// `*` and `#`: searches for cells whose whole text, as `searchin` says,
    /// is the current cell's, so `Paid` skips `Unpaid`. The pattern becomes
    /// the last search, so `n` and `N` go on with it.
    fn search_cell(&mut self, direction: Direction, count: usize) {
        let within = self.state.options.searchin();
        let text = match self.book.search_text(&self.book.location, within) {
            Ok(text) => text,
            Err(e) => {
                self.set_message(MessageLevel::Error, e.to_string());
                return;
            }
        };
        if text.is_empty() {
            self.set_message(MessageLevel::Error, "E348: No string under cursor".to_owned());
            return;
        }
        let pattern = format!("^{}$", regex::escape(&text));
        self.state.search_history.add(&pattern);
        let case = self.case_options();
        if let Err(msg) = self.state.search.set_pattern(&pattern, case) {
            self.set_message(MessageLevel::Error, msg);
            return;
        }
        self.state.search.direction = direction;
        self.search_next(direction, count);
    }

    /// The search to highlight and the text it matches: `None` when
    /// `hlsearch` is off, after `:noh`, or before any search. While the
    /// `incsearch` preview has a pattern, it is the one highlighted.
    pub(crate) fn highlight_matcher(&mut self) -> Option<(regex::Regex, book::search::SearchIn)> {
        if !self.state.options.hlsearch() {
            return None;
        }
        let within = self.state.options.searchin();
        if let Some(matcher) = self.state.search.preview.as_ref().and_then(|p| p.matcher.as_ref()) {
            return Some((matcher.clone(), within));
        }
        if !self.state.search.highlight {
            return None;
        }
        let case = self.case_options();
        let (_, matcher) = self.state.search.last(case)?.ok()?;
        Some((matcher.clone(), self.state.options.searchin()))
    }

    /// The case options a pattern is compiled with.
    fn case_options(&self) -> CaseOptions {
        CaseOptions {
            ignorecase: self.state.options.ignorecase(),
            smartcase: self.state.options.smartcase(),
        }
    }

    fn set_message(&mut self, level: MessageLevel, text: String) {
        self.state.message = Some(Message { level, text });
    }

    /// Installs a theme and restyles the text area, which holds its own styles.
    fn set_theme(&mut self, theme: Theme) {
        self.state.theme = theme;
        style_text_area(&mut self.text_area, &self.state.theme);
    }

    /// Applies a loaded theme and reports it on the message line.
    fn apply_theme(&mut self, name: &str, theme: Theme, warnings: Vec<Warning>) {
        self.set_theme(theme);
        let (level, text) = match warnings.len() {
            0 => (MessageLevel::Info, format!("theme {name}")),
            1 => (
                MessageLevel::Error,
                format!("theme {name}: 1 warning (:theme to list)"),
            ),
            n => (
                MessageLevel::Error,
                format!("theme {name}: {n} warnings (:theme to list)"),
            ),
        };
        self.state.theme_warnings = warnings;
        self.set_message(level, text);
    }

    /// `:theme <name>`. An unknown or unreadable theme keeps the current one.
    fn switch_theme(&mut self, name: &str) {
        match lookup::load(name, self.state.themes_dir.as_deref()) {
            Ok((theme, warnings)) => self.apply_theme(name, theme, warnings),
            Err(msg) => self.set_message(MessageLevel::Error, msg),
        }
    }

    /// Chooses the startup theme from the flag, then the config file, then the
    /// default. Never fails: a bad theme falls back to `default` with a message.
    pub fn init_theme(&mut self, flag: Option<&str>) {
        self.init_theme_from(flag, lookup::config_dir());
    }

    fn init_theme_from(&mut self, flag: Option<&str>, config_dir: Option<PathBuf>) {
        let config = config_dir.as_deref().and_then(lookup::read_config);
        let themes_dir = config_dir.map(|d| d.join("themes"));
        let (theme, warnings) = lookup::resolve(flag, config.as_deref(), themes_dir.as_deref());
        self.state.themes_dir = themes_dir;
        let name = flag.or(config.as_deref()).unwrap_or(&theme.name).to_owned();
        if warnings.is_empty() && flag.is_none() && config.is_none() {
            self.set_theme(theme);
        } else {
            self.apply_theme(&name, theme, warnings);
        }
    }

    /// Loads the keys file from `--keys`, else the config directory. Never
    /// fails: a problem is a warning, counted on the message line and listed by
    /// `:nmap`.
    pub fn init_keys(&mut self, flag: Option<&Path>) {
        self.init_keys_from(flag, lookup::config_dir());
    }

    fn init_keys_from(&mut self, flag: Option<&Path>, config_dir: Option<PathBuf>) {
        self.state.keymap.nav = Default::default();
        self.state.keymap.range = Default::default();
        let loaded = keys::file::load_file(flag, config_dir.as_deref(), &mut self.state.keymap);
        let count = loaded.warnings.len();
        self.state.keymap.loaded_from = loaded.path;
        self.state.keymap.warnings = loaded.warnings;
        if count > 0 {
            let plural = if count == 1 { "" } else { "s" };
            self.set_message(
                MessageLevel::Error,
                format!("keys: {count} warning{plural} (:nmap to list)"),
            );
        }
    }

    /// The `:theme` dialog: available themes, then the last load's warnings.
    fn theme_listing(&self) -> Markdown {
        let mut text = String::from("# Themes\n\n");
        for (name, source) in lookup::list(self.state.themes_dir.as_deref()) {
            let kind = match source {
                lookup::Source::Builtin => "built-in",
                lookup::Source::User => "user",
            };
            let current = if name == self.state.theme.name { ", current" } else { "" };
            text.push_str(&format!("* {name} ({kind}{current})\n"));
        }
        if !self.state.theme_warnings.is_empty() {
            text.push_str("\n## Warnings\n\n");
            for w in &self.state.theme_warnings {
                if w.line == 0 {
                    text.push_str(&format!("* {}\n", w.msg));
                } else {
                    text.push_str(&format!("* line {}: {}\n", w.line, w.msg));
                }
            }
        }
        text.push_str("\nType `theme <name>` to switch.\n");
        Markdown::from_str_themed(&text, &self.state.theme)
    }

    fn enter_dialog_mode(&mut self, msg: Markdown) {
        self.state.popup = Some(msg);
        self.state.modality_stack.push(Modality::Dialog);
    }

    fn enter_range_select_mode(&mut self, init_start: bool) {
        self.state.range_select.original_location = Some(self.book.location.clone());
        if init_start {
            self.state.range_select.start = Some(self.book.location.clone());
        } else {
            self.state.range_select.start = None;
        }
        self.state.range_select.end = None;
        self.state.modality_stack.push(Modality::RangeSelect);
    }

    fn enter_edit_mode(&mut self) {
        self.state.modality_stack.push(Modality::CellEdit);
        self.text_area
            .set_cursor_line_style(Style::default().add_modifier(Modifier::UNDERLINED));
        self.text_area
            .set_cursor_style(Style::default().add_modifier(Modifier::SLOW_BLINK));
        self.text_area.move_cursor(CursorMove::Bottom);
        self.text_area.move_cursor(CursorMove::End);
    }

    fn exit_quit_mode(&mut self) -> Result<Option<ExitCode>> {
        self.state.pop_modality();
        Ok(None)
    }

    fn set_command_text(&mut self, text: String) {
        *self.state.command_state.value_mut() = text;
        self.state.command_state.move_end();
    }

    fn exit_command_mode(&mut self) -> Result<Option<ExitCode>> {
        let cmd = self.state.command_state.value().to_owned();
        // Record a command that fails too, so a typo can be fixed with Up.
        self.state.command_history.add(&cmd);
        self.state.command_state.blur();
        *self.state.command_state.status_mut() = Status::Done;
        self.state.pop_modality();
        self.handle_command(cmd)
    }

    /// Leave Command mode without running the command.
    fn cancel_command_mode(&mut self) -> Result<Option<ExitCode>> {
        self.state.command_state.blur();
        *self.state.command_state.status_mut() = Status::Aborted;
        self.state.pop_modality();
        Ok(None)
    }

    fn exit_dialog_mode(&mut self) -> Result<()> {
        self.state.pop_modality();
        Ok(())
    }

    fn exit_range_select_mode(&mut self) -> Result<()> {
        self.book.location = self
            .state
            .range_select
            .original_location
            .clone()
            .expect("Missing original location after range copy");
        self.state.range_select.original_location = None;
        self.state.pop_modality();
        if self.state.modality() == &Modality::CellEdit {
            self.text_area
                .set_yank_text(self.selected_range_to_string());
            self.text_area.paste();
            self.state.dirty = true;
        }
        Ok(())
    }

    fn exit_edit_mode(&mut self, keep: bool) -> Result<()> {
        self.text_area.set_cursor_line_style(Style::default());
        self.text_area.set_cursor_style(Style::default());
        let contents = self.text_area.lines().join("\n");
        if self.state.dirty && keep {
            self.book.edit_current_cell(contents)?;
            self.book.evaluate();
        }
        self.text_area = reset_text_area(self.book.get_current_cell_contents()?, &self.state.theme);
        self.state.dirty = false;
        self.state.pop_modality();
        Ok(())
    }

    fn handle_movement_change(&mut self) {
        let contents = self
            .book
            .get_current_cell_contents()
            .expect("Unexpected failure getting current cell contents");
        self.text_area = reset_text_area(contents, &self.state.theme);
    }

    fn save_file(&mut self) -> Result<()> {
        // If a file_path was previously set (i.e. the book was loaded from or
        // saved-as a concrete path), use save() to update it in place.
        // Otherwise fall back to save_as(&self.name) which also sets file_path
        // so that subsequent `:w` calls use save() directly.
        if self.book.get_file_path().is_some() {
            self.book.save()?;
        } else {
            self.book.save_as(&self.name)?;
        }
        Ok(())
    }

    fn save_to<S: Into<String>>(&mut self, path: S) -> Result<()> {
        self.book.save_as(path.into().as_str())?;
        Ok(())
    }

    fn quit_app(&mut self) -> std::result::Result<Option<ExitCode>, anyhow::Error> {
        if self.enter_quit_mode() {
            return Ok(None);
        }
        Ok(Some(ExitCode::SUCCESS))
    }
}

fn load_book(path: &Path, locale: &str, tz: &str) -> Result<Book, anyhow::Error> {
    let book = if path.exists() {
        Book::load(path, locale, tz)?
    } else {
        Book::default()
    };
    Ok(book)
}

/// Applies the `edit` and `border` slots to the cell edit box.
fn style_text_area(text_area: &mut TextArea, theme: &Theme) {
    text_area.set_style(theme.get(Slot::Edit));
    text_area.set_block(Block::bordered().border_style(theme.get(Slot::Border)));
}

fn reset_text_area<'a>(content: String, theme: &Theme) -> TextArea<'a> {
    let mut text_area = TextArea::from(content.lines());
    text_area.set_cursor_line_style(Style::default());
    text_area.set_cursor_style(Style::default());
    style_text_area(&mut text_area, theme);
    text_area
}
