//! Sidebar navigation item with accent indicator.

use iced::Length;
use iced::widget::button;
use iced::widget::container;
use iced::widget::row;
use iced::widget::text;

use crate::state::Message;
use crate::state::Page;
use crate::ui::style;

/// Sidebar nav item with active accent bar.
pub(in crate::ui) fn sidebar_item(
    label: &'static str,
    page: Page,
    active: Page,
) -> iced::Element<'static, Message> {
    let is_active = page == active;

    let accent = container(text(""))
        .width(Length::Fixed(style::SIDEBAR_ACCENT_WIDTH))
        .height(Length::Fill)
        .style(move |_theme: &iced::Theme| {
            if is_active {
                iced::widget::container::Style::default()
                    .background(iced::Background::Color(style::COLOR_ACCENT))
            } else {
                iced::widget::container::Style::default()
            }
        });

    let item_style = if is_active {
        item_active_style as fn(&iced::Theme) -> iced::widget::container::Style
    } else {
        item_inactive_style
    };

    let label = container(text(label).size(style::TEXT_LABEL))
        .padding(style::SIDEBAR_ITEM_PADDING)
        .width(Length::Fill)
        .style(item_style);

    let content = row(vec![accent.into(), label.into()]).height(Length::Shrink);

    button(content)
        .padding(0)
        .on_press(Message::NavigateTo(page))
        .style(move |_theme, status| {
            let is_hovered = matches!(status, button::Status::Hovered);

            let is_pressed = matches!(status, button::Status::Pressed);

            let text_color = if is_active || is_hovered {
                iced::Color::WHITE
            } else {
                style::COLOR_TEXT_DIM
            };

            let bg = if is_active {
                if is_hovered {
                    Some(iced::Background::Color(style::COLOR_SURFACE_HOVER))
                } else {
                    None
                }
            } else if is_pressed {
                Some(iced::Background::Color(style::COLOR_SURFACE))
            } else if is_hovered {
                Some(iced::Background::Color(style::COLOR_SURFACE_HOVER))
            } else {
                None
            };

            button::Style {
                background: bg,
                text_color,
                ..button::Style::default()
            }
        })
        .into()
}

fn item_active_style(theme: &iced::Theme) -> iced::widget::container::Style {
    let bg = theme.palette().background;
    let lighter = iced::Color::from_rgb(
        (bg.r * 1.3).min(1.0),
        (bg.g * 1.3).min(1.0),
        (bg.b * 1.3).min(1.0),
    );
    iced::widget::container::Style::default().background(iced::Background::Color(lighter))
}

fn item_inactive_style(_theme: &iced::Theme) -> iced::widget::container::Style {
    iced::widget::container::Style::default()
}
