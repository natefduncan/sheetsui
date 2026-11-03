use ratatui::{
    self,
    layout::Rect,
    text::{Line, Text},
    widgets::{Block, Paragraph, Tabs, Widget},
    Frame,
};

use super::*;
use crate::ui::theme::Slot;

pub mod viewport;
pub use viewport::Viewport;
pub mod cell_class;
pub mod dialog;
pub mod markdown;

#[cfg(test)]
mod test;

/// Draws one piece of the frame into its own rect.
type RenderFn<'ws> = Box<dyn Fn(Rect, &mut Buffer, &mut Workspace<'ws>)>;

/// A rect paired with the function that draws it.
type RenderPart<'ws> = (Rect, RenderFn<'ws>);

impl<'ws> Workspace<'ws> {
    fn get_render_parts(&mut self, area: Rect) -> Vec<RenderPart<'ws>> {
        use ratatui::widgets::StatefulWidget;
        let mut cs = vec![
            Constraint::Length(2),
            Constraint::Length(3),
            Constraint::Fill(1),
        ];
        let mut rs: Vec<RenderFn<'ws>> = vec![
            Box::new(|rect: Rect, buf: &mut Buffer, ws: &mut Self| {
                let tabs = Tabs::new(
                    ws.book
                        .get_sheet_names()
                        .iter()
                        .enumerate()
                        .map(|(idx, name)| format!("{} {}", name, idx))
                        .collect::<Vec<String>>(),
                )
                .style(ws.state.theme.get(Slot::Tab))
                .highlight_style(ws.state.theme.get(Slot::TabActive))
                .select(Some(ws.book.location.sheet as usize));
                tabs.render(rect, buf);
            }),
            Box::new(|rect: Rect, buf: &mut Buffer, ws: &mut Self| {
                let [text_rect, info_rect] =
                    Layout::horizontal(vec![Constraint::Fill(1), Constraint::Fill(1)]).areas(rect);
                ws.text_area.render(text_rect, buf);
                let hint = Paragraph::new(vec![
                    Line::from(""),
                    Line::from("ALT-h to toggle help dialog").centered(),
                ])
                .style(ws.state.theme.get(Slot::Hint));
                hint.render(info_rect, buf);
            }),
            Box::new(move |rect: Rect, buf: &mut Buffer, ws: &mut Self| {
                let sheet_name = ws.book.get_sheet_name().unwrap_or("Unknown");
                let theme = &ws.state.theme;
                let table_block = Block::bordered()
                    .border_style(theme.get(Slot::Border))
                    .title_top(Line::styled(sheet_name, theme.get(Slot::Title)));
                let viewport = Viewport::new(
                    &ws.book,
                    if ws.state.modality() == &Modality::RangeSelect {
                        Some(&ws.state.range_select)
                    } else {
                        None
                    },
                    &ws.state.theme,
                )
                .with_selected(ws.book.location.clone())
                .block(table_block);
                StatefulWidget::render(viewport, rect, buf, &mut ws.state.viewport_state);
            }),
        ];

        if self.state.modality() == &Modality::Command {
            cs.push(Constraint::Max(1));
            rs.push(Box::new(|rect: Rect, buf: &mut Buffer, ws: &mut Self| {
                StatefulWidget::render(
                    TextPrompt::from("Command"),
                    rect,
                    buf,
                    &mut ws.state.command_state,
                );
                // TextPrompt has no style API, so style its rect afterwards.
                buf.set_style(rect, ws.state.theme.get(Slot::Command));
            }));
        }
        let rects: Vec<Rect> = Vec::from(
            Layout::vertical(cs)
                .vertical_margin(2)
                .horizontal_margin(2)
                .flex(Flex::Legacy)
                .split(area)
                .as_ref(),
        );
        rects
            .into_iter()
            .zip(rs)
            .collect()
    }
}

impl<'widget, 'ws: 'widget> Widget for &'widget mut Workspace<'ws> {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        if self.state.modality() == &Modality::Dialog {
            let lines = self
                .state
                .popup
                .as_ref()
                .map(|md| md.get_text())
                .unwrap_or_else(|| Text::raw("Popup message here"));
            let popup = dialog::Dialog::new(lines, "Help", &self.state.theme).scroll(self.state.dialog_scroll);
            popup.render(area, buf);
        } else if self.state.modality() == &Modality::Quit {
            let popup =
                dialog::Dialog::new(
                    Text::raw("File is not yet saved. Save it first?"),
                    "Quit",
                    &self.state.theme,
                )
                    .with_bottom_title("Y/N");
            popup.render(area, buf);
        } else {
            let theme = &self.state.theme;
            let status = theme.get(Slot::Status);
            let mut outer_block = Block::bordered()
                .border_style(theme.get(Slot::Border))
                .title(Line::styled(
                    self.name
                        .file_name()
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_else(|| String::from("Unknown")),
                    theme.get(Slot::Title),
                ));
            if let Some(m) = &self.state.message {
                let slot = match m.level {
                    MessageLevel::Info => Slot::Info,
                    MessageLevel::Error => Slot::Error,
                };
                outer_block = outer_block.title_bottom(Line::styled(
                    format!(" {} ", m.text),
                    self.state.theme.get(slot),
                ));
            }
            if !self.state.pending_keys.is_empty() {
                outer_block = outer_block.title_bottom(Line::styled(
                    format!(" {} ", crate::ui::keys::format_keys(&self.state.pending_keys)),
                    status,
                ));
            }
            let outer_block = outer_block
                .title_bottom(Line::styled(
                    match self.state.modality() {
                        Modality::Navigate => "navigate",
                        Modality::CellEdit => "edit",
                        Modality::Command => "command",
                        Modality::Dialog => "",
                        Modality::RangeSelect => "range-copy",
                        Modality::Quit => "",
                    },
                    status,
                ))
                .title_bottom(
                    Line::styled(
                        format!("{},{}", self.book.location.row, self.book.location.col),
                        status,
                    )
                    .right_aligned(),
                );

            for (rect, f) in self.get_render_parts(area) {
                f(rect, buf, self);
            }

            outer_block.render(area, buf);
        }
    }
}

pub fn draw(frame: &mut Frame, ws: &mut Workspace) {
    frame.render_widget(&mut *ws, frame.area());
    // The prompt is drawn into a buffer, so only the frame can show the
    // terminal cursor. Rendering the prompt set its position.
    if ws.state.modality() == &Modality::Command {
        frame.set_cursor_position(ws.state.command_state.cursor());
    }
}
