//! Setting toggle component with label, description, and accent toggler.

use iced::Background;
use iced::Color;
use iced::Length;
use iced::widget::column;
use iced::widget::container;
use iced::widget::row;
use iced::widget::text;
use iced::widget::toggler;

use crate::state::Message;
use crate::ui::style;

/// Setting row with label, description, and toggler.
pub(in crate::ui) fn setting_toggle(
    label: &'static str,
    description: &'static str,
    enabled: bool,
    message: Message,
) -> iced::Element<'static, Message> {
    let label_col = column(vec![
        text(label).size(style::TEXT_LABEL).into(),
        text(description)
            .size(style::TEXT_DESCRIPTION)
            .color(style::COLOR_TEXT_DIM)
            .into(),
    ])
    .spacing(style::LABEL_SPACING);

    let toggle = toggler(enabled)
        .on_toggle(move |_| message.clone())
        .style(toggler_style);

    container(
        row(vec![
            container(label_col).width(Length::Fill).into(),
            toggle.into(),
        ])
        .align_y(iced::alignment::Vertical::Center),
    )
    .padding(style::SETTING_ROW_PADDING)
    .into()
}

fn toggler_style(_theme: &iced::Theme, status: toggler::Status) -> toggler::Style {
    let (bg, fg) = match status {
        toggler::Status::Active { is_toggled } => {
            if is_toggled {
                (style::COLOR_ACCENT, Color::WHITE)
            } else {
                (style::COLOR_SURFACE, style::COLOR_TEXT_DIM)
            }
        }
        toggler::Status::Hovered { is_toggled } => {
            if is_toggled {
                (style::COLOR_ACCENT_HOVER, Color::WHITE)
            } else {
                (style::COLOR_SURFACE_HOVER, style::COLOR_TEXT_DIM)
            }
        }
        toggler::Status::Disabled { is_toggled } => {
            if is_toggled {
                (style::COLOR_ACCENT_DIM, style::COLOR_TEXT_DIM)
            } else {
                (style::COLOR_SURFACE, style::COLOR_TEXT_DIM)
            }
        }
    };

    toggler::Style {
        background: Background::Color(bg),
        foreground: Background::Color(fg),
        background_border_width: 0.0,
        background_border_color: Color::TRANSPARENT,
        foreground_border_width: 0.0,
        foreground_border_color: Color::TRANSPARENT,
        text_color: None,
        border_radius: None,
        padding_ratio: 0.25,
    }
}
