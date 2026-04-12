//! Top menu bar with toggle buttons.

use iced::alignment;
use iced::widget::button;
use iced::widget::container;
use iced::widget::row;
use iced::widget::text;

use super::style;
use crate::state::AppState;
use crate::state::Message;

/// Render the top menu bar.
pub(super) fn menu_bar(state: &AppState) -> iced::Element<'_, Message> {
    container(
        row(vec![
            toggle_button(
                "Hide project name",
                state.settings.hide_project_name,
                Message::ToggleHideProjectName(true),
                Message::ToggleHideProjectName(false),
            ),
            toggle_button(
                "Hide system usage",
                state.settings.hide_system_usage,
                Message::ToggleHideSystemUsage(true),
                Message::ToggleHideSystemUsage(false),
            ),
            toggle_button(
                "Minimize to tray",
                state.settings.close_to_tray,
                Message::ToggleCloseToTray(true),
                Message::ToggleCloseToTray(false),
            ),
            button(text("Update interval"))
                .padding(style::PADDING_BUTTON)
                .on_press(Message::OpenIntervalModal)
                .into(),
        ])
        .align_y(alignment::Vertical::Center)
        .spacing(style::SPACING),
    )
    .padding(0)
    .into()
}

fn toggle_button(
    label: &'static str,
    enabled: bool,
    on_message: Message,
    off_message: Message,
) -> iced::Element<'static, Message> {
    let message = if enabled { off_message } else { on_message };

    let label = if enabled {
        format!("[ON] {label}")
    } else {
        format!("[OFF] {label}")
    };

    button(row(vec![text(label).into()]).spacing(style::SPACING_BUTTON_CONTENT))
        .padding(style::PADDING_BUTTON)
        .on_press(message)
        .into()
}
