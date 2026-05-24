use iced::Border;
use iced::Length;
use iced::widget::column;
use iced::widget::container;
use iced::widget::container::Style;
use iced::widget::text;

use crate::state::Message;
use crate::ui::style;

/// Bordered stat card with accent-colored title.
pub(in crate::ui) fn stat_card(
    title: &'static str,
    value: String,
) -> iced::Element<'static, Message> {
    let label = text(title)
        .size(style::TEXT_LABEL)
        .color(style::COLOR_ACCENT);

    let value = text(value).size(style::TEXT_LABEL);
    let card = column(vec![label.into(), value.into()]).spacing(style::SPACING_TIGHT);

    container(card)
        .padding(style::PADDING_CARD)
        .width(Length::Fill)
        .style(card_style)
        .into()
}

fn card_style(_theme: &iced::Theme) -> Style {
    Style::default().border(Border {
        color: style::COLOR_TEXT_DIM,
        width: style::BORDER_WIDTH,
        radius: style::BORDER_RADIUS_CARD.into(),
    })
}
