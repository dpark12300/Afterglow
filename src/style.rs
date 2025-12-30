use crate::config::ThemeVariant;
use iced::widget::{button, container};
use iced::{border, Background, Border, Color, Shadow, Theme, Vector};

#[derive(Debug, Clone, Copy)]
pub struct Palette {
    pub background: Color,
    pub sidebar: Color,
    pub surface: Color,
    pub surface_hover: Color,
    pub accent: Color,
    pub accent_hover: Color,
    pub success: Color,
    pub success_hover: Color,
    pub text: Color,
    pub text_muted: Color,
    pub border: Color,
    pub is_light: bool,
}

pub fn palette(theme: ThemeVariant) -> Palette {
    match theme {
        ThemeVariant::Dark => Palette {
            background: Color::from_rgb(0.09, 0.09, 0.11),
            sidebar: Color::from_rgb(0.07, 0.07, 0.08),
            surface: Color::from_rgb(0.13, 0.13, 0.16),
            surface_hover: Color::from_rgb(0.16, 0.16, 0.20),
            accent: Color::from_rgb(0.36, 0.36, 1.0),
            accent_hover: Color::from_rgb(0.42, 0.42, 1.0),
            success: Color::from_rgb(0.20, 0.65, 0.33),
            success_hover: Color::from_rgb(0.24, 0.70, 0.38),
            text: Color::from_rgb(0.90, 0.90, 0.92),
            text_muted: Color::from_rgb(0.55, 0.55, 0.60),
            border: Color::from_rgb(0.20, 0.20, 0.24),
            is_light: false,
        },
        ThemeVariant::Light => Palette {
            background: Color::from_rgb(0.96, 0.96, 0.97),
            sidebar: Color::from_rgb(0.99, 0.99, 1.0),
            surface: Color::from_rgb(1.0, 1.0, 1.0),
            surface_hover: Color::from_rgb(0.93, 0.93, 0.95),
            accent: Color::from_rgb(0.27, 0.45, 0.98),
            accent_hover: Color::from_rgb(0.37, 0.53, 1.0),
            success: Color::from_rgb(0.20, 0.65, 0.33),
            success_hover: Color::from_rgb(0.26, 0.72, 0.40),
            text: Color::from_rgb(0.11, 0.11, 0.18),
            text_muted: Color::from_rgb(0.45, 0.45, 0.52),
            border: Color::from_rgb(0.82, 0.82, 0.88),
            is_light: true,
        },
    }
}

pub fn app_container(palette: Palette) -> impl Fn(&Theme) -> container::Style {
    move |_theme| container::Style {
        background: Some(palette.background.into()),
        text_color: Some(palette.text),
        ..Default::default()
    }
}

pub fn sidebar(palette: Palette) -> impl Fn(&Theme) -> container::Style {
    move |_theme| container::Style {
        background: Some(palette.sidebar.into()),
        border: Border {
            color: palette.border,
            width: 1.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    }
}

pub fn card(palette: Palette) -> impl Fn(&Theme) -> container::Style {
    move |_theme| container::Style {
        background: Some(palette.surface.into()),
        border: Border {
            color: palette.border,
            width: 1.0,
            radius: 12.0.into(),
        },
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.15),
            offset: Vector::new(0.0, 4.0),
            blur_radius: 10.0,
        },
        ..Default::default()
    }
}

pub fn modal_scrim() -> impl Fn(&Theme) -> container::Style {
    |_theme| container::Style {
        background: Some(Color::from_rgba(0.0, 0.0, 0.0, 0.6).into()),
        ..Default::default()
    }
}

pub fn modal_card(palette: Palette) -> impl Fn(&Theme) -> container::Style {
    move |_theme| container::Style {
        background: Some(palette.surface.into()),
        border: Border {
            color: palette.border,
            width: 1.0,
            radius: 20.0.into(),
        },
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.35),
            offset: Vector::new(0.0, 16.0),
            blur_radius: 40.0,
        },
        ..Default::default()
    }
}

pub fn preview_card(palette: Palette) -> impl Fn(&Theme) -> container::Style {
    move |_theme| container::Style {
        background: Some(palette.surface_hover.into()),
        border: Border {
            color: palette.border,
            width: 1.0,
            radius: 12.0.into(),
        },
        ..Default::default()
    }
}

pub fn image_card(palette: Palette) -> impl Fn(&Theme) -> container::Style {
    move |_theme| container::Style {
        background: Some(palette.surface.into()),
        border: Border {
            color: palette.border,
            width: 1.0,
            radius: 8.0.into(),
        },
        ..Default::default()
    }
}

pub fn cover_card(palette: Palette, is_hovered: bool) -> impl Fn(&Theme) -> container::Style {
    move |_theme| {
        let border_color = if is_hovered {
            palette.accent
        } else {
            palette.border
        };

        container::Style {
            background: Some(palette.surface.into()),
            border: Border {
                color: border_color,
                width: 1.0,
                radius: 12.0.into(),
            },
            ..Default::default()
        }
    }
}

pub fn title_overlay(palette: Palette) -> impl Fn(&Theme) -> container::Style {
    move |_theme| {
        let overlay_color = if palette.is_light {
            Color::from_rgba(1.0, 1.0, 1.0, 0.85)
        } else {
            Color::from_rgba(0.0, 0.0, 0.0, 0.75)
        };

        container::Style {
            background: Some(overlay_color.into()),
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: border::Radius::default().bottom(12.0),
            },
            ..Default::default()
        }
    }
}

pub fn image_action_overlay() -> impl Fn(&Theme) -> container::Style {
    |_theme| container::Style {
        background: None,
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: border::Radius::default().bottom(12.0),
        },
        ..Default::default()
    }
}

