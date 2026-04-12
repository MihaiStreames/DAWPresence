//! Discord connection status icon.

use iced::Background;
use iced::Border;
use iced::Length;
use iced::widget::container;
use iced::widget::text;

use crate::state::Message;
use crate::ui::style;

/// Colored circle indicating Discord connection state.
pub(in crate::ui) fn status_icon(connected: bool) -> iced::Element<'static, Message> {
    let color = if connected {
        style::COLOR_ACCENT
    } else {
        style::COLOR_ERROR
    };

    let circle = container(text(""))
        .width(Length::Fixed(style::STATUS_ICON_SIZE))
        .height(Length::Fixed(style::STATUS_ICON_SIZE))
        .style(move |_theme: &iced::Theme| iced::widget::container::Style {
            background: Some(Background::Color(color)),
            border: Border {
                radius: (style::STATUS_ICON_SIZE / 2.0).into(),
                ..Border::default()
            },
            ..Default::default()
        });

    // outer wrapper for vertical offset to align with heading text
    container(circle)
        .padding(iced::Padding {
            top: style::STATUS_ICON_TOP_OFFSET,
            right: 0.0,
            bottom: 0.0,
            left: 0.0,
        })
        .into()
}
