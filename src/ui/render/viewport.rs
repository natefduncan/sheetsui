use std::collections::HashSet;

use anyhow::Result;
use regex::Regex;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Flex, Rect},
    style::Stylize,
    text::{Line, Text},
    widgets::{Block, Cell, Row, StatefulWidget, Table, Widget},
};

use super::{
    cell_class::{cell_style, classify, search_style},
    Address, Book, RangeSelection,
};
use crate::{
    book::{self, search::SearchIn},
    ui::theme::{color::parse_workbook_color, Slot, Theme},
};

/// A visible column to show in our Viewport.
#[derive(Clone, Debug)]
pub struct VisibleColumn {
    pub idx: usize,
    pub length: u16,
}

impl<'a> From<&'a VisibleColumn> for Constraint {
    fn from(value: &'a VisibleColumn) -> Self {
        Constraint::Length(value.length)
    }
}

#[derive(Debug, Default, Clone)]
pub struct ViewportState {
    prev_corner: Address,
    /// True when the last render shrank the freeze to fit the screen.
    pub(crate) freeze_clipped: bool,
    /// True once the clip message was shown, so it is not shown every frame.
    pub(crate) freeze_clip_reported: bool,
}

/// A renderable viewport over a book.
pub struct Viewport<'ws> {
    pub(crate) selected: Address,
    book: &'ws Book,
    range_selection: Option<&'ws RangeSelection>,
    block: Option<Block<'ws>>,
    theme: &'ws Theme,
    /// The search to highlight, if `hlsearch` is on and there is one.
    search: Option<(&'ws Regex, SearchIn)>,
}

pub(crate) const COLNAMES: [&str; 26] = [
    "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O", "P", "Q", "R", "S",
    "T", "U", "V", "W", "X", "Y", "Z",
];

impl<'ws> Viewport<'ws> {
    pub fn new(
        book: &'ws Book,
        app_state: Option<&'ws RangeSelection>,
        theme: &'ws Theme,
    ) -> Self {
        Self {
            theme,
            book,
            range_selection: app_state,
            selected: Default::default(),
            block: None,
            search: None,
        }
    }

