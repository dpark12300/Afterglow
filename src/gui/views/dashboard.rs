use crate::app::{
    AspectAction, AspectPrompt, CurrentGameImages, DownloadStatus, ImageTab, Message, PreviewState,
    TRANSFORM_ACTIONS,
};
use crate::gui::scroll::ScrollRegion;
use crate::lutris::database::Game;
use crate::sources::traits::{GameImage, ImageKind, SearchResult};
use crate::style::{self, ActionButtonVariant, Palette};
use iced::widget::scrollable::{self as scrollables, Scrollbar};
use iced::widget::text::Wrapping;
use iced::widget::Id as ScrollId;
use iced::widget::{
    button, column, container, image, mouse_area, row, scrollable, text, Space, Stack,
};
use iced::{alignment, Alignment, ContentFit, Element, Length};
use std::collections::HashMap;

const MAX_TITLE_CHARS: usize = 18;
const COVER_WIDTH: f32 = 130.0;
const COVER_HEIGHT: f32 = 180.0;
const CARD_WIDTH: f32 = COVER_WIDTH;
const CARD_HEIGHT: f32 = COVER_HEIGHT;
const CARD_RADIUS: f32 = 12.0;
const PREVIEW_BOX_WIDTH: f32 = 320.0;
const PREVIEW_BOX_HEIGHT: f32 = 320.0;

pub(crate) struct DetailsContext<'a> {
    pub palette: Palette,
    pub game: &'a Game,
    pub current_images: &'a CurrentGameImages,
    pub can_search_images: bool,
}

pub(crate) struct SearchResultsContext<'a> {
    pub palette: Palette,
    pub is_searching: bool,
    pub search_results: &'a [SearchResult],
    pub image_cache: &'a HashMap<String, image::Handle>,
    pub hovered_search_result: Option<&'a str>,
    pub scroll_id: ScrollId,
}

pub(crate) struct ImageSelectionContext<'a> {
    pub palette: Palette,
    pub image_candidates: &'a [GameImage],
    pub selected_tab: ImageTab,
    pub image_cache: &'a HashMap<String, image::Handle>,
    pub download_statuses: &'a HashMap<String, DownloadStatus>,
    pub scroll_id: ScrollId,
}

pub(crate) struct AspectPromptContext<'a> {
    pub palette: Palette,
    pub prompt: &'a AspectPrompt,
}

pub(crate) fn details_view<'a>(ctx: DetailsContext<'a>) -> Element<'a, Message> {
    let DetailsContext {
        palette,
        game,
        current_images,
        can_search_images,
    } = ctx;

    let mut col = column![
        text(&game.name).size(30).color(palette.text),
        text(format!("Runner: {}", game.runner)).color(palette.text_muted),
        text(format!("ID: {}", game.id)).color(palette.text_muted),
    ]
    .spacing(20);

    let mut images_row = row![].spacing(20);
    if let Some(path) = &current_images.cover {
        images_row = images_row.push(
            column![
                text("Cover").color(palette.text_muted),
                container(image(path).width(Length::Fixed(150.0)))
                    .style(style::image_card(palette))
            ]
            .spacing(5),
        );
    }
    if let Some(path) = &current_images.banner {
        images_row = images_row.push(
            column![
                text("Banner").color(palette.text_muted),
                container(image(path).width(Length::Fixed(200.0)))
                    .style(style::image_card(palette))
            ]
            .spacing(5),
        );
    }
    if let Some(path) = &current_images.icon {
        images_row = images_row.push(
            column![
                text("Icon").color(palette.text_muted),
                container(image(path).width(Length::Fixed(64.0))).style(style::image_card(palette))
            ]
            .spacing(5),
        );
    }

    if current_images.cover.is_some()
        || current_images.banner.is_some()
        || current_images.icon.is_some()
    {
        col = col.push(text("Current Images:").size(20).color(palette.text));
        col = col.push(
            scrollable(images_row)
                .direction(scrollables::Direction::Horizontal(Scrollbar::default())),
        );
    }

    if can_search_images {
        col = col.push(
            button("Update Images")
                .on_press(Message::SearchGame)
                .style(style::btn_primary(palette)),
        );
    } else {
        col = col.push(
            text("Please configure SteamGridDB API Key in Settings to update images.")
                .color(palette.text_muted),
        );
    }

    container(col)
        .padding(20)
        .style(style::card(palette))
        .width(Length::Fill)
        .into()
}

