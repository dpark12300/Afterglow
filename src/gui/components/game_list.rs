use crate::lutris::database::Game;
use iced::widget::{button, column, scrollable, text};
use iced::{Element, Length};

#[derive(Debug, Clone)]
pub enum Message {
    GameSelected(Game),
}

pub fn view(games: &[Game]) -> Element<'_, Message> {
    let list = games.iter().fold(column![].spacing(10), |col, game| {
        col.push(
            button(text(&game.name))
                .on_press(Message::GameSelected(game.clone()))
                .width(Length::Fill),
        )
    });

    scrollable(list)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
