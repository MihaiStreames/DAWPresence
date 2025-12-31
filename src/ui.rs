use iced::widget::{button, checkbox, column, container, row, text};

use crate::{AppState, Message, Panel};

/// Render the app UI
pub fn view(state: &AppState) -> iced::Element<'_, Message> {
    column(vec![menu_bar().into(), content_view(state).into()])
        .padding(20)
        .spacing(12)
        .into()
}

/// Render the top menu bar
fn menu_bar() -> iced::Element<'static, Message> {
    container(
        row(vec![
            button(text("File")).into(),
            button(text("Edit")).into(),
            button(text("Settings"))
                .on_press(Message::SelectPanel(Panel::Settings))
                .into(),
        ])
        .spacing(12),
    )
    .padding(8)
    .into()
}

/// Render the active content panel
fn content_view(state: &AppState) -> iced::Element<'_, Message> {
    match state.active_panel {
        Panel::Home => home_view(),
        Panel::Settings => settings_view(state),
    }
}

/// Render the default home panel
fn home_view() -> iced::Element<'static, Message> {
    column(vec![text("Use the menu bar to open Settings").into()])
        .spacing(12)
        .into()
}

/// Render the settings panel
fn settings_view(state: &AppState) -> iced::Element<'_, Message> {
    column(vec![
        text("Settings").size(24).into(),
        checkbox(state.settings.close_to_tray)
            .label("Minimize to tray on close")
            .on_toggle(Message::ToggleCloseToTray)
            .into(),
    ])
    .spacing(12)
    .into()
}
