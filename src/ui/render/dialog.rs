use crate::ui::theme::{Slot, Theme};
use ratatui::{
    self,
    layout::{Constraint, Layout, Rect},
    text::{Line, Text},
    widgets::{Block, Clear, Paragraph, Widget, Wrap},
};

pub struct Dialog<'w> {
    content: Text<'w>,
    title: &'w str,
    bottom_title: &'w str,
    scroll: (u16, u16),
    theme: &'w Theme,
    // TODO(zaphar): Have a max margin?
}

impl<'w> Dialog<'w> {
    pub fn new(content: Text<'w>, title: &'w str, theme: &'w Theme) -> Self {
        Self {
            theme,
            content,
            title,
            bottom_title: "j,k or up,down to scroll",
            scroll: (0, 0),
        }
    }

    pub fn with_bottom_title(mut self, title: &'w str) -> Self {
        self.bottom_title = title;
        self
    }
    pub fn scroll(mut self, line: u16) -> Self {
        self.scroll.0 = line;
        self
    }
}

impl<'w> Widget for Dialog<'w> {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        // First find the center of the area.
        let content_width = 120 + 2;
        let content_height = (self.content.height() + 2) as u16;
        let vertical_margin = if content_height <= area.height {
            area.height
                .saturating_sub(content_height)
                .saturating_div(2)
        } else {
            2
        };
        let horizontal_margin = if content_width <= area.width {
            area
            .width
            .saturating_sub(content_width)
            .saturating_div(2)
        } else {
           2
        };
        let [_, dialog_vertical, _] = Layout::vertical(vec![
            Constraint::Length(vertical_margin),
            Constraint::Fill(1),
            Constraint::Length(vertical_margin),
        ])
        .areas(area);
        let [_, dialog_area, _] = Layout::horizontal(vec![
            Constraint::Length(horizontal_margin),
            Constraint::Fill(1),
            Constraint::Length(horizontal_margin),
        ])
        .areas(dialog_vertical);

        Clear.render(dialog_area, buf);
        let dialog_block = Block::bordered()
            .title_top(Line::styled(self.title, self.theme.get(Slot::Title)))
            .title_bottom(self.bottom_title)
            .style(self.theme.get(Slot::Dialog))
            .border_style(self.theme.get(Slot::DialogBorder));
        let dialog = Paragraph::new(self.content.clone())
            .wrap(Wrap::default())
            .scroll(self.scroll)
            .block(dialog_block)
            ;
        dialog.render(dialog_area, buf);
    }
}
