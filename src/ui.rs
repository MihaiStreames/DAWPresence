use iced::widget::{button, column, container, image, opaque, row, stack, text, text_input};
use iced::{alignment, Background, Border, Color, Length};
use std::sync::LazyLock;

use crate::{AppState, Message};

/// Render the app UI
pub fn view(state: &AppState) -> iced::Element<'_, Message> {
    let base = container(
        column(vec![menu_bar(state), home_view(state)])
            .padding(20)
            .spacing(12),
    )
    .width(Length::Fill)
    .height(Length::Fill);

    if state.show_interval_modal {
        stack(vec![base.into(), opaque(interval_modal(state))]).into()
    } else {
        base.into()
    }
}

/// Render the top menu bar
fn menu_bar(state: &AppState) -> iced::Element<'_, Message> {
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
                .padding(12)
                .on_press(Message::OpenIntervalModal)
                .into(),
        ])
        .align_y(alignment::Vertical::Center)
        .spacing(12),
    )
    .padding(0)
    .into()
}

/// Render the default home panel
fn home_view(state: &AppState) -> iced::Element<'_, Message> {
    let (daw_name, project_name, memory_usage, cpu_usage) = match &state.daw_status {
        Some(status) if status.is_running => (
            status.display_name.clone(),
            if status.project_name.trim().is_empty()
                || status.project_name.eq_ignore_ascii_case("none")
            {
                "No project open".to_string()
            } else {
                status.project_name.clone()
            },
            format!("{} MB", status.memory_mb),
            format!("{:.1}%", status.cpu_usage),
        ),
        _ => (
            "No DAW detected".to_string(),
            "No DAW detected".to_string(),
            "No DAW detected".to_string(),
            "No DAW detected".to_string(),
        ),
    };

    column(vec![
        row(vec![
            text("Status").size(24).into(),
            status_icon(state.discord_connected).into(),
        ])
        .align_y(alignment::Vertical::Center)
        .spacing(8)
        .into(),
        row(vec![
            stat_card("DAW", daw_name),
            stat_card("Project", project_name),
        ])
        .spacing(12)
        .into(),
        row(vec![
            stat_card("Memory", memory_usage),
            stat_card("CPU", cpu_usage),
        ])
        .spacing(12)
        .into(),
    ])
    .spacing(12)
    .into()
}

/// Render a single stat card
fn stat_card(title: &'static str, value: String) -> iced::Element<'static, Message> {
    let card = column(vec![
        text(title).size(14).into(),
        text(value).size(20).into(),
    ])
    .spacing(6);

    container(card)
        .padding(12)
        .width(Length::Fill)
        .style(|theme: &iced::Theme| {
            let palette = theme.palette();
            iced::widget::container::Style::default()
                .background(Background::Color(palette.background))
                .border(Border {
                    color: palette.text,
                    width: 1.0,
                    radius: 6.0.into(),
                })
        })
        .into()
}

/// Render a toggle button
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

    button(row(vec![text(label).into()]).spacing(8))
        .padding(12)
        .on_press(message)
        .into()
}

/// Render the Discord RPC status icon
fn status_icon(connected: bool) -> image::Image<image::Handle> {
    static RED_ICON: LazyLock<image::Handle> =
        LazyLock::new(|| image::Handle::from_bytes(include_bytes!("assets/red.png").as_slice()));
    static GREEN_ICON: LazyLock<image::Handle> =
        LazyLock::new(|| image::Handle::from_bytes(include_bytes!("assets/green.png").as_slice()));

    let handle = if connected {
        GREEN_ICON.clone()
    } else {
        RED_ICON.clone()
    };

    image(handle)
        .width(Length::Fixed(16.0))
        .height(Length::Fixed(16.0))
}

/// Render the update interval modal
fn interval_modal(state: &AppState) -> iced::Element<'_, Message> {
    let error_text = state.update_interval_error.as_deref();
    let error_line = error_text
        .map(|message| text(message).color(Color::from_rgb8(220, 60, 60)).into())
        .unwrap_or_else(|| text("").into());

    let can_submit = state.update_interval_error.is_none();
    let modal = container(
        column(vec![
            text("Update interval (ms)").size(20).into(),
            error_line,
            text_input("2500", &state.update_interval_input)
                .on_input(Message::UpdateIntervalInput)
                .into(),
            row(vec![
                button(text("Cancel"))
                    .on_press(Message::CloseIntervalModal)
                    .into(),
                button(text("Apply"))
                    .on_press_maybe(can_submit.then_some(Message::ApplyInterval))
                    .into(),
            ])
            .spacing(12)
            .into(),
        ])
        .spacing(12),
    )
    .padding(16)
    .width(Length::Shrink)
    .style(|theme: &iced::Theme| {
        let palette = theme.palette();
        iced::widget::container::Style::default()
            .background(Background::Color(palette.background))
            .border(Border {
                color: palette.text,
                width: 1.0,
                radius: 8.0.into(),
            })
    });

    let overlay = container(modal)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .style(|_theme: &iced::Theme| {
            iced::widget::container::Style::default()
                .background(Background::Color(Color::from_rgba8(0, 0, 0, 0.6)))
        });

    overlay.into()
}
