//! Design tokens and reusable style functions.

use iced::Background;
use iced::Border;
use iced::Color;

// -- spacing --
pub(super) const SPACING: f32 = 12.0;
pub(super) const SPACING_TIGHT: f32 = 6.0;
pub(super) const SPACING_BUTTON_CONTENT: f32 = 8.0;

// -- padding --
pub(super) const PADDING_PAGE: f32 = 20.0;
pub(super) const PADDING_CARD: f32 = 12.0;
pub(super) const PADDING_BUTTON: f32 = 12.0;
pub(super) const PADDING_MODAL: f32 = 16.0;

// -- text sizes --
pub(super) const TEXT_HEADING: f32 = 24.0;
pub(super) const TEXT_BODY: f32 = 20.0;
pub(super) const TEXT_LABEL: f32 = 14.0;

// -- border --
pub(super) const BORDER_WIDTH: f32 = 1.0;
pub(super) const BORDER_RADIUS_CARD: f32 = 6.0;
pub(super) const BORDER_RADIUS_MODAL: f32 = 8.0;

// -- icon --
pub(super) const STATUS_ICON_SIZE: f32 = 12.0;
pub(super) const STATUS_ICON_TOP_OFFSET: f32 = 4.0;

// -- colors --
pub(super) const COLOR_ERROR: Color = Color::from_rgb(220.0 / 255.0, 60.0 / 255.0, 60.0 / 255.0);
pub(super) const COLOR_WARNING: Color = Color::from_rgb(1.0, 180.0 / 255.0, 50.0 / 255.0);
pub(super) const OVERLAY_OPACITY: f32 = 0.6;

/// Card container style: themed background with thin border.
pub(super) fn card_style(theme: &iced::Theme) -> iced::widget::container::Style {
    let palette = theme.palette();
    iced::widget::container::Style::default()
        .background(Background::Color(palette.background))
        .border(Border {
            color: palette.text,
            width: BORDER_WIDTH,
            radius: BORDER_RADIUS_CARD.into(),
        })
}

/// Modal container style: themed background with rounded border.
pub(super) fn modal_style(theme: &iced::Theme) -> iced::widget::container::Style {
    let palette = theme.palette();
    iced::widget::container::Style::default()
        .background(Background::Color(palette.background))
        .border(Border {
            color: palette.text,
            width: BORDER_WIDTH,
            radius: BORDER_RADIUS_MODAL.into(),
        })
}

/// Semi-transparent overlay backdrop.
pub(super) fn overlay_style(_theme: &iced::Theme) -> iced::widget::container::Style {
    iced::widget::container::Style::default().background(Background::Color(Color::from_rgba(
        0.0,
        0.0,
        0.0,
        OVERLAY_OPACITY,
    )))
}
