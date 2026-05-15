use tao::monitor::MonitorHandle;
use tao::window::Window;
use tao::platform::windows::{MonitorHandleExtWindows, WindowExtWindows};
use crate::display_config::MonitorId;
use windows_sys::Win32::UI::WindowsAndMessaging::*;

use std::sync::mpsc;
use crate::platform::DisplayChangeEvent;

pub fn get_monitor_id(monitor: &MonitorHandle) -> MonitorId {
    MonitorId(monitor.hmonitor() as u64)
}

pub struct HotplugGuard {}

pub fn register_display_change_hook(tx: mpsc::Sender<DisplayChangeEvent>) -> HotplugGuard {
    // Stub for now
    let _ = tx;
    HotplugGuard {}
}

pub fn apply_overlay_settings(window: &Window, alpha: u8) {
    let hwnd = window.hwnd() as isize;
    unsafe {
        set_ex_style(hwnd, WS_EX_TRANSPARENT | WS_EX_LAYERED | WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW);
        SetLayeredWindowAttributes(hwnd, 0, alpha, LWA_ALPHA);
        SetWindowPos(hwnd, HWND_TOPMOST, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_SHOWWINDOW);
    }
}

unsafe fn set_ex_style(hwnd: isize, additional_flags: u32) {
    let current_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as usize;
    SetWindowLongPtrW(hwnd, GWL_EXSTYLE, (current_style | additional_flags as usize) as isize);
}
