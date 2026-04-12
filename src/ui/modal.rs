//! Modal dialogs (interval editor).

use iced::Length;
use iced::widget::button;
use iced::widget::column;
use iced::widget::container;
use iced::widget::mouse_area;
use iced::widget::opaque;
use iced::widget::row;
use iced::widget::text;
use iced::widget::text_input;

use super::style;
use crate::state::AppState;
use crate::state::Message;

/// Render the update interval modal with click-outside-to-dismiss backdrop.
pub(super) fn interval_modal(state: &AppState) -> iced::Element<'_, Message> {
    let can_submit = state.update_interval_error.is_none();

    let mut items: Vec<iced::Element<'_, Message>> =
        vec![text("Update interval (ms)").size(style::TEXT_BODY).into()];

    if let Some(error) = &state.update_interval_error {
        items.push(text(error.as_str()).color(style::COLOR_ERROR).into());
    }

    if state.modal_dismiss_warning {
        items.push(
            text("Apply or discard changes before closing")
                .color(style::COLOR_WARNING)
                .into(),
        );
    }

    items.push(
        text_input("2500", &state.update_interval_input)
            .on_input(Message::UpdateIntervalInput)
            .into(),
    );

    items.push(
        row(vec![
            button(text("Cancel"))
                .on_press(Message::CloseIntervalModal)
                .into(),
            button(text("Apply"))
                .on_press_maybe(can_submit.then_some(Message::ApplyInterval))
                .into(),
        ])
        .spacing(style::SPACING)
        .into(),
    );

    let modal = container(column(items).spacing(style::SPACING))
        .padding(style::PADDING_MODAL)
        .width(Length::Shrink)
        .style(style::modal_style);

    // backdrop catches clicks, opaque() prevents them reaching the modal content
    let backdrop = mouse_area(
        container(opaque(modal))
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .style(style::overlay_style),
    )
    .on_press(Message::OverlayClicked);

    backdrop.into()
}
