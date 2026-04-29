use iced::Length;
use iced::widget::column;
use iced::widget::container;

use super::sidebar_item;
use crate::state::AppState;
use crate::state::Message;
use crate::state::Page;
use crate::ui::strings;
use crate::ui::style;

/// Render the sidebar with page navigation.
pub(in crate::ui) fn sidebar(state: &AppState) -> iced::Element<'_, Message> {
    let items = column(vec![
        sidebar_item(strings::NAV_HOME, Page::Home, state.active_page),
        sidebar_item(strings::NAV_SETTINGS, Page::Settings, state.active_page),
    ])
    .spacing(style::SIDEBAR_ITEM_SPACING);

    container(items)
        .width(Length::Fixed(style::SIDEBAR_WIDTH))
        .height(Length::Fill)
        .padding(iced::Padding {
            top: style::PADDING_PAGE,
            right: 0.0,
            bottom: 0.0,
            left: 0.0,
        })
        .style(sidebar_bg)
        .into()
}

fn sidebar_bg(theme: &iced::Theme) -> iced::widget::container::Style {
    let bg = theme.palette().background;
    let darker = iced::Color::from_rgb(
        (bg.r * 0.85).max(0.0),
        (bg.g * 0.85).max(0.0),
        (bg.b * 0.85).max(0.0),
    );
    iced::widget::container::Style::default().background(iced::Background::Color(darker))
}