pub(crate) fn search_results_view<'a>(ctx: SearchResultsContext<'a>) -> Element<'a, Message> {
    let SearchResultsContext {
        palette,
        is_searching,
        search_results,
        image_cache,
        hovered_search_result,
        scroll_id,
    } = ctx;

    let mut col = column![row![
        button("Back")
            .on_press(Message::BackToDetails)
            .style(style::btn_secondary(palette)),
        text("Search Results").size(30).color(palette.text)
    ]
    .spacing(20)
    .align_y(Alignment::Center)]
    .spacing(20);

    if is_searching {
        col = col.push(text("Searching...").color(palette.text_muted));
    } else if !search_results.is_empty() {
        col = col.push(text("Select a game match:").size(20).color(palette.text));

        let results_grid = search_results
            .iter()
            .fold(row![].spacing(20), |r, res| {
                let placeholder_cover = |label: &'static str| -> Element<Message> {
                    container(text(label).color(palette.text_muted))
                        .width(Length::Fixed(COVER_WIDTH))
                        .height(Length::Fixed(COVER_HEIGHT))
                        .align_x(alignment::Horizontal::Center)
                        .align_y(alignment::Vertical::Center)
                        .into()
                };

                let cover_visual: Element<Message> = if let Some(url) = &res.image_url {
                    if let Some(handle) = image_cache.get(url) {
                        image(handle.clone())
                            .width(Length::Fixed(COVER_WIDTH))
                            .height(Length::Fixed(COVER_HEIGHT))
                            .content_fit(ContentFit::Cover)
                            .border_radius(CARD_RADIUS)
                            .into()
                    } else {
                        placeholder_cover("Loading...")
                    }
                } else {
                    placeholder_cover("No Image")
                };

                let is_hovered = hovered_search_result == Some(res.id.as_str());
                let needs_scroll = res.name.chars().count() > MAX_TITLE_CHARS;
                let title_overlay_content: Element<Message> = if is_hovered && needs_scroll {
                    let scroll = scrollable(row![text(&res.name)
                        .size(14)
                        .color(palette.text)
                        .width(Length::Shrink)
                        .wrapping(Wrapping::None)])
                    .direction(scrollables::Direction::Horizontal(
                        Scrollbar::new().width(0).scroller_width(0).margin(0),
                    ))
                    .width(Length::Fill)
                    .height(Length::Shrink)
                    .id(title_scroll_id(&res.id));

                    container(scroll)
                        .width(Length::Fill)
                        .align_x(alignment::Horizontal::Center)
                        .into()
                } else {
                    let text = text(truncate_title(&res.name))
                        .size(14)
                        .color(palette.text)
                        .width(Length::Fill)
                        .wrapping(Wrapping::None);

                    container(text)
                        .width(Length::Fill)
                        .align_x(alignment::Horizontal::Center)
                        .into()
                };

                let overlay_bar = container(title_overlay_content)
                    .width(Length::Fill)
                    .align_x(alignment::Horizontal::Center)
                    .padding([4, 8])
                    .style(style::title_overlay(palette));

                let cover_with_overlay: Element<Message> = Stack::new()
                    .width(Length::Fixed(COVER_WIDTH))
                    .height(Length::Fixed(COVER_HEIGHT))
                    .clip(true)
                    .push(cover_visual)
                    .push(
                        container(overlay_bar)
                            .width(Length::Fill)
                            .height(Length::Fill)
                            .align_x(alignment::Horizontal::Center)
                            .align_y(alignment::Vertical::Bottom),
                    )
                    .into();

                let card_body = container(cover_with_overlay)
                    .width(Length::Fixed(CARD_WIDTH))
                    .height(Length::Fixed(CARD_HEIGHT))
                    .clip(true)
                    .style(style::cover_card(palette, is_hovered));

                let card_button = button(card_body)
                    .on_press(Message::SearchResultSelected(res.clone()))
                    .style(style::btn_card_plain(palette));

                let interactive_card = mouse_area(card_button)
                    .on_enter(Message::SearchResultHover(res.id.clone()))
                    .on_exit(Message::SearchResultHoverEnd(res.id.clone()));

                r.push(interactive_card)
            })
            .wrap();

        let results_scroll = scrollable(results_grid)
            .height(Length::Fill)
            .width(Length::Fill)
            .id(scroll_id.clone())
            .on_scroll(|viewport| {
                Message::ScrollViewportChanged(ScrollRegion::SearchResults, viewport)
            });

        let results_overlay = mouse_area(
            container(Space::new())
                .width(Length::Fill)
                .height(Length::Fill),
        )
        .on_scroll(|delta| Message::ScrollWheel(ScrollRegion::SearchResults, delta))
        .interaction(iced::mouse::Interaction::None);

        let results_scroll = Stack::new()
            .push(results_scroll)
            .push(results_overlay)
            .width(Length::Fill)
            .height(Length::Fill);

        col = col.push(results_scroll);
    } else {
        col = col.push(text("No results found.").color(palette.text_muted));
    }

    container(col)
        .padding(20)
        .style(style::card(palette))
        .width(Length::Fill)
        .into()
}