pub fn image_candidate_placeholder(palette: Palette) -> impl Fn(&Theme) -> container::Style {
    move |_theme| container::Style {
        background: Some(palette.surface_hover.into()),
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 12.0.into(),
        },
        ..Default::default()
    }
}

pub fn btn_card_plain(palette: Palette) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_theme, _status| button::Style {
        background: None,
        text_color: palette.text,
        border: Border {
            width: 0.0,
            radius: 12.0.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}

pub fn btn_card_interactive(palette: Palette) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_theme, status| {
        let base = button::Style {
            background: Some(palette.surface.into()),
            text_color: palette.text,
            border: Border {
                color: palette.border,
                width: 1.0,
                radius: 16.0.into(),
            },
            shadow: Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.15),
                offset: Vector::new(0.0, 2.0),
                blur_radius: 6.0,
            },
            ..Default::default()
        };

        match status {
            button::Status::Hovered => button::Style {
                background: Some(palette.surface_hover.into()),
                border: Border {
                    color: palette.accent,
                    width: 2.0,
                    ..base.border
                },
                shadow: Shadow {
                    color: Color::from_rgba(0.0, 0.0, 0.0, 0.3),
                    offset: Vector::new(0.0, 6.0),
                    blur_radius: 18.0,
                },
                ..base
            },
            button::Status::Pressed => button::Style {
                background: Some(palette.surface.into()),
                border: Border {
                    color: palette.accent,
                    width: 2.0,
                    ..base.border
                },
                shadow: Shadow {
                    color: Color::from_rgba(0.0, 0.0, 0.0, 0.25),
                    offset: Vector::new(0.0, 3.0),
                    blur_radius: 10.0,
                },
                ..base
            },
            _ => base,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionButtonVariant {
    Primary,
    Success,
}

pub fn btn_action_overlay(
    variant: ActionButtonVariant,
    palette: Palette,
) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_theme, status| {
        let (base_color, hover_color) = match variant {
            ActionButtonVariant::Primary => (palette.accent, palette.accent_hover),
            ActionButtonVariant::Success => (palette.success, palette.success_hover),
        };

        let base = button::Style {
            background: Some(base_color.into()),
            text_color: Color::WHITE,
            border: Border {
                radius: 20.0.into(),
                ..Default::default()
            },
            ..Default::default()
        };

        match status {
            button::Status::Hovered => button::Style {
                background: Some(hover_color.into()),
                ..base
            },
            button::Status::Pressed => button::Style {
                background: Some(base_color.into()),
                ..base
            },
            _ => base,
        }
    }
}

pub fn btn_primary(palette: Palette) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_theme, status| {
        let base = button::Style {
            background: Some(palette.accent.into()),
            text_color: Color::WHITE,
            border: Border {
                radius: 8.0.into(),
                ..Default::default()
            },
            ..Default::default()
        };

        match status {
            button::Status::Hovered => button::Style {
                background: Some(palette.accent_hover.into()),
                ..base
            },
            button::Status::Pressed => button::Style {
                background: Some(palette.accent.into()),
                ..base
            },
            _ => base,
        }
    }
}

pub fn btn_secondary(palette: Palette) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_theme, status| {
        let base = button::Style {
            background: None,
            text_color: palette.text,
            border: Border {
                color: palette.border,
                width: 1.0,
                radius: 8.0.into(),
            },
            ..Default::default()
        };

        match status {
            button::Status::Hovered => button::Style {
                background: Some(palette.surface.into()),
                border: Border {
                    color: palette.accent,
                    width: 1.0,
                    radius: 8.0.into(),
                },
                ..base
            },
            _ => base,
        }
    }
}

pub fn btn_nav(
    is_selected: bool,
    palette: Palette,
) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_theme, status| {
        let base = button::Style {
            background: if is_selected {
                Some(palette.surface.into())
            } else {
                None
            },
            text_color: if is_selected {
                palette.accent
            } else {
                palette.text_muted
            },
            border: Border {
                radius: 6.0.into(),
                ..Default::default()
            },
            ..Default::default()
        };

        match status {
            button::Status::Hovered => button::Style {
                background: Some(palette.surface.into()),
                text_color: palette.text,
                ..base
            },
            _ => base,
        }
    }
}

pub fn btn_tab(
    is_selected: bool,
    palette: Palette,
) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_theme, status| {
        let base = button::Style {
            background: if is_selected {
                Some(palette.accent.into())
            } else {
                Some(palette.surface.into())
            },
            text_color: if is_selected {
                Color::WHITE
            } else {
                palette.text_muted
            },
            border: Border {
                radius: 20.0.into(),
                ..Default::default()
            },
            ..Default::default()
        };

        match status {
            button::Status::Hovered => {
                if is_selected {
                    base
                } else {
                    button::Style {
                        background: Some(palette.surface_hover.into()),
                        text_color: palette.text,
                        ..base
                    }
                }
            }
            _ => base,
        }
    }
}

pub fn text_input(
    palette: Palette,
) -> impl Fn(&Theme, iced::widget::text_input::Status) -> iced::widget::text_input::Style {
    move |_theme, status| {
        let active = iced::widget::text_input::Style {
            background: Background::Color(palette.surface),
            border: Border {
                radius: 8.0.into(),
                width: 1.0,
                color: palette.border,
            },
            icon: palette.text_muted,
            placeholder: palette.text_muted,
            value: palette.text,
            selection: palette.accent,
        };

        match status {
            iced::widget::text_input::Status::Focused { .. } => iced::widget::text_input::Style {
                border: Border {
                    color: palette.accent,
                    ..active.border
                },
                ..active
            },
            iced::widget::text_input::Status::Hovered => iced::widget::text_input::Style {
                border: Border {
                    color: palette.text_muted,
                    ..active.border
                },
                ..active
            },
            _ => active,
        }
    }
}
