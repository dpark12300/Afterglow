mod state;
mod tasks;

use iced::window::icon;
use iced::window::settings::PlatformSpecific;
pub(crate) use state::DownloadStatus;
pub use state::{
    AspectAction, AspectPrompt, CurrentGameImages, ImageTab, PreviewState, TRANSFORM_ACTIONS,
};

use crate::config::{Config, ThemeVariant};
use crate::gui::{dashboard, settings, ScrollRegion, SmoothScrollController};
use crate::lutris::database::Game;
use crate::lutris::paths::LutrisPaths;
use crate::sources::steamgriddb::{boop, SteamGridDB};
use crate::sources::traits::{GameImage, ImageKind, ImageSource, SearchResult};
use crate::style;
use crate::utils::image_loader::download_image;
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use iced::widget::operation::snap_to;
use iced::widget::scrollable::{RelativeOffset, Viewport};
use iced::widget::{
    button, column, container, image, mouse_area, row, scrollable, text, text_input,
    Id as ScrollId, Space, Stack,
};
use iced::{
    alignment, mouse, time, window, Alignment, ContentFit, Element, Length, Size, Subscription,
    Task, Theme,
};
use state::{ratio_matches_target, GameViewStep, RawImageData, ScrollIds};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::time::Duration;
use tasks::{
    check_current_images, download_full_image, generate_preview_image, load_games,
    process_and_save_image,
};
use url::form_urlencoded;

