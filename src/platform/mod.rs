#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(target_os = "windows")]
pub mod windows;

use tao::monitor::MonitorHandle;
use tao::window::Window;
use crate::display_config::{MonitorId, PanelType};
use std::sync::mpsc;

pub fn get_monitor_id(monitor: &MonitorHandle) -> MonitorId {
    #[cfg(target_os = "macos")]
    return macos::get_monitor_id(monitor);
    #[cfg(target_os = "windows")]
    return windows::get_monitor_id(monitor);
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
