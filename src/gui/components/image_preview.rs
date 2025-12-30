use iced::widget::{container, image, text};
use iced::{Element, Length};

pub fn view(path: Option<String>) -> Element<'static, ()> {
    if let Some(p) = path {
        container(image(p).width(Length::Fixed(200.0))).into()
    } else {
        container(text("No image")).into()
    }
}