pub(crate) fn image_selection_view<'a>(ctx: ImageSelectionContext<'a>) -> Element<'a, Message> {
    let ImageSelectionContext {
        palette,
        image_candidates,
        selected_tab,
        image_cache,
        download_statuses,
        scroll_id,
    } = ctx;

    let mut col = column![row![
        button("Back")
            .on_press(Message::BackToSearchResults)
            .style(style::btn_secondary(palette)),
        text("Select Image").size(30).color(palette.text)
    ]
    .spacing(20)
    .align_y(Alignment::Center)]
    .spacing(20);

    let tab_btn = |label, tab| {
        let is_selected = tab == selected_tab;
        let btn = button(text(label))
            .padding([5, 15])
            .style(style::btn_tab(is_selected, palette));

        if is_selected {
            btn
        } else {
            btn.on_press(Message::SelectImageTab(tab))
        }
    };

    let tabs = row![
        tab_btn("Cover", ImageTab::Cover),
        tab_btn("Banner", ImageTab::Banner),
        tab_btn("Icon", ImageTab::Icon),
    ]
    .spacing(10);

    col = col.push(tabs);

    if !image_candidates.is_empty() {
        let filtered_images: Vec<&GameImage> = image_candidates
            .iter()
            .filter(|img| match (selected_tab, &img.kind) {
                (ImageTab::Cover, ImageKind::Cover) => true,
                (ImageTab::Banner, ImageKind::Banner) => true,
                (ImageTab::Icon, ImageKind::Icon) => true,
                _ => false,
            })
            .collect();

        if !filtered_images.is_empty() {
            col = col.push(
                text(format!("Available Images ({}):", filtered_images.len()))
                    .size(20)
                    .color(palette.text),
            );

            let images_grid = filtered_images
                .iter()
                .fold(row![].spacing(20), |r, img| {
                    let (w, h) = match selected_tab {
                        ImageTab::Cover => (150.0, 200.0),
                        ImageTab::Banner => (320.0, 120.0),
                        ImageTab::Icon => (128.0, 128.0),
                    };

                    let placeholder_visual = |label: &'static str| -> Element<Message> {
                        container(text(label).color(palette.text_muted))
                            .width(Length::Fixed(w))
                            .height(Length::Fixed(h))
                            .align_x(alignment::Horizontal::Center)
                            .align_y(alignment::Vertical::Center)
                            .style(style::image_candidate_placeholder(palette))
                            .into()
                    };

                    let preview_visual: Element<Message> =
                        if let Some(handle) = image_cache.get(&img.thumb) {
                            image(handle.clone())
                                .width(Length::Fixed(w))
                                .height(Length::Fixed(h))
                                .content_fit(ContentFit::Cover)
                                .border_radius(CARD_RADIUS)
                                .into()
                        } else {
                            placeholder_visual("Loading...")
                        };

                    let status = download_statuses
                        .get(&img.url)
                        .copied()
                        .unwrap_or(DownloadStatus::Idle);

                    let (button_label, button_variant) = match status {
                        DownloadStatus::Idle => ("⬇", ActionButtonVariant::Primary),
                        DownloadStatus::Downloading => ("⏳", ActionButtonVariant::Primary),
                        DownloadStatus::Success => ("✅", ActionButtonVariant::Success),
                    };

                    let mut action_button = button(button_label)
                        .style(style::btn_action_overlay(button_variant, palette));

                    if status == DownloadStatus::Idle {
                        action_button = action_button.on_press(Message::ApplyImage((*img).clone()));
                    }

                    let action_bar = container(action_button)
                        .width(Length::Fill)
                        .align_x(alignment::Horizontal::Right)
                        .padding([6, 12])
                        .style(style::image_action_overlay());

                    let card_content: Element<Message> = Stack::new()
                        .width(Length::Fixed(w))
                        .height(Length::Fixed(h))
                        .clip(true)
                        .push(preview_visual)
                        .push(
                            container(action_bar)
                                .width(Length::Fill)
                                .height(Length::Fill)
                                .align_y(alignment::Vertical::Bottom),
                        )
                        .into();

                    let card: Element<Message> = container(card_content)
                        .width(Length::Fixed(w))
                        .height(Length::Fixed(h))
                        .clip(true)
                        .style(style::cover_card(palette, false))
                        .into();

                    r.push(card)
                })
                .wrap();

            let image_scroll = scrollable(images_grid)
                .height(Length::Fill)
                .width(Length::Fill)
                .id(scroll_id.clone())
                .on_scroll(|viewport| {
                    Message::ScrollViewportChanged(ScrollRegion::ImageSelection, viewport)
                });

            let image_overlay = mouse_area(
                container(Space::new())
                    .width(Length::Fill)
                    .height(Length::Fill),
            )
            .on_scroll(|delta| Message::ScrollWheel(ScrollRegion::ImageSelection, delta))
            .interaction(iced::mouse::Interaction::None);

            let image_scroll = Stack::new()
                .push(image_scroll)
                .push(image_overlay)
                .width(Length::Fill)
                .height(Length::Fill);

            col = col.push(image_scroll);
        } else {
            col = col.push(text("No images found for this category.").color(palette.text_muted));
        }
    } else {
        col = col.push(text("No images found.").color(palette.text_muted));
    }

    container(col)
        .padding(20)
        .style(style::card(palette))
        .width(Length::Fill)
        .into()
}

