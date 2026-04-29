use iced::Background;
use iced::Border;
use iced::widget::button;

use crate::ui::style;

/// Accent-colored button style.
pub(in crate::ui) fn accent_button(theme: &iced::Theme, status: button::Status) -> button::Style {
    let text_color = theme.palette().background;

    let (bg, tc) = match status {
        button::Status::Active => (style::COLOR_ACCENT, text_color),
        button::Status::Hovered => (style::COLOR_ACCENT_HOVER, text_color),
        button::Status::Pressed => (style::COLOR_ACCENT_DIM, text_color),
        button::Status::Disabled => (style::COLOR_SURFACE, style::COLOR_TEXT_DIM),
    };

    button::Style {
        background: Some(Background::Color(bg)),
        text_color: tc,
        border: Border {
            radius: style::BORDER_RADIUS.into(),
            ..Border::default()
        },
        ..Default::default()
    }
}