const FRAME_TICK_MS: u64 = 16;
const MARQUEE_TICK_MS: u64 = 100;
const MARQUEE_STEP: f32 = 0.02;
const BOOP_DONE_DISPLAY_MS: u64 = 3_000;
pub struct LutrisImageManager {
    games: Vec<Game>,
    selected_game: Option<Game>,
    lutris_paths: Option<LutrisPaths>,
    needs_database_path: bool,
    auto_prompted_for_database: bool,
    error: Option<String>,
    // Config
    config: Config,
    show_settings: bool,
    // Search state
    steamgriddb: Option<SteamGridDB>,
    search_results: Vec<SearchResult>,
    image_candidates: Vec<GameImage>,
    download_statuses: HashMap<String, DownloadStatus>,
    is_searching: bool,
    // Image Cache
    image_cache: HashMap<String, image::Handle>,
    pending_images: HashSet<String>,
    // Current Game Images
    current_images: CurrentGameImages,
    // View State
    game_view_step: GameViewStep,
    selected_image_tab: ImageTab,
    hovered_search_result: Option<String>,
    hovered_scroll_id: Option<ScrollId>,
    hovered_title_scrollable: bool,
    marquee_offset: f32,
    marquee_tick_elapsed: u64,
    scroll_ids: ScrollIds,
    smooth_scroll: SmoothScrollController,
    aspect_prompt: Option<AspectPrompt>,
    window_kinds: HashMap<window::Id, WindowKind>,
    main_window_id: Option<window::Id>,
    prompt_window_id: Option<window::Id>,
    boop_window_id: Option<window::Id>,
    boop_window_pending: bool,
    pending_boop_request: Option<SgdbBoopRequest>,
    boop_popup: Option<SgdbPopupState>,
    boop_notification: Option<SgdbNotification>,
    exit_after_boop: bool,
    boop_only_mode: bool,
    boop_success_visible: bool,
    boop_success_elapsed: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowKind {
    Main,
    AspectPrompt,
    Boop,
}

const POPUP_MATCH_LIMIT: usize = 12;

#[derive(Debug, Clone, PartialEq, Eq)]
struct SgdbBoopRequest {
    asset_type: SgdbAssetType,
    asset_id: String,
    is_nonsteam: bool,
}

#[derive(Debug, Clone)]
pub struct SgdbBoopAsset {
    app_id: String,
    url: String,
    kind: ImageKind,
}

#[derive(Debug, Clone)]
struct SgdbPopupState {
    asset: SgdbBoopAsset,
    filter_value: String,
    filter_modified: bool,
    matches: Vec<SgdbPopupMatch>,
    selected_game_id: Option<i64>,
    awaiting_name: bool,
}

#[derive(Debug, Clone)]
struct SgdbPopupMatch {
    game: Game,
    score: i64,
}

#[derive(Debug, Clone)]
struct SgdbNotification {
    title: String,
    body: String,
    kind: SgdbNotificationKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SgdbNotificationKind {
    Info,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum BoopInvocation {
    Test,
    Apply(SgdbBoopRequest),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SgdbAssetType {
    Grid,
    Hero,
    Logo,
    Icon,
}

impl SgdbBoopAsset {
    fn as_game_image(&self) -> GameImage {
        GameImage {
            url: self.url.clone(),
            thumb: self.url.clone(),
            kind: self.kind.clone(),
        }
    }
}

impl SgdbAssetType {
    fn from_segment(value: &str) -> Option<Self> {
        match value {
            "grid" => Some(Self::Grid),
            "hero" => Some(Self::Hero),
            "logo" => Some(Self::Logo),
            "icon" => Some(Self::Icon),
            _ => None,
        }
    }

    fn as_str(&self) -> &'static str {
        match self {
            Self::Grid => "grid",
            Self::Hero => "hero",
            Self::Logo => "logo",
            Self::Icon => "icon",
        }
    }

    fn image_kind(&self) -> ImageKind {
        match self {
            Self::Grid => ImageKind::Cover,
            Self::Hero => ImageKind::Hero,
            Self::Logo => ImageKind::Logo,
            Self::Icon => ImageKind::Icon,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    GamesLoaded(Result<Vec<Game>, String>),
    ConfigLoaded(Config),
    ToggleSettings,
    ToggleThemeMode,
    SaveSettings,
    LutrisPathChanged(String),
    BrowseForLutrisDatabase,
    LutrisDatabaseSelected(Option<PathBuf>),
    LutrisIconsPathChanged(String),
    BrowseForLutrisIcons,
    LutrisIconsPathSelected(Option<PathBuf>),
    GameSelected(Game),
    CurrentImagesLoaded(CurrentGameImages),
    ApiKeyChanged(String),
    SearchGame,
    SearchCompleted(Result<Vec<SearchResult>, String>),
    SearchResultSelected(SearchResult),
    ImagesLoaded(Result<Vec<GameImage>, String>),
    ApplyImage(GameImage),
    ImageDownloadCompleted {
        url: String,
        slug: String,
        kind: ImageKind,
        result: Result<RawImageData, String>,
    },
    ImageApplied(String, Result<(), String>),
    LoadImage(String),
    ImageLoaded(String, Result<image::Handle, String>),
    SearchImageFound(String, Option<String>),
    BackToDetails,
    BackToSearchResults,
    SelectImageTab(ImageTab),
    SearchResultHover(String),
    SearchResultHoverEnd(String),
    ScrollWheel(ScrollRegion, mouse::ScrollDelta),
    ScrollViewportChanged(ScrollRegion, Viewport),
    Tick,
    ConfigPersisted(Result<(), String>),
    AspectPreviewReady {
        url: String,
        action: AspectAction,
        result: Result<Vec<u8>, String>,
    },
    ConfirmAspectAction(AspectAction),
    CancelAspectPrompt,
    WindowOpened {
        id: window::Id,
        kind: WindowKind,
    },
    WindowClosed(window::Id),
    BoopAssetFetched(Result<SgdbBoopAsset, String>),
    BoopFilterChanged(String),
    BoopMatchSelected(i64),
    BoopApplyConfirmed,
    BoopPopupDismissed,
    BoopNotificationDismissed,
    NoOp,
}

impl LutrisImageManager {
    pub fn new() -> (Self, Task<Message>) {
        let load_config_command = Task::perform(Config::load(), Message::ConfigLoaded);

        let boop_invocation = Self::read_boop_invocation();
        let boop_only_mode = boop_invocation.is_some();

        let mut app = Self {
            games: vec![],
            selected_game: None,
            lutris_paths: None,
            needs_database_path: false,
            auto_prompted_for_database: false,
            error: None,
            config: Config::default(),
            show_settings: false,
            steamgriddb: None,
            search_results: vec![],
            image_candidates: vec![],
            download_statuses: HashMap::new(),
            is_searching: false,
            image_cache: HashMap::new(),
            pending_images: HashSet::new(),
            current_images: CurrentGameImages::default(),
            game_view_step: GameViewStep::default(),
            selected_image_tab: ImageTab::default(),
            hovered_search_result: None,
            hovered_scroll_id: None,
            hovered_title_scrollable: false,
            marquee_offset: 0.0,
            marquee_tick_elapsed: 0,
            scroll_ids: ScrollIds::new(),
            smooth_scroll: SmoothScrollController::default(),
            aspect_prompt: None,
            window_kinds: HashMap::new(),
            main_window_id: None,
            prompt_window_id: None,
            boop_window_id: None,
            boop_window_pending: false,
            pending_boop_request: None,
            boop_popup: None,
            boop_notification: None,
            exit_after_boop: false,
            boop_only_mode,
            boop_success_visible: false,
            boop_success_elapsed: 0,
        };

        let mut startup_tasks = vec![load_config_command];

        if !boop_only_mode {
            let (_, open_main) = window::open(window::Settings {
                platform_specific: PlatformSpecific {
                    application_id: "dev.bigboot.afterglow".into(),
                    ..Default::default()
                },
                icon: Some(
                    icon::from_file_data(include_bytes!("../assets/icon-32.png"), None).unwrap(),
                ),
                ..window::Settings::default()
            });

            startup_tasks.push(open_main.map(|id| Message::WindowOpened {
                id,
                kind: WindowKind::Main,
            }));
        }

        if let Some(invocation) = boop_invocation {
            if let Some(task) = app.handle_boop_invocation(invocation) {
                startup_tasks.push(task);
            }
        }

        (app, Task::batch(startup_tasks))
    }

    pub fn title(&self, window: window::Id) -> String {
        match self.window_kinds.get(&window).copied() {
            Some(WindowKind::AspectPrompt) => String::from("Afterglow - Select Aspect Ratio"),
            _ => String::from("Afterglow"),
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::GamesLoaded(Ok(games)) => {
                self.games = games;
                self.error = None;
                self.refresh_boop_matches();
                Task::none()
            }
            Message::GamesLoaded(Err(e)) => {
                self.error = Some(e);
                Task::none()
            }
            Message::ConfigLoaded(config) => {
                self.config = config;
                self.refresh_steamgriddb_client();
                self.initialize_lutris_paths(false)
            }
            Message::ToggleSettings => {
                self.show_settings = !self.show_settings;
                Task::none()
            }
            Message::ToggleThemeMode => {
                self.config.theme = self.config.theme.toggle();
                self.persist_config()
            }
            Message::ApiKeyChanged(key) => {
                if key.trim().is_empty() {
                    self.config.steamgriddb_api_key = None;
                } else {
                    self.config.steamgriddb_api_key = Some(key);
                }
                Task::none()
            }
            Message::SaveSettings => {
                self.refresh_steamgriddb_client();
                self.show_settings = false;
                self.needs_database_path = self.lutris_paths.is_none();
                Task::batch(vec![
                    self.persist_config(),
                    self.initialize_lutris_paths(false),
                ])
            }
            Message::LutrisPathChanged(path) => {
                let trimmed = path.trim();
                if trimmed.is_empty() {
                    self.config.lutris_database_path = None;
                } else {
                    self.config.lutris_database_path = Some(PathBuf::from(trimmed));
                }
                Task::none()
            }
            Message::BrowseForLutrisDatabase => self.prompt_for_lutris_database(),
            Message::LutrisDatabaseSelected(Some(path)) => self.handle_database_selection(path),
            Message::LutrisDatabaseSelected(None) => Task::none(),
            Message::LutrisIconsPathChanged(path) => {
                let trimmed = path.trim();
                if trimmed.is_empty() {
                    self.config.lutris_icons_path = None;
                } else {
                    self.config.lutris_icons_path = Some(PathBuf::from(trimmed));
                }
                Task::none()
            }
            Message::BrowseForLutrisIcons => self.prompt_for_lutris_icons(),
            Message::LutrisIconsPathSelected(Some(path)) => {
                self.config.lutris_icons_path = Some(path);
                Task::batch(vec![
                    self.persist_config(),
                    self.initialize_lutris_paths(false),
                ])
            }
            Message::LutrisIconsPathSelected(None) => Task::none(),
            Message::GameSelected(game) => self.finish_game_selection(game),
            Message::CurrentImagesLoaded(images) => {
                self.current_images = images;
                Task::none()
            }
            Message::SearchGame => {
                if let Some(game) = &self.selected_game {
                    if let Some(client) = &self.steamgriddb {
                        self.is_searching = true;
                        self.game_view_step = GameViewStep::SearchResults;
                        let client = client.clone();
                        let query = game.name.clone();
                        return Task::perform(
                            async move { client.search(&query).await.map_err(|e| e.to_string()) },
                            Message::SearchCompleted,
                        );
                    }
                }
                Task::none()
            }
            Message::SearchCompleted(Ok(results)) => {
                self.is_searching = false;
                self.search_results = results;

                let mut commands = vec![];
                if let Some(client) = &self.steamgriddb {
                    for res in &self.search_results {
                        let client = client.clone();
                        let id = res.id.clone();
                        let id_for_msg = id.clone();
                        commands.push(Task::perform(
                            async move { client.get_thumbnail(&id).await.unwrap_or(None) },
                            move |url| Message::SearchImageFound(id_for_msg, url),
                        ));
                    }
                }
                Task::batch(commands)
            }
            Message::SearchCompleted(Err(e)) => {
                self.is_searching = false;
                self.error = Some(e);
                Task::none()
            }
            Message::SearchResultSelected(result) => {
                if let Some(client) = &self.steamgriddb {
                    self.game_view_step = GameViewStep::ImageSelection;
                    self.selected_image_tab = ImageTab::default();
                    self.download_statuses.clear();
                    let client = client.clone();
                    let game_id = result.id.clone();
                    return Task::perform(
                        async move { client.get_images(&game_id).await.map_err(|e| e.to_string()) },
                        Message::ImagesLoaded,
                    );
                }
                Task::none()
            }
            Message::ImagesLoaded(Ok(images)) => {
                self.image_candidates = images;
                self.download_statuses.clear();
                let mut commands = vec![];
                for img in &self.image_candidates {
                    if !self.image_cache.contains_key(&img.thumb)
                        && !self.pending_images.contains(&img.thumb)
                    {
                        self.pending_images.insert(img.thumb.clone());
                        let url = img.thumb.clone();
                        let url_for_msg = url.clone();
                        commands.push(Task::perform(download_image(url), move |res| {
                            Message::ImageLoaded(url_for_msg, res)
                        }));
                    }
                }
                Task::batch(commands)
            }
            Message::ImagesLoaded(Err(e)) => {
                self.error = Some(e);
                Task::none()
            }
            Message::LoadImage(url) => {
                if !self.image_cache.contains_key(&url) && !self.pending_images.contains(&url) {
                    self.pending_images.insert(url.clone());
                    return Task::perform(download_image(url.clone()), move |res| {
                        Message::ImageLoaded(url, res)
                    });
                }
                Task::none()
            }
            Message::ImageLoaded(url, Ok(handle)) => {
                self.pending_images.remove(&url);
                self.image_cache.insert(url, handle);
                Task::none()
            }
            Message::ImageLoaded(url, Err(_)) => {
                self.pending_images.remove(&url);
                Task::none()
            }
            Message::SearchImageFound(id, url_opt) => {
                if let Some(url) = url_opt {
                    if let Some(res) = self.search_results.iter_mut().find(|r| r.id == id) {
                        res.image_url = Some(url.clone());
                    }
                    // Trigger download
                    if !self.image_cache.contains_key(&url) && !self.pending_images.contains(&url) {
                        self.pending_images.insert(url.clone());
                        return Task::perform(download_image(url.clone()), move |res| {
                            Message::ImageLoaded(url, res)
                        });
                    }
                }
                Task::none()
            }
            Message::BackToDetails => {
                self.game_view_step = GameViewStep::Details;
                Task::none()
            }
            Message::BackToSearchResults => {
                self.game_view_step = GameViewStep::SearchResults;
                Task::none()
            }
            Message::SelectImageTab(tab) => {
                self.selected_image_tab = tab;
                Task::none()
            }
            Message::SearchResultHover(id) => {
                let needs_scroll = dashboard::title_needs_scroll(&id, &self.search_results);
                self.hovered_search_result = Some(id.clone());
                self.hovered_scroll_id = if needs_scroll {
                    Some(dashboard::title_scroll_id(&id))
                } else {
                    None
                };
                self.hovered_title_scrollable = needs_scroll;
                self.marquee_offset = 0.0;
                self.marquee_tick_elapsed = 0;

                if let Some(scroll_id) = self.hovered_scroll_id.clone() {
                    snap_to(
                        scroll_id,
                        RelativeOffset::<Option<f32>> {
                            x: Some(0.0),
                            y: None,
                        },
                    )
                } else {
                    Task::none()
                }
            }
            Message::SearchResultHoverEnd(id) => {
                self.hovered_search_result = None;
                self.hovered_scroll_id = None;
                self.hovered_title_scrollable = false;
                self.marquee_offset = 0.0;
                self.marquee_tick_elapsed = 0;
                snap_to(
                    dashboard::title_scroll_id(&id),
                    RelativeOffset::<Option<f32>> {
                        x: Some(0.0),
                        y: None,
                    },
                )
            }
            Message::ScrollWheel(region, delta) => {
                self.smooth_scroll.handle_wheel(region, delta);
                Task::none()
            }
            Message::ScrollViewportChanged(region, viewport) => {
                self.smooth_scroll.handle_viewport_change(region, viewport);
                Task::none()
            }
            Message::Tick => {
                let mut commands: Vec<Task<Message>> = Vec::new();

                if self.hovered_title_scrollable {
                    if let Some(scroll_id) = self.hovered_scroll_id.clone() {
                        self.marquee_tick_elapsed += FRAME_TICK_MS;
                        if self.marquee_tick_elapsed >= MARQUEE_TICK_MS {
                            self.marquee_tick_elapsed = 0;
                            self.marquee_offset += MARQUEE_STEP;
                            if self.marquee_offset > 1.0 {
                                self.marquee_offset = 0.0;
                            }
                            commands.push(snap_to(
                                scroll_id,
                                RelativeOffset::<Option<f32>> {
                                    x: Some(self.marquee_offset),
                                    y: None,
                                },
                            ));
                        }
                    }
                }

                for (region, offset) in self.smooth_scroll.step() {
                    let scroll_id = self.scroll_id(region);
                    commands.push(snap_to(
                        scroll_id,
                        RelativeOffset::<Option<f32>> {
                            x: None,
                            y: Some(offset),
                        },
                    ));
                }

                if self.boop_success_visible {
                    self.boop_success_elapsed =
                        self.boop_success_elapsed.saturating_add(FRAME_TICK_MS);

                    if self.boop_success_elapsed >= BOOP_DONE_DISPLAY_MS {
                        self.clear_boop_success_state();

                        if self.exit_after_boop {
                            self.exit_after_boop = false;
                            commands.push(iced::exit());
                        } else if let Some(task) = self.request_close_boop_window() {
                            commands.push(task);
                        }
                    }
                }

                if commands.is_empty() {
                    Task::none()
                } else {
                    Task::batch(commands)
                }
            }
            Message::ApplyImage(image) => self.begin_image_application(image),
            Message::ImageDownloadCompleted {
                url,
                slug,
                kind,
                result,
            } => {
                if let Some(game) = &self.selected_game {
                    if game.slug != slug {
                        self.download_statuses.insert(url, DownloadStatus::Idle);
                        return Task::none();
                    }
                }

                match result {
                    Ok(data) => {
                        let Some(paths) = self.lutris_paths.clone() else {
                            self.download_statuses.insert(url, DownloadStatus::Idle);
                            self.error = Some(
                                "Missing Lutris path. Please reconfigure the database location."
                                    .to_string(),
                            );
                            return Task::none();
                        };
                        if ratio_matches_target(data.width, data.height, &kind) {
                            let apply_url = url.clone();
                            return Task::perform(
                                process_and_save_image(
                                    data,
                                    kind,
                                    slug,
                                    AspectAction::Original,
                                    paths,
                                ),
                                move |res| Message::ImageApplied(apply_url.clone(), res),
                            );
                        }

                        let prompt_data = data.clone();
                        let prompt_kind = kind.clone();
                        self.aspect_prompt = Some(AspectPrompt::new(
                            url.clone(),
                            slug,
                            prompt_kind,
                            prompt_data,
                        ));

                        let mut commands: Vec<Task<Message>> = Vec::new();
                        if let Some(task) = self.ensure_prompt_window() {
                            commands.push(task);
                        }

                        for action in TRANSFORM_ACTIONS {
                            let data_clone = data.clone();
                            let kind_clone = kind.clone();
                            let url_clone = url.clone();
                            commands.push(Task::perform(
                                generate_preview_image(data_clone, kind_clone, action),
                                move |res| Message::AspectPreviewReady {
                                    url: url_clone.clone(),
                                    action,
                                    result: res,
                                },
                            ));
                        }
                        if commands.is_empty() {
                            Task::none()
                        } else {
                            Task::batch(commands)
                        }
                    }
                    Err(e) => {
                        self.download_statuses.insert(url, DownloadStatus::Idle);
                        self.error = Some(e);
                        Task::none()
                    }
                }
            }
            Message::ImageApplied(url, Ok(())) => {
                self.aspect_prompt = None;
                self.download_statuses.insert(url, DownloadStatus::Success);
                let mut commands: Vec<Task<Message>> = Vec::new();

                if self.boop_window_id.is_some() {
                    self.boop_success_visible = true;
                    self.boop_success_elapsed = 0;
                } else if self.exit_after_boop {
                    self.exit_after_boop = false;
                    return iced::exit();
                } else {
                    self.clear_boop_success_state();
                }

                if let Some(game) = &self.selected_game {
                    let slug = game.slug.clone();
                    if let Some(paths) = self.lutris_paths.clone() {
                        commands.push(Task::perform(
                            check_current_images(slug, paths),
                            Message::CurrentImagesLoaded,
                        ));
                    }
                }

                if let Some(task) = self.request_close_prompt_window() {
                    commands.push(task);
                }

                if commands.is_empty() {
                    Task::none()
                } else {
                    Task::batch(commands)
                }
            }
            Message::ImageApplied(url, Err(e)) => {
                self.aspect_prompt = None;
                self.download_statuses.insert(url, DownloadStatus::Idle);
                self.clear_boop_success_state();
                self.error = Some(e);
                if self.exit_after_boop {
                    self.exit_after_boop = false;
                    iced::exit()
                } else if let Some(task) = self.request_close_prompt_window() {
                    task
                } else {
                    Task::none()
                }
            }
            Message::AspectPreviewReady {
                url,
                action,
                result,
            } => {
                if let Some(prompt) = self.aspect_prompt.as_mut() {
                    if prompt.url == url {
                        let preview_state = match result {
                            Ok(bytes) => PreviewState::Ready(image::Handle::from_bytes(bytes)),
                            Err(e) => PreviewState::Error(e),
                        };
                        prompt.set_preview_state(action, preview_state);
                    }
                }
                Task::none()
            }
            Message::ConfirmAspectAction(action) => {
                let Some(paths) = self.lutris_paths.clone() else {
                    self.error = Some(
                        "Missing Lutris path. Please reconfigure the database location.".into(),
                    );
                    return Task::none();
                };
                if let Some(prompt) = self.aspect_prompt.take() {
                    let data = prompt.raw_image;
                    let kind = prompt.kind;
                    let slug = prompt.slug;
                    let apply_url = prompt.url;
                    let process_task = Task::perform(
                        process_and_save_image(data, kind, slug, action, paths),
                        move |res| Message::ImageApplied(apply_url.clone(), res),
                    );

                    let mut commands = vec![process_task];
                    if let Some(task) = self.request_close_prompt_window() {
                        commands.push(task);
                    }

                    return Task::batch(commands);
                }
                Task::none()
            }
            Message::CancelAspectPrompt => {
                if let Some(prompt) = self.aspect_prompt.take() {
                    self.download_statuses
                        .insert(prompt.url.clone(), DownloadStatus::Idle);
                }
                if let Some(task) = self.request_close_prompt_window() {
                    task
                } else {
                    Task::none()
                }
            }
            Message::ConfigPersisted(Ok(())) => Task::none(),
            Message::ConfigPersisted(Err(e)) => {
                self.error = Some(e);
                Task::none()
            }
            Message::WindowOpened { id, kind } => {
                self.window_kinds.insert(id, kind);
                match kind {
                    WindowKind::Main => {
                        self.main_window_id = Some(id);
                        Task::none()
                    }
                    WindowKind::AspectPrompt => {
                        self.prompt_window_id = Some(id);
                        window::gain_focus(id).map(|_: ()| Message::NoOp)
                    }
                    WindowKind::Boop => {
                        self.boop_window_id = Some(id);
                        self.boop_window_pending = false;
                        window::gain_focus(id).map(|_: ()| Message::NoOp)
                    }
                }
            }
            Message::WindowClosed(id) => {
                self.window_kinds.remove(&id);
                if Some(id) == self.prompt_window_id {
                    self.prompt_window_id = None;
                    if let Some(prompt) = self.aspect_prompt.take() {
                        self.download_statuses
                            .insert(prompt.url.clone(), DownloadStatus::Idle);
                    }
                    Task::none()
                } else if Some(id) == self.boop_window_id {
                    self.boop_window_id = None;
                    self.boop_window_pending = false;
                    self.boop_popup = None;
                    self.boop_notification = None;
                    self.pending_boop_request = None;
                    self.exit_after_boop = false;
                    self.clear_boop_success_state();
                    if self.boop_only_mode && self.main_window_id.is_none() {
                        iced::exit()
                    } else {
                        Task::none()
                    }
                } else if Some(id) == self.main_window_id {
                    self.main_window_id = None;
                    iced::exit()
                } else {
                    Task::none()
                }
            }
            Message::BoopAssetFetched(result) => {
                self.pending_boop_request = None;
                self.clear_boop_success_state();
                match result {
                    Ok(asset) => {
                        let popup = SgdbPopupState {
                            filter_value: self.default_boop_filter(&asset),
                            filter_modified: false,
                            matches: Vec::new(),
                            selected_game_id: None,
                            awaiting_name: false,
                            asset,
                        };

                        let mut commands = Vec::new();
                        if let Some(task) = self.ensure_image_cached(&popup.asset.url) {
                            commands.push(task);
                        }

                        if let Some(task) = self.ensure_boop_window() {
                            commands.push(task);
                        }

                        self.boop_popup = Some(popup);
                        self.refresh_boop_matches();
                        if commands.is_empty() {
                            Task::none()
                        } else {
                            Task::batch(commands)
                        }
                    }
                    Err(e) => {
                        self.boop_notification = Some(SgdbNotification {
                            title: "SGDBoop".into(),
                            body: format!("Failed to prepare SGDBoop request: {}", e),
                            kind: SgdbNotificationKind::Error,
                        });
                        if let Some(task) = self.ensure_boop_window() {
                            task
                        } else {
                            Task::none()
                        }
                    }
                }
            }
            Message::BoopFilterChanged(value) => {
                if let Some(popup) = &mut self.boop_popup {
                    popup.filter_value = value;
                    popup.filter_modified = true;
                    self.refresh_boop_matches();
                }
                Task::none()
            }
            Message::BoopMatchSelected(game_id) => {
                let already_selected = self
                    .boop_popup
                    .as_ref()
                    .and_then(|popup| popup.selected_game_id)
                    == Some(game_id);

                if already_selected {
                    return self.apply_selected_boop_game();
                }

                if let Some(popup) = &mut self.boop_popup {
                    popup.selected_game_id = Some(game_id);
                }
                Task::none()
            }
            Message::BoopApplyConfirmed => self.apply_selected_boop_game(),
            Message::BoopPopupDismissed => {
                self.boop_popup = None;
                self.exit_after_boop = false;
                self.clear_boop_success_state();
                if let Some(task) = self.maybe_close_boop_window() {
                    task
                } else {
                    Task::none()
                }
            }
            Message::BoopNotificationDismissed => {
                self.boop_notification = None;
                self.exit_after_boop = false;
                self.clear_boop_success_state();
                if let Some(task) = self.maybe_close_boop_window() {
                    task
                } else {
                    Task::none()
                }
            }
            Message::NoOp => Task::none(),
        }
    }

    pub fn view(&self, window: window::Id) -> Element<'_, Message> {
        let palette = style::palette(self.config.theme);

        match self.window_kinds.get(&window).copied() {
            Some(WindowKind::AspectPrompt) => self.view_prompt_window(palette),
            Some(WindowKind::Boop) => self.view_boop_window(palette),
            _ => self.view_main_window(palette),
        }
    }

    fn view_main_window(&self, palette: style::Palette) -> Element<'_, Message> {
        if let Some(error) = &self.error {
            return container(
                column![
                    text("Error:").size(20).color(palette.accent),
                    text(error).color(palette.text),
                    button("Dismiss")
                        .on_press(Message::GamesLoaded(Ok(self.games.clone())))
                        .style(style::btn_primary(palette))
                ]
                .spacing(10),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .style(style::app_container(palette))
            .into();
        }

        let game_list = self.games.iter().fold(column![].spacing(5), |col, game| {
            let is_selected = self
                .selected_game
                .as_ref()
                .map(|g| g.id == game.id)
                .unwrap_or(false);
            col.push(
                button(text(&game.name))
                    .on_press(Message::GameSelected(game.clone()))
                    .width(Length::Fill)
                    .style(style::btn_nav(is_selected, palette)),
            )
        });

        let theme_toggle_label = match self.config.theme {
            ThemeVariant::Dark => "☀",
            ThemeVariant::Light => "🌙",
        };

        let sidebar_scroll = scrollable(game_list)
            .height(Length::Fill)
            .id(self.scroll_ids.sidebar.clone())
            .on_scroll(|viewport| Message::ScrollViewportChanged(ScrollRegion::Sidebar, viewport));

        let sidebar_overlay = mouse_area(
            container(Space::new())
                .width(Length::Fill)
                .height(Length::Fill),
        )
        .on_scroll(|delta| Message::ScrollWheel(ScrollRegion::Sidebar, delta))
        .interaction(mouse::Interaction::None);

        let sidebar_scroll = Stack::new()
            .push(sidebar_scroll)
            .push(sidebar_overlay)
            .width(Length::Fill)
            .height(Length::Fill);

        let sidebar = container(column![
            row![
                text("Games").size(20).color(palette.text),
                Space::new().width(Length::Fill),
                button(theme_toggle_label)
                    .on_press(Message::ToggleThemeMode)
                    .style(style::btn_secondary(palette)),
                button("⚙")
                    .on_press(Message::ToggleSettings)
                    .style(style::btn_secondary(palette))
            ]
            .spacing(10)
            .padding(10),
            sidebar_scroll
        ])
        .width(Length::Fixed(250.0))
        .height(Length::Fill)
        .style(style::sidebar(palette));

        let missing_lutris_path = self.lutris_paths.is_none();
        let main_content = if self.show_settings {
            settings::settings_view(&self.config, palette, missing_lutris_path)
        } else if missing_lutris_path {
            self.missing_database_view(palette)
        } else if let Some(game) = &self.selected_game {
            match self.game_view_step {
                GameViewStep::Details => dashboard::details_view(dashboard::DetailsContext {
                    palette,
                    game,
                    current_images: &self.current_images,
                    can_search_images: self.steamgriddb.is_some(),
                }),
                GameViewStep::SearchResults => {
                    dashboard::search_results_view(dashboard::SearchResultsContext {
                        palette,
                        is_searching: self.is_searching,
                        search_results: &self.search_results,
                        image_cache: &self.image_cache,
                        hovered_search_result: self.hovered_search_result.as_deref(),
                        scroll_id: self.scroll_ids.search_results.clone(),
                    })
                }
                GameViewStep::ImageSelection => {
                    dashboard::image_selection_view(dashboard::ImageSelectionContext {
                        palette,
                        image_candidates: &self.image_candidates,
                        selected_tab: self.selected_image_tab,
                        image_cache: &self.image_cache,
                        download_statuses: &self.download_statuses,
                        scroll_id: self.scroll_ids.image_selection.clone(),
                    })
                }
            }
        } else {
            dashboard::empty_state_view(palette)
        };

        let content = row![
            sidebar,
            container(main_content)
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(20)
        ];

        let base: Element<_> = container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(style::app_container(palette))
            .into();

        base
    }

    fn view_prompt_window(&self, palette: style::Palette) -> Element<'_, Message> {
        if let Some(prompt) = &self.aspect_prompt {
            dashboard::aspect_prompt_window(dashboard::AspectPromptContext { palette, prompt })
        } else {
            container(text("No aspect-ratio adjustments pending.").color(palette.text_muted))
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .style(style::app_container(palette))
                .into()
        }
    }

    fn view_boop_window(&self, palette: style::Palette) -> Element<'_, Message> {
        let card: Element<_> = if let Some(popup) = &self.boop_popup {
            self.boop_popup_card(popup, palette)
        } else if let Some(notification) = &self.boop_notification {
            self.boop_notification_card(notification, palette)
        } else if self.pending_boop_request.is_some() {
            self.boop_overlay_card(palette)
        } else if self.boop_success_visible {
            self.boop_done_card(palette)
        } else {
            self.boop_idle_card(palette)
        };

        container(card)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(20)
            .align_x(Alignment::Center)
            .align_y(Alignment::Center)
            .style(style::app_container(palette))
            .into()
    }

    fn boop_popup_card(
        &self,
        popup: &SgdbPopupState,
        palette: style::Palette,
    ) -> Element<'_, Message> {
        let preview: Element<_> = if let Some(handle) = self.image_cache.get(&popup.asset.url) {
            image(handle.clone())
                .width(Length::Fixed(280.0))
                .height(Length::Fixed(280.0))
                .content_fit(ContentFit::Contain)
                .into()
        } else {
            container(text("Preview loading...").color(palette.text_muted))
                .width(Length::Fixed(280.0))
                .height(Length::Fixed(280.0))
                .align_x(alignment::Horizontal::Center)
                .align_y(alignment::Vertical::Center)
                .style(style::preview_card(palette))
                .into()
        };

        let target_label = {
            let label = self.default_boop_filter(&popup.asset);
            if label.trim().is_empty() {
                popup.asset.app_id.clone()
            } else {
                label
            }
        };

        let header = column![
            text("Apply SGDBoop image").size(26).color(palette.text),
            text(target_label).color(palette.text_muted),
        ]
        .spacing(4)
        .width(Length::Fill);

        let filter_input = text_input("Filter installed games", &popup.filter_value)
            .on_input(Message::BoopFilterChanged)
            .style(style::text_input(palette));

        let mut matches_column = column![].spacing(6).width(Length::Fill);
        if popup.matches.is_empty() {
            matches_column = matches_column.push(
                text("No Lutris games match the current filter.")
                    .color(palette.text_muted)
                    .width(Length::Fill),
            );
        } else {
            for entry in &popup.matches {
                let label = format!("{}", entry.game.name);
                let row_button = button(text(label))
                    .on_press(Message::BoopMatchSelected(entry.game.id))
                    .style(style::btn_nav(false, palette))
                    .width(Length::Fill);
                matches_column = matches_column.push(row_button);
            }
        }

        let matches_height = Length::Fixed(200.0);
        let matches_scroll = scrollable(matches_column)
            .height(matches_height)
            .width(Length::Fill)
            .id(self.scroll_ids.boop_matches.clone())
            .on_scroll(|viewport| {
                Message::ScrollViewportChanged(ScrollRegion::BoopMatches, viewport)
            });

        let matches_overlay = mouse_area(
            container(Space::new())
                .width(Length::Fill)
                .height(matches_height),
        )
        .on_scroll(|delta| Message::ScrollWheel(ScrollRegion::BoopMatches, delta))
        .interaction(mouse::Interaction::None);

        let matches_list = Stack::new()
            .push(matches_scroll)
            .push(matches_overlay)
            .width(Length::Fill)
            .height(matches_height);

        let right_column = column![
            text("Select a Lutris game").size(20).color(palette.text),
            filter_input,
            if popup.awaiting_name {
                text("Resolving official game name...").color(palette.text_muted)
            } else {
                text(" ")
            },
            matches_list,
        ]
        .spacing(12)
        .width(Length::Fill)
        .height(Length::Shrink);

        let body = row![
            container(preview)
                .width(Length::Fixed(300.0))
                .height(matches_height)
                .align_y(alignment::Vertical::Top),
            right_column,
        ]
        .spacing(24)
        .width(Length::Fill)
        .height(Length::Shrink);

        let actions = row![
            Space::new().width(Length::Fill),
            button("Cancel")
                .on_press(Message::BoopPopupDismissed)
                .style(style::btn_secondary(palette)),
        ]
        .spacing(10)
        .align_y(Alignment::Center);

        container(column![header, body, actions].spacing(20))
            .width(Length::Fill)
            .max_width(880.0)
            .height(Length::Shrink)
            .padding(24.0)
            .style(style::modal_card(palette))
            .into()
    }

