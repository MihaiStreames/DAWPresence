use iced::widget::container;
use iced::widget::text;

use crate::state::Message;
use crate::ui::style;

pub(in crate::ui) fn section_heading(label: &'static str) -> iced::Element<'static, Message> {
    container(
        text(label)
            .size(style::TEXT_LABEL)
            .color(style::COLOR_ACCENT),
    )
    .padding(iced::Padding {
        top: style::SPACING,
        right: 0.0,
        bottom: style::SECTION_BOTTOM_PADDING,
        left: 0.0,
    })
    .into()
}
