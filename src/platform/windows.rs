use tao::monitor::MonitorHandle;
use tao::window::Window;
use tao::platform::windows::{MonitorHandleExtWindows, WindowExtWindows};
use crate::display_config::{MonitorId, PanelType};
use windows_sys::Win32::UI::WindowsAndMessaging::*;
use windows_sys::Win32::Foundation::*;
use windows_sys::Win32::Graphics::Gdi::*;
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::System::Registry::*;

use windows::{
    core::*,
    Win32::System::Wmi::*,
    Win32::System::Com::*,
};

use std::sync::mpsc;
use std::thread;
use std::ptr::null_mut;
use crate::platform::DisplayChangeEvent;

pub fn get_monitor_id(monitor: &MonitorHandle) -> MonitorId {
    MonitorId(monitor.hmonitor() as u64)
}

fn init_com() {
    unsafe {
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
    }
}

/// 内蔵ディスプレイ判定
pub fn is_internal_display(monitor: &MonitorHandle) -> bool {
    unsafe {
        let hmonitor = monitor.hmonitor() as HMONITOR;
        let mut info: MONITORINFOEXW = std::mem::zeroed();
        info.monitorInfo.cbSize = std::mem::size_of::<MONITORINFOEXW>() as u32;
        if GetMonitorInfoW(hmonitor, &mut info.monitorInfo as *mut _ as *mut _) == 0 {
            return false;
        }

        let mut device: DISPLAY_DEVICEW = std::mem::zeroed();
        device.cb = std::mem::size_of::<DISPLAY_DEVICEW>() as u32;
        
        if EnumDisplayDevicesW(info.szDevice.as_ptr(), 0, &mut device, 0) != 0 {
            let name = String::from_utf16_lossy(&device.DeviceString).to_uppercase();
            if name.contains("EDP") || name.contains("INTERNAL") || name.contains("INTEGRATED") || name.contains("LAPTOP") {
                return true;
            }
        }
        
        // Fallback: Check if it's the primary device and has a laptop-like width
        let is_primary = (device.StateFlags & DISPLAY_DEVICE_PRIMARY_DEVICE) != 0;
        let phys_w = monitor.size().width;
        is_primary && phys_w <= 2560
    }
}

/// GetDeviceCaps(HORZSIZE / VERTSIZE) でmm単位の物理サイズを取得する
pub fn get_physical_size_mm(monitor: &MonitorHandle) -> Option<(f32, f32)> {
    unsafe {
        let hmonitor = monitor.hmonitor() as HMONITOR;
        let mut info: MONITORINFOEXW = std::mem::zeroed();
        info.monitorInfo.cbSize = std::mem::size_of::<MONITORINFOEXW>() as u32;
        if GetMonitorInfoW(hmonitor, &mut info.monitorInfo as *mut _ as *mut _) == 0 {
            return None;
        }

        let hdc = CreateDCW(
            info.szDevice.as_ptr(),
            info.szDevice.as_ptr(),
            std::ptr::null(),
            std::ptr::null(),
        );
        if hdc.is_null() { return None; }
        
        let w_mm = GetDeviceCaps(hdc, HORZSIZE as i32) as f32;
        let h_mm = GetDeviceCaps(hdc, VERTSIZE as i32) as f32;
        DeleteDC(hdc);
        
        if w_mm > 0.0 && h_mm > 0.0 {
            Some((w_mm, h_mm))
        } else {
            None
        }
    }
}

pub struct HotplugGuard {
    hwnd: SendHWND,
    thread_handle: Option<thread::JoinHandle<()>>,
}

#[derive(Clone, Copy)]
struct SendHWND(HWND);
unsafe impl Send for SendHWND {}

