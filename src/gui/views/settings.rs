use crate::app::Message;
use crate::config::Config;
use crate::style::{self, Palette};
use iced::widget::{button, column, container, row, text, text_input};
use iced::{Element, Length};

pub fn settings_view<'a>(
    config: &'a Config,
    palette: Palette,
    needs_lutris_path: bool,
) -> Element<'a, Message> {
    let lutris_path_value = config
        .lutris_database_path
        .as_ref()
        .map(|p| p.display().to_string())
        .unwrap_or_default();
    let icons_path_value = config
        .lutris_icons_path
        .as_ref()
        .map(|p| p.display().to_string())
        .unwrap_or_default();

    let mut content = column![text("Settings").size(30).color(palette.text)].spacing(20);

    if needs_lutris_path {
        content = content.push(
            text("Lutris database not found. Please locate your pga.db file.")
                .color(palette.accent),
        );
    }

    content = content
        .push(text("SteamGridDB API Key:").color(palette.text_muted))
        .push(
            text_input(
                "Paste your API key here",
                config.steamgriddb_api_key.as_deref().unwrap_or(""),
            )
            .on_input(Message::ApiKeyChanged)
            .secure(true)
            .style(style::text_input(palette)),
        )
        .push(text("Lutris pga.db location:").color(palette.text_muted))
        .push(
            row![
                text_input("/home/user/.local/share/lutris/pga.db", &lutris_path_value)
                    .on_input(Message::LutrisPathChanged)
                    .style(style::text_input(palette))
                    .width(Length::Fill),
                button("Browse…")
                    .on_press(Message::BrowseForLutrisDatabase)
                    .style(style::btn_secondary(palette)),
            ]
            .spacing(10),
        )
        .push(text("Lutris icon directory:").color(palette.text_muted))
        .push(
            row![
                text_input(
                    "/home/user/.local/share/icons/hicolor/128x128/apps",
                    &icons_path_value
                )
                .on_input(Message::LutrisIconsPathChanged)
                .style(style::text_input(palette))
                .width(Length::Fill),
                button("Browse…")
                    .on_press(Message::BrowseForLutrisIcons)
                    .style(style::btn_secondary(palette)),
            ]
            .spacing(10),
        )
        .push(
            button("Save")
                .on_press(Message::SaveSettings)
                .style(style::btn_primary(palette)),
        );

    container(content.spacing(20).padding(20))
        .style(style::card(palette))
        .width(Length::Fill)
        .into()
}