    /// Highlights the cells that `matcher` matches, reading what `within`
    /// says.
    pub fn with_search(mut self, search: Option<(&'ws Regex, SearchIn)>) -> Self {
        self.search = search;
        self
    }

    pub fn with_selected(mut self, location: Address) -> Self {
        self.selected = location;
        self
    }

    pub(crate) fn get_visible_rows(&self, height: u16, state: &ViewportState) -> Vec<usize> {
        // NOTE(jeremy): For now the row default height is 1. We'll have
        // to adjust that if this changes.
        let frozen = self.clipped_frozen_rows(height);
        // The frozen rows take screen rows, and so does the line under them.
        let line = (frozen > 0) as u16;
        let height = height.saturating_sub(frozen as u16 + line).max(2);
        let mut length = 1;
        // `prev_corner` is the first scrolling row, never above the freeze.
        let start_row = std::cmp::max(
            frozen + 1,
            std::cmp::min(self.selected.row, state.prev_corner.row),
        );
        let mut start = start_row;
        let mut end = start_row;
        for row_idx in start_row..=(book::LAST_ROW as usize) {
            let updated_length = length + 1;
            if updated_length <= height {
                length = updated_length;
                end = row_idx;
            } else if self.selected.row >= row_idx {
                start += 1;
                end = row_idx;
            } else {
                break;
            }
        }
        (1..=frozen).chain(start..=end).collect()
    }

    /// The frozen rows that fit, leaving the header, the boundary line and one
    /// row to scroll.
    fn clipped_frozen_rows(&self, height: u16) -> usize {
        let stored = self.book.frozen_rows(self.book.location.sheet);
        let room = if stored > 0 { 3 } else { 2 };
        stored.min(height.saturating_sub(room) as usize)
    }

    pub(crate) fn get_visible_columns(
        &self,
        width: u16,
        state: &ViewportState,
    ) -> Result<Vec<VisibleColumn>> {
        Ok(self.visible_columns_split(width, state)?.1)
    }

    /// The visible columns, and how many of the leading ones are frozen.
    fn visible_columns_split(
        &self,
        width: u16,
        state: &ViewportState,
    ) -> Result<(usize, Vec<VisibleColumn>)> {
        // TODO(zaphar): This should be a shared constant with our first column.
        // We start out with a length of 5 already reserved
        let mut length = 5;
        // Pin the frozen columns, but keep room for one column to scroll in.
        // The boundary line takes one column while any column is frozen.
        let stored = self.book.frozen_columns(self.book.location.sheet);
        if stored > 0 {
            length += 1;
        }
        let mut frozen = Vec::new();
        for idx in 1..=stored {
            let size = self.book.get_col_size(idx)? as u16;
            let next = self.book.get_col_size(idx + 1)? as u16;
            if length + size + next >= width {
                break;
            }
            length += size;
            frozen.push(VisibleColumn { idx, length: size });
        }
        let nfrozen = frozen.len();
        if nfrozen == 0 && stored > 0 {
            // Clipped away entirely: no line either.
            length -= 1;
        }
        let mut visible = Vec::new();
        // `prev_corner` is the first scrolling column, never left of the freeze.
        let start_idx = std::cmp::max(
            nfrozen + 1,
            std::cmp::min(self.selected.col, state.prev_corner.col),
        );
        for idx in start_idx..=(book::LAST_COLUMN as usize) {
            let size = self.book.get_col_size(idx)? as u16;
            let updated_length = length + size;
            let col = VisibleColumn { idx, length: size };
            if updated_length < width {
                length = updated_length;
                visible.push(col);
            } else if self.selected.col >= col.idx {
                // We need a sliding window now
                if let Some(first) = visible.first() {
                    // subtract the first columns size.
                    length -= first.length;
                    // remove the first column.
                    // TODO(jwall): This is a bit inefficient. Can we do better?
                    visible = visible.into_iter().skip(1).collect();
                }
                // Add this col to the visible.
                length += size;
                visible.push(col);
                // What if the length is still too long?
                if length > width {
                    if let Some(first) = visible.first() {
                        // subtract the first columns size.
                        length -= first.length;
                    }
                    visible = visible.into_iter().skip(1).collect();
                }
            } else {
                break;
            }
        }
        frozen.extend(visible);
        Ok((nfrozen, frozen))
    }

    pub fn block(mut self, block: Block<'ws>) -> Self {
        self.block = Some(block);
        self
    }

    pub(crate) fn to_table<'widget>(
        &self,
        width: u16,
        height: u16,
        state: &mut ViewportState,
    ) -> Result<Table<'widget>> {
        let (frozen_cols, visible_columns) = self.visible_columns_split(width, state)?;
        let visible_rows = self.get_visible_rows(height, state);
        if let Some(vc) = visible_columns.get(frozen_cols) {
            state.prev_corner.col = vc.idx
        }
        let sheet = self.book.location.sheet;
        let frozen_rows = self.clipped_frozen_rows(height);
        state.freeze_clipped = frozen_rows < self.book.frozen_rows(sheet)
            || frozen_cols < self.book.frozen_columns(sheet);
        if let Some(vr) = visible_rows.get(frozen_rows) {
            state.prev_corner.row = *vr;
        }
        let matches = self.visible_matches(&visible_rows, &visible_columns);
        let line_style = self.theme.get(Slot::FreezeLine);
        // The boundary line: a thin column after the frozen columns and a
        // thin row after the frozen rows. Only drawn while a freeze shows.
        let vline = |text: &'static str| Cell::new(text).style(line_style);
        let with_vline = |mut cells: Vec<Cell<'widget>>, glyph: &'static str| {
            if frozen_cols > 0 {
                cells.insert(1 + frozen_cols, vline(glyph));
            }
            cells
        };
        let mut rows: Vec<Row> = Vec::with_capacity(visible_rows.len() + 1);
        for (pos, ri) in visible_rows.iter().copied().enumerate() {
            let mut cells = vec![Cell::new(Text::from(ri.to_string()))
                .style(self.theme.get(Slot::HeaderRow))];
            cells.extend(visible_columns.iter().map(
                |VisibleColumn { idx: ci, length: _ }| {
                    let content = self
                        .book
                        .get_cell_addr_rendered(&Address {
                            row: ri,
                            col: *ci,
                            sheet: self.book.location.sheet,
                        })
                        .unwrap();
                    let is_match = matches.contains(&(ri, *ci));
                    self.compute_cell_style(ri, *ci, is_match, Cell::new(Text::raw(content)))
                },
            ));
            rows.push(Row::new(with_vline(cells, "│")));
            if frozen_rows > 0 && pos + 1 == frozen_rows {
                let mut rule = vec![Cell::new("─".repeat(5)).style(line_style)];
                rule.extend(
                    visible_columns
                        .iter()
                        .map(|c| Cell::new("─".repeat(c.length as usize)).style(line_style)),
                );
                rows.push(Row::new(with_vline(rule, "┼")));
            }
        }
        let constraints: Vec<Constraint> = visible_columns
            .iter()
            .map(Constraint::from)
            .collect();
        let mut header = Vec::with_capacity(constraints.len());
        header.push(Cell::new("").style(self.theme.get(Slot::HeaderRow)));
        header.extend(visible_columns.iter().map(|c| c.idx).map(|i| {
            let count = if i == 26 { 1 } else { (i / 26) + 1 };
            let even = i % 2 == 0;
            Cell::new(Line::raw(COLNAMES[(i - 1) % 26].repeat(count)).centered())
                .style(self.theme.get(if even {
                    Slot::HeaderEven
                } else {
                    Slot::HeaderOdd
                }))
        }));
        let header = with_vline(header, "│");
        let mut col_constraints = vec![Constraint::Length(5)];
        col_constraints.extend(constraints);
        if frozen_cols > 0 {
            col_constraints.insert(1 + frozen_cols, Constraint::Length(1));
        }
        Ok(Table::new(rows, col_constraints)
            .header(Row::new(header))
            .column_spacing(0)
            .flex(Flex::Start))
    }

    /// The visible cells that match the search, row and column. Only the
    /// visible rectangle is checked, never the whole sheet.
    fn visible_matches(&self, rows: &[usize], cols: &[VisibleColumn]) -> HashSet<(usize, usize)> {
        let Some((matcher, within)) = self.search else {
            return HashSet::new();
        };
        let (Some(left), Some(right)) = (cols.first(), cols.last()) else {
            return HashSet::new();
        };
        // The rows are runs: the frozen rows, then the scrolling window.
        // Search each run, never the rows hidden between them.
        let mut found = HashSet::new();
        let mut rest = rows;
        while let Some(&top) = rest.first() {
            let run = 1 + rest.windows(2).take_while(|w| w[1] == w[0] + 1).count();
            let bottom = rest[run - 1];
            found.extend(
                self.book
                    .matches_in(
                        self.book.location.sheet,
                        matcher,
                        within,
                        top..=bottom,
                        left.idx..=right.idx,
                    )
                    .unwrap_or_default(),
            );
            rest = &rest[run..];
        }
        found
    }

    fn compute_cell_style<'widget>(
        &self,
        ri: usize,
        ci: usize,
        is_match: bool,
        mut cell: Cell<'widget>,
    ) -> Cell<'widget> {
        // TODO(zaphar): Should probably create somekind of formatter abstraction.
        if let Some(style) = self.book.get_cell_style(&Address {
            sheet: self.book.location.sheet,
            row: ri,
            col: ci,
        }) {
            cell = self.compute_cell_colors(&style, ri, ci, is_match, cell);
            cell = if style.font.b { cell.bold() } else { cell };
            cell = if style.font.i { cell.italic() } else { cell };
        }
        cell
    }

    fn compute_cell_colors<'widget>(
        &self,
        style: &ironcalc::base::types::Style,
        ri: usize,
        ci: usize,
        is_match: bool,
        cell: Cell<'widget>,
    ) -> Cell<'widget> {
        let workbook_theme = &self.book.model.get_model().workbook.theme;
        let file_color = |color: &ironcalc::base::types::Color| {
            style_color_hex(color, workbook_theme).and_then(|hex| parse_workbook_color(&hex))
        };
        let addr = Address {
            sheet: self.book.location.sheet,
            row: ri,
            col: ci,
        };
        let class = match self.book.get_cell_class_inputs(&addr) {
            Some((value, ty, num_fmt)) => classify(&value, ty, &num_fmt),
            None => Slot::Cell,
        };
        let in_selection = match self.range_selection.and_then(|r| r.get_range()) {
            Some((start, end)) => {
                ri >= start.row && ri <= end.row && ci >= start.col && ci <= end.col
            }
            None => false,
        };
        let is_cursor = self.book.location.row == ri && self.book.location.col == ci;
        // TODO(zaphar): Support ironcalc style options
        let style = cell_style(
            self.theme,
            file_color(&style.font.color),
            file_color(&style.fill.color),
            class,
            in_selection,
            is_cursor,
        );
        cell.style(search_style(style, self.theme, is_match, is_cursor))
    }
}

/// Resolves an ironcalc style color to a `#RRGGBB` string, or `None` when the
/// style sets no color. A theme color resolves through the workbook theme.
fn style_color_hex(
    color: &ironcalc::base::types::Color,
    theme: &ironcalc::base::types::Theme,
) -> Option<String> {
    match color {
        ironcalc::base::types::Color::None => None,
        other => Some(other.to_rgb(theme)),
    }
}

#[cfg(test)]
pub(crate) fn map_color(
    color: Option<&String>,
    otherwise: ratatui::style::Color,
) -> ratatui::style::Color {
    color
        .and_then(|s| parse_workbook_color(s))
        .unwrap_or(otherwise)
}

impl<'ws> StatefulWidget for Viewport<'ws> {
    type State = ViewportState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // The block surrounding this table adds 2 additional rows and columns
        // to the available rect for rendering this table.
        let mut table = self
            .to_table(area.width - 2, area.height - 2, state)
            .expect("Failed to turn viewport into a table.");
        if let Some(block) = self.block {
            table = table.block(block);
        }
        Widget::render(table, area, buf);
    }
}
