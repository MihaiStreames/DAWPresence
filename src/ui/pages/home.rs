//! Home panel with stat cards and status icon.

use iced::alignment;
use iced::widget::column;
use iced::widget::row;
use iced::widget::text;

use crate::daw::UNKNOWN_PROJECT;
use crate::state::AppState;
use crate::state::Message;
use crate::ui::components;
use crate::ui::strings;
use crate::ui::style;

/// Render the default home panel.
pub(in crate::ui) fn home_view(state: &AppState) -> iced::Element<'_, Message> {
    let (daw_name, project_name, memory_usage, cpu_usage) = match &state.daw_status {
        Some(status) if status.is_running => (
            status.display_name.clone(),
            if state.settings.hide_project_name {
                strings::PROJECT_HIDDEN.to_string()
            } else if status.project_name.trim().is_empty()
                || status.project_name.eq_ignore_ascii_case(UNKNOWN_PROJECT)
            {
                strings::NO_PROJECT_OPEN.to_string()
            } else {
                status.project_name.clone()
            },
            format!("{} MB", status.memory_mb),
            format!("{:.1}%", status.cpu_usage),
        ),
        _ => (
            strings::NO_DAW_DETECTED.to_string(),
            strings::NO_DAW_DETECTED.to_string(),
            strings::NO_DAW_DETECTED.to_string(),
            strings::NO_DAW_DETECTED.to_string(),
        ),
    };

    column(vec![
        row(vec![
            text(strings::STATUS).size(style::TEXT_HEADING).into(),
            components::status_icon(state.discord_connected),
        ])
        .align_y(alignment::Vertical::Center)
        .spacing(style::SPACING)
        .into(),
        row(vec![
            components::stat_card(strings::CARD_DAW, daw_name),
            components::stat_card(strings::CARD_PROJECT, project_name),
        ])
        .spacing(style::SPACING)
        .into(),
        row(vec![
            components::stat_card(strings::CARD_MEMORY, memory_usage),
            components::stat_card(strings::CARD_CPU, cpu_usage),
        ])
        .spacing(style::SPACING)
        .into(),
    ])
    .spacing(style::SPACING)
    .into()
}
