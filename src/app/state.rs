use crate::sources::traits::ImageKind;
use ::image::ImageFormat;
use iced::widget::{image, Id as ScrollId};
use std::collections::HashMap;
use std::path::PathBuf;

pub const COVER_DIMENSIONS: (u32, u32) = (600, 900);
pub const BANNER_DIMENSIONS: (u32, u32) = (920, 430);
pub const ICON_DIMENSIONS: (u32, u32) = (512, 512);
const HERO_DIMENSIONS: (u32, u32) = (1920, 1080);
const LOGO_DIMENSIONS: (u32, u32) = (1024, 512);
const ASPECT_TOLERANCE: f32 = 0.04;

#[derive(Debug, Clone, Default)]
pub struct CurrentGameImages {
    pub cover: Option<PathBuf>,
    pub banner: Option<PathBuf>,
    pub icon: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum GameViewStep {
    #[default]
    Details,
    SearchResults,
    ImageSelection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ImageTab {
    #[default]
    Cover,
    Banner,
    Icon,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DownloadStatus {
    #[default]
    Idle,
    Downloading,
    Success,
}

#[derive(Debug, Clone)]
pub struct ScrollIds {
    pub sidebar: ScrollId,
    pub search_results: ScrollId,
    pub image_selection: ScrollId,
    pub boop_matches: ScrollId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AspectAction {
    Original,
    Stretch,
    Cover,
    Contain,
}

pub const TRANSFORM_ACTIONS: [AspectAction; 3] = [
    AspectAction::Stretch,
    AspectAction::Cover,
    AspectAction::Contain,
];

#[derive(Debug, Clone)]
pub struct AspectPrompt {
    pub url: String,
    pub slug: String,
    pub kind: ImageKind,
    pub raw_image: RawImageData,
    pub previews: HashMap<AspectAction, PreviewState>,
    pub original_handle: image::Handle,
}

#[derive(Debug, Clone)]
pub enum PreviewState {
    Pending,
    Ready(image::Handle),
    Error(String),
}

#[derive(Debug, Clone)]
pub struct RawImageData {
    pub bytes: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub format: ImageFormat,
}

pub fn target_dimensions(kind: &ImageKind) -> (u32, u32) {
    match kind {
        ImageKind::Cover => COVER_DIMENSIONS,
        ImageKind::Banner => BANNER_DIMENSIONS,
        ImageKind::Icon => ICON_DIMENSIONS,
        ImageKind::Hero => HERO_DIMENSIONS,
        ImageKind::Logo => LOGO_DIMENSIONS,
    }
}

pub fn aspect_ratio(width: u32, height: u32) -> f32 {
    if height == 0 {
        return 0.0;
    }
    width as f32 / height as f32
}

pub fn ratio_matches_target(width: u32, height: u32, kind: &ImageKind) -> bool {
    let (target_w, target_h) = target_dimensions(kind);
    let actual = aspect_ratio(width, height);
    let expected = aspect_ratio(target_w, target_h);
    (actual - expected).abs() <= ASPECT_TOLERANCE
}

impl ScrollIds {
    pub fn new() -> Self {
        Self {
            sidebar: ScrollId::new("sidebar-scroll"),
            search_results: ScrollId::new("search-results-scroll"),
            image_selection: ScrollId::new("image-selection-scroll"),
            boop_matches: ScrollId::new("protocol-matches-scroll"),
        }
    }
}

impl AspectPrompt {
    pub fn new(url: String, slug: String, kind: ImageKind, raw_image: RawImageData) -> Self {
        let mut previews = HashMap::new();
        for action in TRANSFORM_ACTIONS {
            previews.insert(action, PreviewState::Pending);
        }

        let original_handle = image::Handle::from_bytes(raw_image.bytes.clone());

        Self {
            url,
            slug,
            kind,
            raw_image,
            previews,
            original_handle,
        }
    }

    pub fn target_dimensions(&self) -> (u32, u32) {
        target_dimensions(&self.kind)
    }

    pub fn actual_dimensions(&self) -> (u32, u32) {
        (self.raw_image.width, self.raw_image.height)
    }

    pub fn preview_state(&self, action: AspectAction) -> &PreviewState {
        self.previews.get(&action).expect("preview state missing")
    }

    pub fn set_preview_state(&mut self, action: AspectAction, state: PreviewState) {
        self.previews.insert(action, state);
    }
}
