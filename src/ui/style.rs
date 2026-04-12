//! Global design tokens shared across components and pages.

use iced::Color;

// -- spacing --
pub(super) const SPACING: f32 = 12.0;
pub(super) const SPACING_TIGHT: f32 = 6.0;

// -- padding --
pub(super) const PADDING_PAGE: f32 = 20.0;
pub(super) const PADDING_CARD: f32 = 12.0;
pub(super) const PADDING_BUTTON_V: f32 = 7.0;
pub(super) const PADDING_BUTTON_H: f32 = 12.0;

// -- sidebar --
pub(super) const SIDEBAR_WIDTH: f32 = 80.0;
pub(super) const SIDEBAR_ITEM_PADDING: f32 = 12.0;
pub(super) const SIDEBAR_ITEM_SPACING: f32 = 2.0;
pub(super) const SIDEBAR_ACCENT_WIDTH: f32 = 3.0;

// -- settings --
pub(super) const TEXT_DESCRIPTION: f32 = 12.0;
pub(super) const SETTING_ROW_PADDING: f32 = 10.0;
pub(super) const SECTION_BOTTOM_PADDING: f32 = 4.0;
pub(super) const INPUT_WIDTH: f32 = 100.0;
pub(super) const LABEL_SPACING: f32 = 2.0;

// -- text sizes --
pub(super) const TEXT_HEADING: f32 = 24.0;
pub(super) const TEXT_LABEL: f32 = 14.0;

// -- border --
pub(super) const BORDER_WIDTH: f32 = 1.0;
pub(super) const BORDER_RADIUS: f32 = 4.0;
pub(super) const BORDER_RADIUS_CARD: f32 = 6.0;

// -- icon --
pub(super) const STATUS_ICON_SIZE: f32 = 12.0;
pub(super) const STATUS_ICON_TOP_OFFSET: f32 = 4.0;

// -- colors --
pub(super) const COLOR_ERROR: Color = Color::from_rgb(200.0 / 255.0, 80.0 / 255.0, 80.0 / 255.0);
pub(super) const COLOR_ACCENT: Color = Color::from_rgb(88.0 / 255.0, 166.0 / 255.0, 1.0);
pub(super) const COLOR_ACCENT_HOVER: Color = Color::from_rgb(108.0 / 255.0, 180.0 / 255.0, 1.0);
pub(super) const COLOR_ACCENT_DIM: Color =
    Color::from_rgb(60.0 / 255.0, 120.0 / 255.0, 200.0 / 255.0);
pub(super) const COLOR_TEXT_DIM: Color =
    Color::from_rgb(140.0 / 255.0, 140.0 / 255.0, 140.0 / 255.0);
pub(super) const COLOR_SURFACE: Color = Color::from_rgb(50.0 / 255.0, 50.0 / 255.0, 50.0 / 255.0);
pub(super) const COLOR_SURFACE_HOVER: Color =
    Color::from_rgb(65.0 / 255.0, 65.0 / 255.0, 65.0 / 255.0);
