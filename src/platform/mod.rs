#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(target_os = "windows")]
pub mod windows;

use tao::monitor::MonitorHandle;
use tao::window::Window;
use crate::display_config::{MonitorId, PanelType, DisplayCategory};
use std::sync::mpsc;

pub fn get_monitor_id(monitor: &MonitorHandle) -> MonitorId {
    #[cfg(target_os = "macos")]
    return macos::get_monitor_id(monitor);
    #[cfg(target_os = "windows")]
    return windows::get_monitor_id(monitor);
}

pub fn is_internal_display(monitor: &MonitorHandle) -> bool {
    #[cfg(target_os = "macos")]
    return macos::is_internal_display(monitor);
    #[cfg(target_os = "windows")]
    return windows::is_internal_display(monitor);
}

pub fn get_physical_size_mm(monitor: &MonitorHandle) -> Option<(f32, f32)> {
    #[cfg(target_os = "macos")]
    return macos::get_physical_size_mm(monitor);
    #[cfg(target_os = "windows")]
    return windows::get_physical_size_mm(monitor);
}

/// ディスプレイカテゴリを自動判定する
pub fn detect_display_category(monitor: &MonitorHandle) -> (DisplayCategory, f32) {
    let phys = get_physical_size_mm(monitor);
    let px_size = monitor.size();
    let is_internal = is_internal_display(monitor);

    let ppi: f32 = if let Some((w_mm, h_mm)) = phys {
        let diag_mm = (w_mm * w_mm + h_mm * h_mm).sqrt();
        let diag_px = ((px_size.width as f32).powi(2) + (px_size.height as f32).powi(2)).sqrt();
        (diag_px / diag_mm) * 25.4
    } else {
        estimate_ppi_from_resolution(is_internal, px_size.width, px_size.height)
    };

    let diag_inch: f32 = if let Some((w_mm, h_mm)) = phys {
        let diag_mm = (w_mm * w_mm + h_mm * h_mm).sqrt();
        diag_mm / 25.4
    } else {
        estimate_diag_inch(is_internal, px_size.width, px_size.height)
    };

    let category = if is_internal {
        if ppi >= 180.0 {
            DisplayCategory::NotebookHiDpi
        } else {
            DisplayCategory::NotebookFhd
        }
    } else {
        let is_4k_or_higher = px_size.width >= 2560 // HiDPI 3008も4K相当として捕捉
            || (ppi > 130.0 && !is_internal);        // PPI が高く外付けなら 4K 相当
        let is_large = diag_inch >= 26.0; // 27インチに近いものを含む
        if is_large && is_4k_or_higher {
            DisplayCategory::ExternalLarge4K
        } else {
            DisplayCategory::ExternalGeneral
        }
    };

    (category, ppi)
}

fn estimate_ppi_from_resolution(is_internal: bool, w: u32, h: u32) -> f32 {
    let diag_px = ((w as f32).powi(2) + (h as f32).powi(2)).sqrt();
    let assumed_diag_inch = if is_internal {
        14.0
    } else if w >= 3840 {
        27.0
    } else {
        24.0
    };
    diag_px / assumed_diag_inch
}

fn estimate_diag_inch(is_internal: bool, w: u32, _h: u32) -> f32 {
    if is_internal { 14.0 } else if w >= 3840 { 27.0 } else { 24.0 }
}

pub fn apply_overlay_settings(window: &Window, _alpha: u8) {
    #[cfg(target_os = "macos")]
    macos::apply_overlay_settings(window, _alpha);
    #[cfg(target_os = "windows")]
    windows::apply_overlay_settings(window, _alpha);
}

#[derive(Debug)]
pub enum DisplayChangeEvent {
    Changed,
}

pub struct HotplugGuard {
    #[cfg(target_os = "macos")]
    _inner: macos::HotplugGuard,
    #[cfg(target_os = "windows")]
    _inner: windows::HotplugGuard,
}

pub fn register_hotplug_handler(tx: mpsc::Sender<DisplayChangeEvent>) -> HotplugGuard {
    #[cfg(target_os = "macos")]
    return HotplugGuard { _inner: macos::register_hotplug_observer(tx) };
    #[cfg(target_os = "windows")]
    return HotplugGuard { _inner: windows::register_display_change_hook(tx) };
}

pub fn detect_panel_type(monitor: &MonitorHandle) -> PanelType {
    #[cfg(target_os = "macos")]
    return macos::detect_panel_type(monitor);
    #[cfg(target_os = "windows")]
    return windows::detect_panel_type(monitor);
}

pub fn get_monitor_name(monitor: &MonitorHandle) -> String {
    #[cfg(target_os = "macos")]
    return macos::get_monitor_name(monitor);
    #[cfg(target_os = "windows")]
    return windows::get_monitor_name(monitor);
}

pub fn show_error_dialog(title: &str, message: &str) {
    #[cfg(target_os = "macos")]
    macos::show_error_dialog(title, message);
    #[cfg(target_os = "windows")]
    windows::show_error_dialog(title, message);
}

pub fn show_info_dialog(title: &str, message: &str) {
    #[cfg(target_os = "macos")]
    macos::show_info_dialog(title, message);
    #[cfg(target_os = "windows")]
    windows::show_info_dialog(title, message);
}

#[allow(dead_code)]
pub fn send_notification(title: &str, subtitle: &str, body: &str) {
    #[cfg(target_os = "macos")]
    macos::send_notification(title, subtitle, body);
    #[cfg(target_os = "windows")]
    windows::send_notification(title, subtitle, body);
}

#[cfg(target_os = "macos")]
pub fn has_screen_capture_access() -> bool {
    return macos::has_screen_capture_access();
}

pub fn is_dark_mode() -> bool {
    #[cfg(target_os = "macos")]
    return macos::is_dark_mode();
    #[cfg(target_os = "windows")]
    return windows::is_dark_mode();
}

#[allow(dead_code)]
pub fn set_dark_mode(enabled: bool) {
    #[cfg(target_os = "macos")]
    macos::set_dark_mode(enabled);
    #[cfg(target_os = "windows")]
    windows::set_dark_mode(enabled);
}

#[allow(dead_code)]
pub fn get_brightness() -> f32 {
    #[cfg(target_os = "macos")]
    return macos::get_brightness();
    #[cfg(target_os = "windows")]
    return windows::get_brightness();
}

#[allow(dead_code)]
pub fn set_brightness(level: f32) {
    #[cfg(target_os = "macos")]
    macos::set_brightness(level);
    #[cfg(target_os = "windows")]
    windows::set_brightness(level);
}

use image::DynamicImage;

#[allow(dead_code)]
pub fn capture_primary_display() -> Result<DynamicImage, String> {
    #[cfg(target_os = "macos")]
    return macos::capture_primary_display();
    #[cfg(target_os = "windows")]
    return windows::capture_primary_display();
}

pub fn capture_display(monitor_id: &MonitorId) -> Result<DynamicImage, String> {
    #[cfg(target_os = "macos")]
    return macos::capture_display(monitor_id);
    #[cfg(target_os = "windows")]
    return windows::capture_display(monitor_id);
}
