mod components;
mod pages;
mod strings;
pub(crate) mod style;
pub(crate) mod tray;

use iced::Length;
use iced::widget::container;
use iced::widget::row;

use crate::state::AppState;
use crate::state::Message;
use crate::state::Page;

/// Render the app layout: sidebar + active page.
pub(crate) fn view(state: &AppState) -> iced::Element<'_, Message> {
    let content = match state.active_page {
        Page::Home => pages::home_view(state),
        Page::Settings => pages::settings_view(state),
    };

    let page = container(content)
        .padding(style::PADDING_PAGE)
        .width(Length::Fill)
        .height(Length::Fill);

    row(vec![components::sidebar(state), page.into()])
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
