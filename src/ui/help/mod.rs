use crate::ui::{render::markdown::Markdown, theme::Theme};

pub fn to_widget(topic: &str, theme: &Theme) -> Markdown {
    match topic {
        "navigate" => Markdown::from_str_themed(include_str!("../../../docs/navigation.md"), theme),
        "edit" => Markdown::from_str_themed(include_str!("../../../docs/edit.md"), theme),
        "command" => Markdown::from_str_themed(include_str!("../../../docs/command.md"), theme),
        "visual" => Markdown::from_str_themed(include_str!("../../../docs/visual.md"), theme),
        _ => Markdown::from_str_themed(include_str!("../../../docs/intro.md"), theme),
    }
}