pub(crate) fn aspect_prompt_window<'a>(ctx: AspectPromptContext<'a>) -> Element<'a, Message> {
    let AspectPromptContext { palette, prompt } = ctx;
    let (actual_w, actual_h) = prompt.actual_dimensions();
    let (target_w, target_h) = prompt.target_dimensions();

    let header = column![
        text("Aspect Ratio Mismatch").size(28).color(palette.text),
        text(format!(
            "Source {}×{} → Target {}×{}",
            actual_w, actual_h, target_w, target_h
        ))
        .color(palette.text_muted),
        text("Choose how the image should fit the Lutris slot. Each option shows the final framing before saving.")
            .color(palette.text_muted)
    ]
    .spacing(6);

    let options_row = TRANSFORM_ACTIONS
        .iter()
        .fold(
            row![].spacing(20).align_y(Alignment::Start),
            |row, action| row.push(aspect_option_card(*action, palette, prompt)),
        )
        .wrap();

    let actions = row![
        Space::new().width(Length::Fill),
        button("Cancel")
            .on_press(Message::CancelAspectPrompt)
            .style(style::btn_secondary(palette))
    ]
    .spacing(12);

    let content = column![
        header,
        text("Transformation Options").size(20).color(palette.text),
        options_row,
        actions
    ]
    .spacing(20);

    let card = container(content)
        .width(Length::Fill)
        .max_width(960.0)
        .padding(24)
        .style(style::modal_card(palette));

    container(card)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .padding(20)
        .style(style::app_container(palette))
        .into()
}

pub(crate) fn empty_state_view(palette: Palette) -> Element<'static, Message> {
    container(text("Select a game").size(20).color(palette.text_muted))
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .style(style::card(palette))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

