use iced::Background;
use iced::Border;
use iced::widget::container::Style;
use iced::widget::scrollable;

use crate::ui::style;

/// Scrollbar with accent color on hover/drag.
pub(in crate::ui) fn scrollable_style(
    _theme: &iced::Theme,
    status: scrollable::Status,
) -> scrollable::Style {
    let scroller_color = match status {
        scrollable::Status::Active { .. } => style::COLOR_SURFACE,
        scrollable::Status::Hovered { .. } => style::COLOR_ACCENT_DIM,
        scrollable::Status::Dragged { .. } => style::COLOR_ACCENT,
    };

    let rail = scrollable::Rail {
        background: None,
        border: Border::default(),
        scroller: scrollable::Scroller {
            background: Background::Color(scroller_color),
            border: Border {
                radius: style::BORDER_RADIUS.into(),
                ..Border::default()
            },
        },
    };

    scrollable::Style {
        container: Style::default(),
        vertical_rail: rail,
        horizontal_rail: rail,
        gap: None,
        auto_scroll: scrollable::AutoScroll {
            background: Background::Color(style::COLOR_SURFACE),
            border: Border::default(),
            shadow: iced::Shadow::default(),
            icon: style::COLOR_ACCENT,
        },
    }
}
