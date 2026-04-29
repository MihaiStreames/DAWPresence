use iced::Border;
use iced::Length;
use iced::widget::button;
use iced::widget::column;
use iced::widget::container;
use iced::widget::row;
use iced::widget::text;
use iced::widget::text_input;

use super::accent_button;
use crate::state::Message;
use crate::ui::style;

/// Props for a labeled input field with action button.
pub(in crate::ui) struct InputProps<'a> {
    pub label: &'static str,
    pub description: &'a str,
    pub description_color: iced::Color,
    pub placeholder: &'static str,
    pub value: &'a str,
    pub button_label: &'static str,
    pub on_submit: Option<Message>,
}

/// Input field with label on left, input+button on right, feedback below label.
pub(in crate::ui) fn labeled_input<'a>(
    props: InputProps<'a>,
    on_input: impl Fn(String) -> Message + 'a,
) -> iced::Element<'a, Message> {
    let label_col = column(vec![
        text(props.label).size(style::TEXT_LABEL).into(),
        text(props.description)
            .size(style::TEXT_DESCRIPTION)
            .color(props.description_color)
            .into(),
    ])
    .spacing(style::LABEL_SPACING);

    let input_row = row(vec![
        text_input(props.placeholder, props.value)
            .on_input(on_input)
            .width(Length::Fixed(style::INPUT_WIDTH))
            .style(input_style)
            .into(),
        button(text(props.button_label).size(style::TEXT_LABEL))
            .padding(iced::Padding {
                top: style::PADDING_BUTTON_V,
                right: style::PADDING_BUTTON_H,
                bottom: style::PADDING_BUTTON_V,
                left: style::PADDING_BUTTON_H,
            })
            .style(accent_button)
            .on_press_maybe(props.on_submit)
            .into(),
    ])
    .spacing(style::SPACING_TIGHT)
    .align_y(iced::alignment::Vertical::Center);

    container(
        row(vec![
            container(label_col).width(Length::Fill).into(),
            input_row.into(),
        ])
        .align_y(iced::alignment::Vertical::Center),
    )
    .padding(style::SETTING_ROW_PADDING)
    .into()
}

fn input_style(theme: &iced::Theme, status: text_input::Status) -> text_input::Style {
    let palette = theme.palette();

    let border_color = match status {
        text_input::Status::Active => style::COLOR_TEXT_DIM,
        text_input::Status::Hovered => style::COLOR_ACCENT,
        text_input::Status::Focused { .. } => style::COLOR_ACCENT,
        text_input::Status::Disabled => style::COLOR_SURFACE,
    };

    text_input::Style {
        background: iced::Background::Color(palette.background),
        border: Border {
            color: border_color,
            width: style::BORDER_WIDTH,
            radius: style::BORDER_RADIUS.into(),
        },
        icon: palette.text,
        placeholder: style::COLOR_TEXT_DIM,
        value: palette.text,
        selection: style::COLOR_ACCENT_DIM,
    }
}