pub(crate) fn title_scroll_id(id: &str) -> ScrollId {
    ScrollId::from(format!("search-title-{id}"))
}

pub(crate) fn title_needs_scroll(id: &str, results: &[SearchResult]) -> bool {
    results
        .iter()
        .find(|res| res.id == id)
        .map(|res| res.name.chars().count() > MAX_TITLE_CHARS)
        .unwrap_or(false)
}

fn truncate_title(title: &str) -> String {
    let mut chars = title.chars();
    if chars.clone().count() <= MAX_TITLE_CHARS {
        title.to_string()
    } else {
        let visible: String = chars
            .by_ref()
            .take(MAX_TITLE_CHARS.saturating_sub(1))
            .collect();
        format!("{}…", visible.trim_end())
    }
}

fn preview_box_size() -> (f32, f32) {
    (PREVIEW_BOX_WIDTH, PREVIEW_BOX_HEIGHT)
}

fn fit_preview_size(dimensions: (u32, u32)) -> (f32, f32) {
    let (max_w, max_h) = preview_box_size();
    let (w, h) = dimensions;
    if w == 0 || h == 0 {
        return (max_w, max_h);
    }
    let width = w as f32;
    let height = h as f32;
    let scale = f32::min(max_w / width, max_h / height);
    (width * scale, height * scale)
}

fn aspect_option_card<'a>(
    action: AspectAction,
    palette: Palette,
    prompt: &'a AspectPrompt,
) -> Element<'a, Message> {
    let preview_state = prompt.preview_state(action);
    let (box_w, box_h) = preview_box_size();
    let (fit_w, fit_h) = fit_preview_size(prompt.target_dimensions());
    let mut is_clickable = false;

    let preview_box: Element<Message> = match preview_state {
        PreviewState::Pending => container(text("Rendering…").color(palette.text_muted))
            .padding(10)
            .width(Length::Fixed(box_w))
            .height(Length::Fixed(box_h))
            .align_x(alignment::Horizontal::Center)
            .align_y(alignment::Vertical::Center)
            .style(style::preview_card(palette))
            .into(),
        PreviewState::Ready(handle) => {
            is_clickable = true;
            container(
                image(handle.clone())
                    .width(Length::Fixed(fit_w))
                    .height(Length::Fixed(fit_h))
                    .content_fit(ContentFit::Fill),
            )
            .padding(10)
            .width(Length::Fixed(box_w))
            .height(Length::Fixed(box_h))
            .align_x(alignment::Horizontal::Center)
            .align_y(alignment::Vertical::Center)
            .style(style::preview_card(palette))
            .into()
        }
        PreviewState::Error(err) => container(text(err).color(palette.accent))
            .padding(10)
            .width(Length::Fixed(box_w))
            .height(Length::Fixed(box_h))
            .align_x(alignment::Horizontal::Center)
            .align_y(alignment::Vertical::Center)
            .style(style::preview_card(palette))
            .into(),
    };

    let mut card_content = column![
        text(action_label(action)).size(20).color(palette.text),
        text(action_description(action))
            .size(14)
            .color(palette.text_muted),
        preview_box,
    ]
    .spacing(10);

    if let PreviewState::Error(err) = preview_state {
        card_content = card_content.push(text(err).color(palette.accent));
    }

    let mut button_card = button(card_content)
        .width(Length::Fixed(260.0))
        .height(Length::Shrink)
        .padding(16)
        .style(style::btn_card_interactive(palette));
    if is_clickable {
        button_card = button_card.on_press(Message::ConfirmAspectAction(action));
    }

    button_card.into()
}

fn action_label(action: AspectAction) -> &'static str {
    match action {
        AspectAction::Stretch => "Stretch",
        AspectAction::Cover => "Cover",
        AspectAction::Contain => "Contain",
        AspectAction::Original => "Original",
    }
}

fn action_description(action: AspectAction) -> &'static str {
    match action {
        AspectAction::Stretch => "Fill the frame by scaling width & height independently.",
        AspectAction::Cover => "Crop the image after scaling to cover the slot edge to edge.",
        AspectAction::Contain => "Letterbox or pillarbox to keep the original composition.",
        AspectAction::Original => "Keep the downloaded dimensions without changes.",
    }
}
