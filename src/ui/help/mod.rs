use crate::ui::{
    keys::{
        help::expand,
        map::{KeyMap, MapMode},
    },
    render::markdown::Markdown,
    theme::Theme,
};

pub fn to_widget(topic: &str, theme: &Theme, keymap: &KeyMap) -> Markdown {
    match topic {
        "navigate" => {
            let text = expand(
                include_str!("../../../docs/navigation.md"),
                MapMode::Navigate,
                keymap,
            );
            Markdown::from_str_themed(&text, theme)
        }
        "edit" => Markdown::from_str_themed(include_str!("../../../docs/edit.md"), theme),
        "command" => Markdown::from_str_themed(include_str!("../../../docs/command.md"), theme),
        "themes" => Markdown::from_str_themed(include_str!("../../../docs/themes.md"), theme),
        "visual" => {
            let text = expand(
                include_str!("../../../docs/visual.md"),
                MapMode::RangeSelect,
                keymap,
            );
            Markdown::from_str_themed(&text, theme)
        }
        _ => Markdown::from_str_themed(include_str!("../../../docs/intro.md"), theme),
    }
}
