//! Home panel with stat cards and status icon.

use std::sync::LazyLock;

use iced::Length;
use iced::alignment;
use iced::widget::column;
use iced::widget::container;
use iced::widget::image;
use iced::widget::row;
use iced::widget::text;

use super::style;
use crate::daw::UNKNOWN_PROJECT;
use crate::state::AppState;
use crate::state::Message;

/// Render the default home panel.
pub(super) fn home_view(state: &AppState) -> iced::Element<'_, Message> {
    let (daw_name, project_name, memory_usage, cpu_usage) = match &state.daw_status {
        Some(status) if status.is_running => (
            status.display_name.clone(),
            if state.settings.hide_project_name {
                "(hidden)".to_string()
            } else if status.project_name.trim().is_empty()
                || status.project_name.eq_ignore_ascii_case(UNKNOWN_PROJECT)
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
            text("Status").size(style::TEXT_HEADING).into(),
            status_icon(state.discord_connected),
        ])
        .align_y(alignment::Vertical::Center)
        .spacing(style::SPACING)
        .into(),
        row(vec![
            stat_card("DAW", daw_name),
            stat_card("Project", project_name),
        ])
        .spacing(style::SPACING)
        .into(),
        row(vec![
            stat_card("Memory", memory_usage),
            stat_card("CPU", cpu_usage),
        ])
        .spacing(style::SPACING)
        .into(),
    ])
    .spacing(style::SPACING)
    .into()
}

fn stat_card(title: &'static str, value: String) -> iced::Element<'static, Message> {
    let card = column(vec![
        text(title).size(style::TEXT_LABEL).into(),
        text(value).size(style::TEXT_BODY).into(),
    ])
    .spacing(style::SPACING_TIGHT);

    container(card)
        .padding(style::PADDING_CARD)
        .width(Length::Fill)
        .style(style::card_style)
        .into()
}

fn status_icon(connected: bool) -> iced::Element<'static, Message> {
    static RED_ICON: LazyLock<image::Handle> =
        LazyLock::new(|| image::Handle::from_bytes(include_bytes!("../assets/red.png").as_slice()));
    static GREEN_ICON: LazyLock<image::Handle> = LazyLock::new(|| {
        image::Handle::from_bytes(include_bytes!("../assets/green.png").as_slice())
    });

    let handle = if connected {
        GREEN_ICON.clone()
    } else {
        RED_ICON.clone()
    };

    container(
        image(handle)
            .width(Length::Fixed(style::STATUS_ICON_SIZE))
            .height(Length::Fixed(style::STATUS_ICON_SIZE)),
    )
    .padding(iced::Padding {
        top: style::STATUS_ICON_TOP_OFFSET,
        right: 0.0,
        bottom: 0.0,
        left: 0.0,
    })
    .into()
}
