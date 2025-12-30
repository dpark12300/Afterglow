pub mod app;
pub mod config;
pub mod gui;
pub mod lutris;
pub mod sources;
pub mod style;
pub mod utils;

use app::LutrisImageManager;

pub fn main() -> iced::Result {
    iced::daemon(
        LutrisImageManager::new,
        LutrisImageManager::update,
        LutrisImageManager::view,
    )
    .title(LutrisImageManager::title)
    .subscription(LutrisImageManager::subscription)
    .theme(LutrisImageManager::theme)
    .run()
}
