use iced::widget::column;
use iced::widget::container;
use iced::widget::scrollable;
use iced::widget::text;

use crate::state::AppState;
use crate::state::Message;
use crate::ui::components;
use crate::ui::strings;
use crate::ui::style;

/// Render the settings page.
pub(in crate::ui) fn settings_view(state: &AppState) -> iced::Element<'_, Message> {
    let content = column(vec![
        text(strings::SETTINGS_TITLE)
            .size(style::TEXT_HEADING)
            .into(),
        components::section_heading(strings::SECTION_PRIVACY),
        components::setting_toggle(
            strings::HIDE_PROJECT_NAME,
            strings::HIDE_PROJECT_NAME_DESC,
            state.settings.hide_project_name,
            Message::ToggleHideProjectName(!state.settings.hide_project_name),
        ),
        components::setting_toggle(
            strings::HIDE_SYSTEM_USAGE,
            strings::HIDE_SYSTEM_USAGE_DESC,
            state.settings.hide_system_usage,
            Message::ToggleHideSystemUsage(!state.settings.hide_system_usage),
        ),
        components::section_heading(strings::SECTION_BEHAVIOR),
        components::setting_toggle(
            strings::MINIMIZE_TO_TRAY,
            strings::MINIMIZE_TO_TRAY_DESC,
            state.settings.close_to_tray,
            Message::ToggleCloseToTray(!state.settings.close_to_tray),
        ),
        components::setting_toggle(
            strings::AUTO_START,
            strings::AUTO_START_DESC,
            state.auto_start_enabled,
            Message::ToggleAutoStart(!state.auto_start_enabled),
        ),
        components::section_heading(strings::SECTION_TIMING),
        interval_editor(state),
    ])
    .spacing(style::SPACING_TIGHT);

    scrollable(container(content).padding(iced::Padding {
        top: 0.0,
        right: style::SPACING,
        bottom: 0.0,
        left: 0.0,
    }))
    .style(components::scrollable_style)
    .into()
}

fn interval_editor(state: &AppState) -> iced::Element<'_, Message> {
    let has_changes = state.update_interval_input != state.settings.update_interval.to_string();
    let can_apply = state.update_interval_error.is_none() && has_changes;

    let (description, color) = if let Some(error) = &state.update_interval_error {
        (error.as_str(), style::COLOR_ERROR)
    } else if state.interval_applied {
        (strings::APPLIED, style::COLOR_ACCENT)
    } else {
        (strings::UPDATE_INTERVAL_DESC, style::COLOR_TEXT_DIM)
    };

    components::labeled_input(
        components::InputProps {
            label: strings::UPDATE_INTERVAL,
            description,
            description_color: color,
            placeholder: strings::UPDATE_INTERVAL_PLACEHOLDER,
            value: &state.update_interval_input,
            button_label: strings::APPLY,
            on_submit: can_apply.then_some(Message::ApplyInterval),
        },
        Message::UpdateIntervalInput,
    )
}
