//! Tray and window icon loading with LazyLock caching.

use std::sync::LazyLock;

use iced::window;
use tray_icon::Icon;

use crate::error::TrayError;

#[cfg(target_os = "windows")]
const ICON_RED_DATA: &[u8] = include_bytes!("../../../assets/app/red.ico");
#[cfg(not(target_os = "windows"))]
const ICON_RED_DATA: &[u8] = include_bytes!("../../../assets/app/red.png");

#[cfg(target_os = "windows")]
const ICON_BLUE_DATA: &[u8] = include_bytes!("../../../assets/app/blue.ico");
#[cfg(not(target_os = "windows"))]
const ICON_BLUE_DATA: &[u8] = include_bytes!("../../../assets/app/blue.png");

#[cfg(target_os = "windows")]
const ICON_MAIN_DATA: &[u8] = include_bytes!("../../../assets/app/main.ico");
#[cfg(not(target_os = "windows"))]
const ICON_MAIN_DATA: &[u8] = include_bytes!("../../../assets/app/main.png");

#[cfg(target_os = "windows")]
const ICON_FORMAT: image::ImageFormat = image::ImageFormat::Ico;
#[cfg(not(target_os = "windows"))]
const ICON_FORMAT: image::ImageFormat = image::ImageFormat::Png;

/// Cached decoded RGBA pixels (decoded once, cloned on each use)
static ICON_RED_RGBA: LazyLock<(Vec<u8>, u32, u32)> = LazyLock::new(|| decode_icon(ICON_RED_DATA));
static ICON_BLUE_RGBA: LazyLock<(Vec<u8>, u32, u32)> =
    LazyLock::new(|| decode_icon(ICON_BLUE_DATA));

fn decode_icon(data: &[u8]) -> (Vec<u8>, u32, u32) {
    let image = image::load_from_memory_with_format(data, ICON_FORMAT)
        .expect("embedded icon data must be valid")
        .into_rgba8();
    (image.to_vec(), image.width(), image.height())
}

fn cached_rgba(connected: bool) -> (Vec<u8>, u32, u32) {
    let (rgba, w, h) = if connected {
        &*ICON_BLUE_RGBA
    } else {
        &*ICON_RED_RGBA
    };
    (rgba.clone(), *w, *h)
}

/// Load tray icon from cached RGBA.
pub(super) fn load_tray_icon(connected: bool) -> Result<Icon, TrayError> {
    let (rgba, width, height) = cached_rgba(connected);
    Icon::from_rgba(rgba, width, height).map_err(|e| TrayError::IconFailed(e.to_string()))
}

/// Load window icon from cached RGBA.
pub(crate) fn load_window_icon() -> Result<window::Icon, TrayError> {
    static MAIN_RGBA: LazyLock<(Vec<u8>, u32, u32)> = LazyLock::new(|| decode_icon(ICON_MAIN_DATA));
    let (rgba, w, h) = &*MAIN_RGBA;
    window::icon::from_rgba(rgba.clone(), *w, *h).map_err(|e| TrayError::IconFailed(e.to_string()))
}