    fn boop_notification_card(
        &self,
        notification: &SgdbNotification,
        palette: style::Palette,
    ) -> Element<'_, Message> {
        let icon = match notification.kind {
            SgdbNotificationKind::Info => "ℹ",
            SgdbNotificationKind::Error => "⚠",
        };

        let body = column![
            text(format!("{} {}", icon, notification.title))
                .size(24)
                .color(palette.text),
            text(notification.body.clone()).color(palette.text_muted),
            row![
                Space::new().width(Length::Fill),
                button("Close")
                    .on_press(Message::BoopNotificationDismissed)
                    .style(style::btn_primary(palette))
            ]
        ]
        .spacing(12);

        container(body)
            .width(Length::Fixed(520.0))
            .padding(24)
            .style(style::modal_card(palette))
            .into()
    }

    fn boop_overlay_card(&self, palette: style::Palette) -> Element<'_, Message> {
        let body = column![
            text("Preparing SGDBoop asset...")
                .size(24)
                .color(palette.text),
            text("Please wait while Afterglow downloads the requested image.")
                .color(palette.text_muted),
        ]
        .spacing(12)
        .align_x(Alignment::Center);

        container(body)
            .width(Length::Fixed(420.0))
            .padding(24)
            .style(style::modal_card(palette))
            .into()
    }

    fn boop_done_card(&self, palette: style::Palette) -> Element<'_, Message> {
        let seconds = self.boop_success_seconds_remaining();
        let countdown_label = format!("Closing in {}", seconds);

        container(
            column![
                text("✅ Image applied!").size(24).color(palette.text),
                text(countdown_label).color(palette.text_muted)
            ]
            .spacing(12)
            .align_x(Alignment::Center),
        )
        .width(Length::Fixed(520.0))
        .padding(24)
        .align_x(Alignment::Center)
        .style(style::modal_card(palette))
        .into()
    }

    fn boop_idle_card(&self, palette: style::Palette) -> Element<'_, Message> {
        container(
            column![
                text("Ready for SGDBoop requests")
                    .size(24)
                    .color(palette.text),
                text("Launch another SGDBoop link or close this window.").color(palette.text_muted)
            ]
            .spacing(12)
            .align_x(Alignment::Center),
        )
        .width(Length::Fixed(520.0))
        .padding(24)
        .align_x(Alignment::Center)
        .style(style::modal_card(palette))
        .into()
    }

    fn missing_database_view(&self, palette: style::Palette) -> Element<'_, Message> {
        container(
            column![
                text("Lutris database not found")
                    .size(28)
                    .color(palette.text),
                text("Select your Lutris Database to start managing artwork.")
                    .color(palette.text_muted),
                row![
                    button("Select Database")
                        .on_press(Message::BrowseForLutrisDatabase)
                        .style(style::btn_primary(palette)),
                    button("Open Settings")
                        .on_press(Message::ToggleSettings)
                        .style(style::btn_secondary(palette))
                ]
                .spacing(15)
            ]
            .spacing(20)
            .align_x(Alignment::Center),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .style(style::card(palette))
        .into()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let mut subscriptions = vec![window::close_events().map(Message::WindowClosed)];

        if self.hovered_title_scrollable
            || self.smooth_scroll.is_animating()
            || self.boop_success_visible
        {
            subscriptions
                .push(time::every(Duration::from_millis(FRAME_TICK_MS)).map(|_| Message::Tick));
        }

        Subscription::batch(subscriptions)
    }

    pub fn theme(&self, _window: window::Id) -> Option<Theme> {
        Some(match self.config.theme {
            ThemeVariant::Dark => Theme::Dark,
            ThemeVariant::Light => Theme::Light,
        })
    }

    fn scroll_id(&self, region: ScrollRegion) -> ScrollId {
        match region {
            ScrollRegion::Sidebar => self.scroll_ids.sidebar.clone(),
            ScrollRegion::SearchResults => self.scroll_ids.search_results.clone(),
            ScrollRegion::ImageSelection => self.scroll_ids.image_selection.clone(),
            ScrollRegion::BoopMatches => self.scroll_ids.boop_matches.clone(),
        }
    }

    fn ensure_prompt_window(&mut self) -> Option<Task<Message>> {
        if self.prompt_window_id.is_some() {
            None
        } else {
            let size = Self::prompt_window_size();
            let (_, open) = window::open(window::Settings {
                size,
                min_size: Some(size),
                max_size: Some(size),
                resizable: false,
                ..window::Settings::default()
            });

            Some(open.map(|id| Message::WindowOpened {
                id,
                kind: WindowKind::AspectPrompt,
            }))
        }
    }

    fn request_close_prompt_window(&mut self) -> Option<Task<Message>> {
        self.prompt_window_id
            .map(|id| window::close(id).map(|_: ()| Message::NoOp))
    }

    fn prompt_window_size() -> Size {
        Size::new(1024.0, 820.0)
    }

    fn ensure_boop_window(&mut self) -> Option<Task<Message>> {
        if self.boop_window_id.is_some() || self.boop_window_pending {
            None
        } else {
            let size = Self::boop_window_size();
            let (_, open) = window::open(window::Settings {
                size,
                min_size: Some(size),
                max_size: Some(size),
                resizable: false,
                ..window::Settings::default()
            });

            self.boop_window_pending = true;

            Some(open.map(|id| Message::WindowOpened {
                id,
                kind: WindowKind::Boop,
            }))
        }
    }

    fn request_close_boop_window(&mut self) -> Option<Task<Message>> {
        self.boop_window_id
            .map(|id| window::close(id).map(|_: ()| Message::NoOp))
    }

    fn maybe_close_boop_window(&mut self) -> Option<Task<Message>> {
        if self.boop_popup.is_none()
            && self.boop_notification.is_none()
            && self.pending_boop_request.is_none()
        {
            self.request_close_boop_window()
        } else {
            None
        }
    }

    fn boop_window_size() -> Size {
        Size::new(860.0, 560.0)
    }

    fn clear_boop_success_state(&mut self) {
        self.boop_success_visible = false;
        self.boop_success_elapsed = 0;
    }

    fn boop_success_seconds_remaining(&self) -> u64 {
        let remaining = BOOP_DONE_DISPLAY_MS.saturating_sub(self.boop_success_elapsed);
        ((remaining + 999) / 1000).max(1)
    }

    fn refresh_steamgriddb_client(&mut self) {
        if let Some(key) = &self.config.steamgriddb_api_key {
            if !key.is_empty() {
                self.steamgriddb = Some(SteamGridDB::new(key.clone()));
                return;
            }
        }
        self.steamgriddb = None;
    }

    fn persist_config(&self) -> Task<Message> {
        let config = self.config.clone();
        Task::perform(
            async move { config.save().await.map_err(|e| e.to_string()) },
            Message::ConfigPersisted,
        )
    }

    fn initialize_lutris_paths(&mut self, prompt_if_missing: bool) -> Task<Message> {
        if let Some(db_path) = self.find_available_database() {
            if let Some(paths) = self.build_lutris_paths(db_path.clone()) {
                let should_persist = self
                    .config
                    .lutris_database_path
                    .as_ref()
                    .map(|existing| existing != &db_path)
                    .unwrap_or(true);
                self.config.lutris_database_path = Some(db_path);
                let mut tasks = vec![self.load_games_for(paths)];
                if should_persist {
                    tasks.push(self.persist_config());
                }
                return Task::batch(tasks);
            }
        }

        self.lutris_paths = None;
        self.games.clear();
        self.selected_game = None;
        self.needs_database_path = true;

        if prompt_if_missing && !self.auto_prompted_for_database {
            self.auto_prompted_for_database = true;
            self.prompt_for_lutris_database()
        } else {
            Task::none()
        }
    }

    fn find_available_database(&self) -> Option<PathBuf> {
        let mut candidates: Vec<PathBuf> = Vec::new();
        if let Some(path) = &self.config.lutris_database_path {
            candidates.push(path.clone());
        }
        for candidate in LutrisPaths::default_database_locations() {
            if !candidates.iter().any(|existing| existing == &candidate) {
                candidates.push(candidate);
            }
        }

        for candidate in candidates {
            if candidate.is_file() {
                return Some(candidate);
            }
        }
        None
    }

    fn build_lutris_paths(&self, database_path: PathBuf) -> Option<LutrisPaths> {
        LutrisPaths::from_database_path(database_path, self.config.lutris_icons_path.clone())
    }

    fn load_games_for(&mut self, paths: LutrisPaths) -> Task<Message> {
        let db_path = paths.database_path();
        self.lutris_paths = Some(paths);
        self.needs_database_path = false;
        self.auto_prompted_for_database = false;
        self.error = None;
        self.games.clear();
        self.selected_game = None;
        self.current_images = CurrentGameImages::default();
        self.game_view_step = GameViewStep::default();
        Task::perform(load_games(db_path), Message::GamesLoaded)
    }

    fn handle_database_selection(&mut self, path: PathBuf) -> Task<Message> {
        if let Some(paths) = self.build_lutris_paths(path.clone()) {
            self.config.lutris_database_path = Some(path);
            let tasks = vec![self.load_games_for(paths), self.persist_config()];
            return Task::batch(tasks);
        }
        self.needs_database_path = true;
        Task::none()
    }

    fn prompt_for_lutris_database(&mut self) -> Task<Message> {
        Task::perform(select_lutris_database(), Message::LutrisDatabaseSelected)
    }

    fn prompt_for_lutris_icons(&mut self) -> Task<Message> {
        Task::perform(select_lutris_icons_dir(), Message::LutrisIconsPathSelected)
    }

    fn finish_game_selection(&mut self, game: Game) -> Task<Message> {
        let commands = self.prepare_selected_game(game);
        if commands.is_empty() {
            Task::none()
        } else {
            Task::batch(commands)
        }
    }

    fn prepare_selected_game(&mut self, game: Game) -> Vec<Task<Message>> {
        self.selected_game = Some(game.clone());
        self.search_results.clear();
        self.image_candidates.clear();
        self.download_statuses.clear();
        self.show_settings = false;
        self.current_images = CurrentGameImages::default();
        self.game_view_step = GameViewStep::Details;
        self.aspect_prompt = None;

        let mut commands = Vec::new();
        if let Some(paths) = self.lutris_paths.clone() {
            commands.push(Task::perform(
                check_current_images(game.slug.clone(), paths),
                Message::CurrentImagesLoaded,
            ));
        }

        if let Some(task) = self.request_close_prompt_window() {
            commands.push(task);
        }

        commands
    }

    fn begin_image_application(&mut self, image: GameImage) -> Task<Message> {
        if let Some(game) = &self.selected_game {
            let url = image.url.clone();
            if matches!(
                self.download_statuses.get(&url),
                Some(DownloadStatus::Downloading)
            ) {
                return Task::none();
            }

            self.download_statuses
                .insert(url.clone(), DownloadStatus::Downloading);
            self.aspect_prompt = None;

            let kind = image.kind.clone();
            let slug = game.slug.clone();
            let task_url = url.clone();
            let download_task = Task::perform(
                async move { download_full_image(task_url).await },
                move |result| Message::ImageDownloadCompleted {
                    url: url.clone(),
                    slug: slug.clone(),
                    kind: kind.clone(),
                    result,
                },
            );

            let mut commands = vec![download_task];
            if let Some(task) = self.request_close_prompt_window() {
                commands.push(task);
            }

            return Task::batch(commands);
        }

        Task::none()
    }

    fn apply_selected_boop_game(&mut self) -> Task<Message> {
        let Some(popup) = self.boop_popup.as_ref() else {
            return Task::none();
        };

        let selected_id = popup
            .selected_game_id
            .or_else(|| popup.matches.first().map(|entry| entry.game.id));
        let asset = popup.asset.clone();

        if let Some(id) = selected_id {
            if let Some(game) = self.games.iter().find(|g| g.id == id).cloned() {
                self.clear_boop_success_state();
                self.boop_popup = None;
                self.exit_after_boop = self.boop_only_mode;
                let mut commands = self.prepare_selected_game(game);
                commands.push(self.begin_image_application(asset.as_game_image()));
                if !self.boop_only_mode {
                    if let Some(task) = self.request_close_boop_window() {
                        commands.push(task);
                    }
                }
                return if commands.is_empty() {
                    Task::none()
                } else {
                    Task::batch(commands)
                };
            }
        }

        self.clear_boop_success_state();
        self.boop_notification = Some(SgdbNotification {
            title: "SGDBoop".into(),
            body: "Select a Lutris game to continue.".into(),
            kind: SgdbNotificationKind::Error,
        });
        if let Some(task) = self.ensure_boop_window() {
            task
        } else {
            Task::none()
        }
    }

    fn ensure_image_cached(&mut self, url: &str) -> Option<Task<Message>> {
        if self.image_cache.contains_key(url) || self.pending_images.contains(url) {
            return None;
        }

        let key = url.to_string();
        self.pending_images.insert(key.clone());
        Some(Task::perform(download_image(key.clone()), move |res| {
            Message::ImageLoaded(key.clone(), res)
        }))
    }

    fn default_boop_filter(&self, asset: &SgdbBoopAsset) -> String {
        if let Some(nonsteam) = asset.app_id.strip_prefix("nonsteam-") {
            return decode_nonsteam_label(nonsteam);
        }

        asset
            .app_id
            .strip_prefix("steam-")
            .or_else(|| asset.app_id.strip_prefix("steam:"))
            .unwrap_or(&asset.app_id)
            .to_string()
    }

    fn refresh_boop_matches(&mut self) {
        let Some(popup) = &mut self.boop_popup else {
            return;
        };

        if self.games.is_empty() {
            popup.matches.clear();
            popup.selected_game_id = None;
            return;
        }

        let filter = popup.filter_value.trim();
        let matches = if filter.is_empty() {
            self.games
                .iter()
                .take(POPUP_MATCH_LIMIT)
                .cloned()
                .map(|game| SgdbPopupMatch { game, score: 0 })
                .collect::<Vec<_>>()
        } else {
            let matcher = SkimMatcherV2::default();
            let mut scored: Vec<SgdbPopupMatch> = self
                .games
                .iter()
                .filter_map(|game| {
                    matcher
                        .fuzzy_match(&game.name, filter)
                        .or_else(|| matcher.fuzzy_match(&game.slug, filter))
                        .map(|score| SgdbPopupMatch {
                            game: game.clone(),
                            score,
                        })
                })
                .collect();

            scored.sort_by(|a, b| {
                b.score
                    .cmp(&a.score)
                    .then_with(|| a.game.name.cmp(&b.game.name))
            });
            scored.truncate(POPUP_MATCH_LIMIT);
            scored
        };

        let current_selection = popup.selected_game_id;
        popup.matches = matches;
        if popup
            .matches
            .iter()
            .any(|entry| Some(entry.game.id) == current_selection)
        {
            popup.selected_game_id = current_selection;
        } else {
            popup.selected_game_id = popup.matches.first().map(|entry| entry.game.id);
        }
    }

    fn read_boop_invocation() -> Option<BoopInvocation> {
        std::env::args()
            .skip(1)
            .find_map(|arg| Self::parse_boop_arg(&arg))
    }

    fn parse_boop_arg(arg: &str) -> Option<BoopInvocation> {
        let scheme = arg.trim();
        let Some(rest) = scheme.strip_prefix("sgdb://boop/") else {
            return None;
        };

        if rest.eq_ignore_ascii_case("test") {
            return Some(BoopInvocation::Test);
        }

        let mut segments = rest.split('/');
        let ty = segments.next()?.trim();
        let asset_type = SgdbAssetType::from_segment(ty)?;
        let asset_id = segments.next()?.trim();
        if asset_id.is_empty() {
            return None;
        }
        let mode = segments.next();
        let is_nonsteam = matches!(mode, Some(value) if value.eq_ignore_ascii_case("nonsteam"));

        Some(BoopInvocation::Apply(SgdbBoopRequest {
            asset_type,
            asset_id: asset_id.to_string(),
            is_nonsteam,
        }))
    }

    fn handle_boop_invocation(&mut self, invocation: BoopInvocation) -> Option<Task<Message>> {
        self.clear_boop_success_state();
        match invocation {
            BoopInvocation::Test => {
                self.pending_boop_request = None;
                self.boop_popup = None;
                self.exit_after_boop = false;
                self.boop_notification = Some(SgdbNotification {
                    title: "SGDBoop".into(),
                    body: "SGDBoop integration is working!".into(),
                    kind: SgdbNotificationKind::Info,
                });
                self.ensure_boop_window()
            }
            BoopInvocation::Apply(request) => {
                self.pending_boop_request = Some(request.clone());
                self.exit_after_boop = false;
                let mut tasks = vec![Self::start_boop_fetch(request)];
                if let Some(task) = self.ensure_boop_window() {
                    tasks.push(task);
                }
                Some(Task::batch(tasks))
            }
        }
    }

    fn start_boop_fetch(request: SgdbBoopRequest) -> Task<Message> {
        Task::perform(
            async move { Self::resolve_boop_asset(request).await },
            Message::BoopAssetFetched,
        )
    }

    async fn resolve_boop_asset(request: SgdbBoopRequest) -> Result<SgdbBoopAsset, String> {
        let response = boop::fetch_asset(
            request.asset_type.as_str(),
            &request.asset_id,
            request.is_nonsteam,
        )
        .await?;
        Ok(SgdbBoopAsset {
            kind: request.asset_type.image_kind(),
            url: response.asset_url,
            app_id: response.app_id,
        })
    }
}

async fn select_lutris_database() -> Option<PathBuf> {
    tokio::task::spawn_blocking(|| {
        rfd::FileDialog::new()
            .set_title("Select Lutris Database")
            .add_filter("Lutris database (pga.db)", &["db"])
            .pick_file()
    })
    .await
    .ok()
    .flatten()
}

async fn select_lutris_icons_dir() -> Option<PathBuf> {
    tokio::task::spawn_blocking(|| {
        rfd::FileDialog::new()
            .set_title("Select Lutris Icon Directory")
            .pick_folder()
    })
    .await
    .ok()
    .flatten()
}

fn decode_nonsteam_label(value: &str) -> String {
    let encoded = format!("label={}", value);
    form_urlencoded::parse(encoded.as_bytes())
        .find(|(key, _)| key == "label")
        .map(|(_, val)| val.into_owned())
        .unwrap_or_else(|| value.replace('+', " "))
}
