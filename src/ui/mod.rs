//! Main window view rendering.

mod home;
mod menu;
mod modal;
mod style;
pub(crate) mod tray;

use iced::Length;
use iced::widget::column;
use iced::widget::container;
use iced::widget::stack;

use crate::state::AppState;
use crate::state::Message;

/// Render the app UI.
pub(crate) fn view(state: &AppState) -> iced::Element<'_, Message> {
    let base = container(
        column(vec![menu::menu_bar(state), home::home_view(state)])
            .padding(style::PADDING_PAGE)
            .spacing(style::SPACING),
    )
    .width(Length::Fill)
    .height(Length::Fill);

    if state.show_interval_modal {
        stack(vec![base.into(), modal::interval_modal(state)]).into()
    } else {
        base.into()
    }
}
