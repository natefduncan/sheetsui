//! Finding cells whose text matches a pattern.
//!
//! A search visits the cells of one sheet that have content, in row-major
//! order: left to right across a row, then the next row down. It has no UI in
//! it, so it is tested on a book alone. See `planning/search-design.md`.
use anyhow::{anyhow, Result};
use ironcalc::base::{expressions::utils::parse_reference_a1, types::Cell};
use regex::Regex;

use super::Book;
use crate::ui::Address;

/// Which way a search goes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Direction {
    #[default]
    Forward,
    Backward,
}

/// Which of a cell's two strings a search matches: the `searchin` option.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SearchIn {
    /// The displayed value, as the sheet draws it: `$12.00`, `50%`, or a
    /// formula's result.
    #[default]
    Value,
    /// The raw contents, as CellEdit shows them: `12`, `0.5`, `=A1*2`.
    Raw,
}

/// A cell that matched.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hit {
    pub address: Address,
    /// The search went past the end of the sheet and started again at the
    /// other end.
    pub wrapped: bool,
}

impl Book {
    /// Finds the next cell of `sheet` after `from` whose text, as `within`
    /// says, matches `matcher`, going in `direction`. `from` itself is only
    /// visited last, after a wrap. Without `wrapscan` the search stops at the
    /// end of the sheet.
    pub fn search(
        &self,
        sheet: u32,
        matcher: &Regex,
        within: SearchIn,
        from: &Address,
        direction: Direction,
        wrapscan: bool,
    ) -> Result<Option<Hit>> {
        let cells = self.searchable_cells(sheet)?;
        let from = (from.row as i32, from.col as i32);
        let is_match = |&(row, col): &(i32, i32)| {
            let text = match within {
                SearchIn::Value => self.model.get_formatted_cell_value(sheet, row, col),
                SearchIn::Raw => self.model.get_cell_content(sheet, row, col),
            };
            text.is_ok_and(|text| matcher.is_match(&text))
        };
        // `ahead` runs from the cursor to the end of the sheet, `behind` from
        // the other end back to the cursor, which is visited last.
        let found = match direction {
            Direction::Forward => {
                let (behind, ahead) = cells.split_at(cells.partition_point(|c| *c <= from));
                match ahead.iter().find(|c| is_match(c)) {
                    Some(cell) => Some((cell, false)),
                    None if wrapscan => behind.iter().find(|c| is_match(c)).map(|c| (c, true)),
                    None => None,
                }
            }
            Direction::Backward => {
                let (ahead, behind) = cells.split_at(cells.partition_point(|c| *c < from));
                match ahead.iter().rev().find(|c| is_match(c)) {
                    Some(cell) => Some((cell, false)),
                    None if wrapscan => {
                        behind.iter().rev().find(|c| is_match(c)).map(|c| (c, true))
                    }
                    None => None,
                }
            }
        };
        Ok(found.map(|(&(row, col), wrapped)| Hit {
            address: Address {
                sheet,
                row: row as usize,
                col: col as usize,
            },
            wrapped,
        }))
    }

    /// The cells of `sheet` a search visits, in row-major order: those with
    /// content, except the hidden part of a merged cell and the values an
    /// array formula spills, which belong to the cell at their top left.
    fn searchable_cells(&self, sheet: u32) -> Result<Vec<(i32, i32)>> {
        let worksheet = self
            .model
            .get_model()
            .workbook
            .worksheet(sheet)
            .map_err(|e| anyhow!(e))?;
        let merged: Vec<_> = worksheet
            .merge_cells
            .iter()
            .filter_map(|range| {
                let mut ends = range.split(':').filter_map(parse_reference_a1);
                Some((ends.next()?, ends.next()?))
            })
            .collect();
        let hidden = |row: i32, col: i32| {
            merged.iter().any(|(start, end)| {
                (start.row..=end.row).contains(&row)
                    && (start.column..=end.column).contains(&col)
                    && (row, col) != (start.row, start.column)
            })
        };
        let mut cells: Vec<(i32, i32)> = worksheet
            .sheet_data
            .iter()
            .flat_map(|(row, cols)| {
                cols.iter()
                    .filter(|(_, cell)| {
                        !matches!(cell, Cell::EmptyCell { .. } | Cell::SpillCell { .. })
                    })
                    .map(move |(col, _)| (*row, *col))
            })
            .filter(|(row, col)| !hidden(*row, *col))
            .collect();
        cells.sort_unstable();
        Ok(cells)
    }
}