impl Drop for HotplugGuard {
    fn drop(&mut self) {
        if !self.hwnd.0.is_null() {
            unsafe {
                PostMessageW(self.hwnd.0, WM_CLOSE, 0, 0);
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
                let _ = hwnd_tx.send(SendHWND(hwnd));
                
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
                let _ = hwnd_tx.send(SendHWND(null_mut()));
            }
        }
    });

    let hwnd = hwnd_rx.recv().unwrap_or(SendHWND(null_mut()));
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
        WM_SETTINGCHANGE => {
            unsafe {
                if lparam != 0 {
                    let s = String::from_utf16_lossy(std::slice::from_raw_parts(lparam as *const u16, 20));
                    if s.contains("ImmersiveColorSet") {
                        let tx_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut mpsc::Sender<DisplayChangeEvent>;
                        if !tx_ptr.is_null() {
                            let tx = &*tx_ptr;
                            let _ = tx.send(DisplayChangeEvent::Changed);
                        }
                    }
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

/// Windows での DPI Awareness を有効にする
pub fn enable_dpi_awareness() {
    unsafe {
        use windows_sys::Win32::UI::HiDpi::*;
        // SetProcessDpiAwarenessContext (Windows 10 1703+)
        SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
    }
}

pub fn is_dark_mode() -> bool {
    unsafe {
        let subkey = "Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize\0"
            .encode_utf16()
            .collect::<Vec<u16>>();
        let value_name = "AppsUseLightTheme\0".encode_utf16().collect::<Vec<u16>>();
        
        let mut hkey: HKEY = null_mut();
        if RegOpenKeyExW(HKEY_CURRENT_USER, subkey.as_ptr(), 0, KEY_READ, &mut hkey) == 0 {
            let mut data: u32 = 0;
            let mut size = std::mem::size_of::<u32>() as u32;
            let res = RegQueryValueExW(hkey, value_name.as_ptr(), null_mut(), null_mut(), &mut data as *mut _ as *mut _, &mut size);
            RegCloseKey(hkey);
            if res == 0 {
                return data == 0; // 0 means Dark Mode
            }
        }
    }
    false
}

pub fn set_dark_mode(enabled: bool) {
    unsafe {
        let subkey = "Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize\0"
            .encode_utf16()
            .collect::<Vec<u16>>();
        let value_name = "AppsUseLightTheme\0".encode_utf16().collect::<Vec<u16>>();
        let value_name_system = "SystemUsesLightTheme\0".encode_utf16().collect::<Vec<u16>>();
        
        let mut hkey: HKEY = null_mut();
        if RegOpenKeyExW(HKEY_CURRENT_USER, subkey.as_ptr(), 0, KEY_SET_VALUE, &mut hkey) == 0 {
            let data: u32 = if enabled { 0 } else { 1 };
            RegSetValueExW(hkey, value_name.as_ptr(), 0, REG_DWORD, &data as *const _ as *const _, 4);
            RegSetValueExW(hkey, value_name_system.as_ptr(), 0, REG_DWORD, &data as *const _ as *const _, 4);
            RegCloseKey(hkey);
            
            // Broadcast setting change so other apps (and the taskbar) update immediately
            let lparam = "ImmersiveColorSet\0".encode_utf16().collect::<Vec<u16>>();
            SendMessageTimeoutW(
                HWND_BROADCAST as HWND,
                WM_SETTINGCHANGE,
                0,
                lparam.as_ptr() as LPARAM,
                SMTO_ABORTIFHUNG,
                2000,
                std::ptr::null_mut(),
            );
        }
    }
}

pub fn get_brightness() -> f32 {
    init_com();
    match get_brightness_wmi() {
        Ok(b) => b,
        Err(_) => 0.5,
    }
}

fn get_brightness_wmi() -> windows::core::Result<f32> {
    unsafe {
        let locator: IWbemLocator = CoCreateInstance(&WbemLocator, None, CLSCTX_INPROC_SERVER)?;
        let services = locator.ConnectServer(&BSTR::from("root\\WMI"), None, None, None, 0, None, None)?;
        
        let query = BSTR::from("SELECT CurrentBrightness FROM WmiMonitorBrightness");
        let enumerator = services.ExecQuery(&BSTR::from("WQL"), &query, WBEM_FLAG_FORWARD_ONLY | WBEM_FLAG_RETURN_IMMEDIATELY, None)?;
        
        let mut brightness = 0.5f32;
        let mut objects = [None; 1];
        let mut returned = 0;
        enumerator.Next(WBEM_INFINITE, &mut objects, &mut returned).ok()?;
        
        if returned > 0 {
            if let Some(obj) = &objects[0] {
                let mut variant = windows::core::VARIANT::default();
                obj.Get(windows::core::w!("CurrentBrightness"), 0, &mut variant, None, None)?;
                // Use windows_sys raw variant for layout access
                let raw: &windows_sys::Win32::System::Variant::VARIANT = std::mem::transmute(&variant);
                brightness = raw.Anonymous.Anonymous.Anonymous.uiVal as f32 / 100.0;
            }
        }
        Ok(brightness)
    }
}

pub fn set_brightness(level: f32) {
    init_com();
    let _ = set_brightness_wmi(level);
}

fn set_brightness_wmi(level: f32) -> windows::core::Result<()> {
    unsafe {
        let locator: IWbemLocator = CoCreateInstance(&WbemLocator, None, CLSCTX_INPROC_SERVER)?;
        let services = locator.ConnectServer(&BSTR::from("root\\WMI"), None, None, None, 0, None, None)?;
        
        let b = (level * 100.0).clamp(0.0, 100.0) as u8;
        
        let query = BSTR::from("SELECT * FROM WmiMonitorBrightnessMethods");
        let enumerator = services.ExecQuery(&BSTR::from("WQL"), &query, WBEM_FLAG_FORWARD_ONLY | WBEM_FLAG_RETURN_IMMEDIATELY, None)?;
        
        let mut objects = [None; 1];
        let mut returned = 0;
        while enumerator.Next(WBEM_INFINITE, &mut objects, &mut returned).is_ok() && returned > 0 {
            if let Some(obj) = &objects[0] {
                let mut path_variant = windows::core::VARIANT::default();
                obj.Get(windows::core::w!("__PATH"), 0, &mut path_variant, None, None)?;
                let raw_path: &windows_sys::Win32::System::Variant::VARIANT = std::mem::transmute(&path_variant);
                let path_ptr = raw_path.Anonymous.Anonymous.Anonymous.bstrVal;
                
                let class_name = BSTR::from("WmiMonitorBrightnessMethods");
                let method_name = BSTR::from("WmiSetBrightness");
                let mut class = None;
                services.GetObject(&class_name, WBEM_GENERIC_FLAG_TYPE(0), None, Some(&mut class), None)?;
                
                let in_params_def = class.as_ref().unwrap();
                let mut in_params_obj: Option<IWbemClassObject> = None;
                in_params_def.GetMethod(&method_name, 0, &mut in_params_obj, std::ptr::null_mut())?;
                
                let in_params = in_params_obj.as_ref().unwrap().SpawnInstance(0)?;
                
                let mut var_timeout = windows::core::VARIANT::default();
                let raw_timeout: &mut windows_sys::Win32::System::Variant::VARIANT = std::mem::transmute(&mut var_timeout);
                raw_timeout.Anonymous.Anonymous.vt = windows_sys::Win32::System::Variant::VT_UI4;
                raw_timeout.Anonymous.Anonymous.Anonymous.ulVal = 0;
                in_params.Put(windows::core::w!("Timeout"), 0, &var_timeout, 0)?;
                
                let mut var_brightness = windows::core::VARIANT::default();
                let raw_brightness: &mut windows_sys::Win32::System::Variant::VARIANT = std::mem::transmute(&mut var_brightness);
                raw_brightness.Anonymous.Anonymous.vt = windows_sys::Win32::System::Variant::VT_UI1;
                raw_brightness.Anonymous.Anonymous.Anonymous.bVal = b;
                in_params.Put(windows::core::w!("Brightness"), 0, &var_brightness, 0)?;
                
                let path = BSTR::from_raw(path_ptr);
                let res = services.ExecMethod(&path, &method_name, WBEM_GENERIC_FLAG_TYPE(0), None, &in_params, None, None);
                std::mem::forget(path); // VARIANT owns the underlying BSTR
                res?;
            }
        }
        Ok(())
    }
}

pub fn show_error_dialog(title: &str, message: &str) {
    let wide_title: Vec<u16> = title.encode_utf16().chain(std::iter::once(0)).collect();
    let wide_message: Vec<u16> = message.encode_utf16().chain(std::iter::once(0)).collect();
    unsafe {
        MessageBoxW(std::ptr::null_mut(), wide_message.as_ptr(), wide_title.as_ptr(), MB_ICONERROR | MB_OK);
    }
}

unsafe fn set_ex_style(hwnd: HWND, additional_flags: u32) {
    let current_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as usize;
    SetWindowLongPtrW(hwnd, GWL_EXSTYLE, (current_style | additional_flags as usize) as isize);
}

pub fn get_monitor_name(monitor: &MonitorHandle) -> String {
    unsafe {
        let hmonitor = monitor.hmonitor() as HMONITOR;
        let mut info: MONITORINFOEXW = std::mem::zeroed();
        info.monitorInfo.cbSize = std::mem::size_of::<MONITORINFOEXW>() as u32;
        
        if GetMonitorInfoW(hmonitor, &mut info.monitorInfo as *mut _ as *mut _) != 0 {
            let mut device: DISPLAY_DEVICEW = std::mem::zeroed();
            device.cb = std::mem::size_of::<DISPLAY_DEVICEW>() as u32;
            
            // First call with adapter name to get the monitor device path
            if EnumDisplayDevicesW(info.szDevice.as_ptr(), 0, &mut device, 0) != 0 {
                let name = String::from_utf16_lossy(&device.DeviceString)
                    .trim_matches(char::from(0))
                    .to_string();
                if !name.is_empty() {
                    return name;
                }
            }
        }
    }
    monitor.name().unwrap_or_else(|| format!("Monitor #{}", get_monitor_id(monitor).0))
}

pub fn detect_panel_type(monitor: &MonitorHandle) -> PanelType {
    let name = get_monitor_name(monitor).to_uppercase();
    let mut score_oled = 0;
    let mut score_ips = 0;
    let mut score_tn = 0;

    // Keyword matching
    if name.contains("OLED") || name.contains("AMOLED") {
        score_oled += 10;
    }
    if name.contains("IPS") || name.contains("ULTRAFINE") {
        score_ips += 5;
    }
    if name.contains(" TN ") || name.ends_with(" TN") || name.contains("ZOWIE") {
        score_tn += 8;
    }

    // Refresh rate check
    let hz = monitor.video_modes().next().map(|m| m.refresh_rate()).unwrap_or(60);
    if hz >= 120 {
        score_oled += 2;
        score_ips += 2;
    }

    if score_oled >= score_ips && score_oled >= score_tn && score_oled > 0 {
        PanelType::Oled
    } else if score_tn >= score_ips && score_tn > 0 {
        PanelType::LcdTn
    } else if score_ips > 0 {
        PanelType::LcdIps
    } else {
        PanelType::Unknown
    }
}

