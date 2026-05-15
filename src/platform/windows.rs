use tao::monitor::MonitorHandle;
use tao::window::Window;
use tao::platform::windows::{MonitorHandleExtWindows, WindowExtWindows};
use crate::display_config::MonitorId;
use windows_sys::Win32::UI::WindowsAndMessaging::*;
use windows_sys::Win32::Foundation::*;
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;

use std::sync::mpsc;
use std::thread;
use std::ptr::null_mut;
use crate::platform::DisplayChangeEvent;

pub fn get_monitor_id(monitor: &MonitorHandle) -> MonitorId {
    MonitorId(monitor.hmonitor() as u64)
}

pub struct HotplugGuard {
    hwnd: HWND,
    thread_handle: Option<thread::JoinHandle<()>>,
}

impl Drop for HotplugGuard {
    fn drop(&mut self) {
        if !self.hwnd.is_null() {
            unsafe {
                PostMessageW(self.hwnd, WM_CLOSE, 0, 0);
            }
        }
        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }
    }
}

pub fn register_display_change_hook(tx: mpsc::Sender<DisplayChangeEvent>) -> HotplugGuard {
    let (hwnd_tx, hwnd_rx) = mpsc::channel();
    
    let thread_handle = thread::spawn(move || {
        unsafe {
            let hinstance = GetModuleHandleW(null_mut());
            let class_name = "SoftveilDisplayChangeHook\0".encode_utf16().collect::<Vec<u16>>();
            
            let wnd_class = WNDCLASSW {
                style: 0,
                lpfnWndProc: Some(window_proc),
                cbClsExtra: 0,
                cbWndExtra: 0,
                hInstance: hinstance,
                hIcon: null_mut(),
                hCursor: null_mut(),
                hbrBackground: null_mut(),
                lpszMenuName: null_mut(),
                lpszClassName: class_name.as_ptr(),
            };
            
            RegisterClassW(&wnd_class);
            
            // Create a hidden top-level window to receive broadcast messages (WM_DISPLAYCHANGE)
            let hwnd = CreateWindowExW(
                0,
                class_name.as_ptr(),
                null_mut(),
                0,
                0, 0, 0, 0,
                null_mut(),
                null_mut(),
                hinstance,
                null_mut(),
            );
            
            if !hwnd.is_null() {
                let tx_ptr = Box::into_raw(Box::new(tx));
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, tx_ptr as isize);
                let _ = hwnd_tx.send(hwnd);
                
                let mut msg = std::mem::zeroed();
                while GetMessageW(&mut msg, null_mut(), 0, 0) != 0 {
                    TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }
                
                let tx_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut mpsc::Sender<DisplayChangeEvent>;
                if !tx_ptr.is_null() {
                    let _ = Box::from_raw(tx_ptr);
                }
            } else {
                let _ = hwnd_tx.send(null_mut());
            }
        }
    });

    let hwnd = hwnd_rx.recv().unwrap_or(null_mut());
    HotplugGuard {
        hwnd,
        thread_handle: Some(thread_handle),
    }
}

extern "system" fn window_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_DISPLAYCHANGE => {
            unsafe {
                let tx_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut mpsc::Sender<DisplayChangeEvent>;
                if !tx_ptr.is_null() {
                    let tx = &*tx_ptr;
                    let _ = tx.send(DisplayChangeEvent::Changed);
                }
            }
            0
        }
        WM_DESTROY => {
            unsafe {
                PostQuitMessage(0);
            }
            0
        }
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}

pub fn apply_overlay_settings(window: &Window, alpha: u8) {
    let hwnd = window.hwnd() as HWND;
    unsafe {
        set_ex_style(hwnd, WS_EX_TRANSPARENT | WS_EX_LAYERED | WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW);
        SetLayeredWindowAttributes(hwnd, 0, alpha, LWA_ALPHA);
        SetWindowPos(hwnd, HWND_TOPMOST as HWND, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_SHOWWINDOW);
    }
}

unsafe fn set_ex_style(hwnd: HWND, additional_flags: u32) {
    let current_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as usize;
    SetWindowLongPtrW(hwnd, GWL_EXSTYLE, (current_style | additional_flags as usize) as isize);
}

