use tao::monitor::MonitorHandle;
use tao::window::Window;
use crate::logger;
use tao::platform::macos::MonitorHandleExtMacOS;
use tao::platform::macos::WindowExtMacOS;
use crate::display_config::{MonitorId, PanelType};
use crate::platform::DisplayChangeEvent;
use objc2_app_kit::{NSWindow, NSWindowCollectionBehavior, NSColor, NSScreen, NSAlert};
use objc2_foundation::{NSNotificationCenter, NSDistributedNotificationCenter, NSString, NSObject, NSNotification, NSUserDefaults, NSAppleScript};
use objc2::{msg_send, rc::Retained, MainThreadMarker, AnyThread};
use block2::StackBlock;
use std::sync::{Mutex, mpsc};
use std::collections::HashMap;
use std::process::Command;
use serde_json::Value;
use std::thread;
use std::time::Duration;
use image::DynamicImage;

#[link(name = "IOKit", kind = "framework")]
extern "C" {
    fn IOServiceGetMatchingServices(
        masterPort: u32,
        matching: *mut std::ffi::c_void,
        existing: *mut u32,
    ) -> i32;
    fn IOServiceMatching(name: *const i8) -> *mut std::ffi::c_void;
    fn IOIteratorNext(iterator: u32) -> u32;
    fn IODisplayGetFloatParameter(
        service: u32,
        options: u32,
        parameterName: *const NSString,
        value: *mut f32,
    ) -> i32;
    fn IODisplaySetFloatParameter(
        service: u32,
        options: u32,
        parameterName: *const NSString,
        value: f32,
    ) -> i32;
    fn IOObjectRelease(obj: u32) -> i32;
}

static MONITOR_NAME_CACHE: Mutex<Option<HashMap<u64, String>>> = Mutex::new(None);

pub fn get_monitor_id(monitor: &MonitorHandle) -> MonitorId {
    MonitorId(monitor.native_id() as u64)
}

/// 内蔵ディスプレイかどうかを判定する
pub fn is_internal_display(monitor: &MonitorHandle) -> bool {
    // 方法A: CGMainDisplayID との比較 (クラムシェル時は外付けがメインになるので注意が必要)
    // より確実には IOKit や system_profiler を使うべきだが、ここでは基本ロジックを実装
    #[link(name = "CoreGraphics", kind = "framework")]
    extern "C" {
        fn CGDisplayIsBuiltin(display: u32) -> i32;
    }
    unsafe {
        let id = get_monitor_id(monitor).0 as u32;
        CGDisplayIsBuiltin(id) != 0
    }
}

/// (mm, mm) = (width_mm, height_mm) を返す。取得できない場合は None。
pub fn get_physical_size_mm(monitor: &MonitorHandle) -> Option<(f32, f32)> {
    #[link(name = "CoreGraphics", kind = "framework")]
    extern "C" {
        fn CGDisplayScreenSize(display: u32) -> CGSize;
    }
    #[repr(C)]
    struct CGSize { width: f64, height: f64 }

    unsafe {
        let id = get_monitor_id(monitor).0 as u32;
        let size = CGDisplayScreenSize(id);
        if size.width > 0.0 && size.height > 0.0 {
            Some((size.width as f32, size.height as f32))
        } else {
            None
        }
    }
}

pub fn apply_overlay_settings(window: &Window, alpha: u8) {
    let ns_window = window.ns_window() as *mut NSWindow;
    unsafe {
        let ns_window = &*ns_window;
        
        ns_window.setIgnoresMouseEvents(true);
        ns_window.setBackgroundColor(Some(&NSColor::clearColor()));
        
        // Set a very high level to stay above system transitions and spaces.
        ns_window.setLevel(10000); 
        
        ns_window.setCollectionBehavior(
            NSWindowCollectionBehavior::CanJoinAllSpaces |
            NSWindowCollectionBehavior::Stationary |
            NSWindowCollectionBehavior::FullScreenAuxiliary |
            NSWindowCollectionBehavior::IgnoresCycle
        );

        ns_window.setHidesOnDeactivate(false);
        ns_window.setAnimationBehavior(objc2_app_kit::NSWindowAnimationBehavior::None);
        ns_window.setOpaque(false);
        ns_window.setHasShadow(false);
        
        // Convert u8 alpha (0-255) to f64 (0.0-1.0)
        ns_window.setAlphaValue(alpha as f64 / 255.0);
    }
}

pub fn register_theme_change_observer(tx: mpsc::Sender<DisplayChangeEvent>) -> Retained<NSObject> {
    let center: Retained<NSDistributedNotificationCenter> = unsafe {
        msg_send![objc2::class!(NSDistributedNotificationCenter), defaultCenter]
    };
    let notification_name = NSString::from_str("AppleInterfaceThemeChangedNotification");
    
    let block = StackBlock::new(move |_notif: &NSNotification| {
        let _ = tx.send(DisplayChangeEvent::Changed);
    });
    let block = block.copy();
    
    let token: Retained<NSObject> = unsafe {
        msg_send![
            &*center,
            addObserverForName: &*notification_name,
            object: None::<&NSObject>,
            queue: None::<&NSObject>,
            usingBlock: &*block
        ]
    };
    
    token
}

pub struct HotplugGuard {
    token: Retained<NSObject>,
    theme_token: Retained<NSObject>,
}

pub fn get_monitor_name(monitor: &MonitorHandle) -> String {
    let id = get_monitor_id(monitor).0;
    
    let mut cache = MONITOR_NAME_CACHE.lock().unwrap();
    if cache.is_none() {
        *cache = Some(fetch_monitor_names());
    }
    
    if let Some(ref map) = *cache {
        if let Some(name) = map.get(&id) {
            return name.clone();
        }
    }
    
    monitor.name().unwrap_or_else(|| format!("Display {}", id))
}

fn fetch_monitor_names() -> HashMap<u64, String> {
    let mut map = HashMap::new();
    let output = Command::new("system_profiler")
        .args(["SPDisplaysDataType", "-json"])
        .output();

    if let Ok(output) = output {
        if let Ok(val) = serde_json::from_slice::<Value>(&output.stdout) {
            if let Some(displays) = val["SPDisplaysDataType"].as_array() {
                for card in displays {
                    if let Some(ndrvs) = card["spdisplays_ndrvs"].as_array() {
                        for display in ndrvs {
                            let name = display["_name"].as_str().unwrap_or("Unknown");
                            let id_str = display["_spdisplays_displayID"].as_str().unwrap_or("");
                            
                            let id_opt: Option<u64> = if id_str.starts_with("0x") || id_str.starts_with("0X") {
                                u64::from_str_radix(&id_str[2..], 16).ok()
                            } else {
                                id_str.parse::<u64>().ok()
                            };

                            if let Some(id) = id_opt {
                                map.insert(id, name.to_string());
                            }
                        }
                    }
                }
            }
        }
    }
    map
}

pub fn register_hotplug_observer(tx: mpsc::Sender<DisplayChangeEvent>) -> HotplugGuard {
    let center = NSNotificationCenter::defaultCenter();
    let notification_name = NSString::from_str("NSApplicationDidChangeScreenParametersNotification");
    
    let tx_clone = tx.clone();
    let block = StackBlock::new(move |_notif: &NSNotification| {
        // Clear cache on hotplug
        if let Ok(mut cache) = MONITOR_NAME_CACHE.lock() {
            *cache = None;
        }
        
        let tx_clone2 = tx_clone.clone();
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(800)); // Wait for OS to settle
            let _ = tx_clone2.send(DisplayChangeEvent::Changed);
        });
    });
    let block = block.copy();
    
    let token: Retained<NSObject> = unsafe {
        msg_send![
            &*center,
            addObserverForName: &*notification_name,
            object: None::<&NSObject>,
            queue: None::<&NSObject>,
            usingBlock: &*block
        ]
    };
    
    let theme_token = register_theme_change_observer(tx.clone());
    
    HotplugGuard { token, theme_token }
}

impl Drop for HotplugGuard {
    fn drop(&mut self) {
        let center = NSNotificationCenter::defaultCenter();
        let dist_center: Retained<NSDistributedNotificationCenter> = unsafe {
            msg_send![objc2::class!(NSDistributedNotificationCenter), defaultCenter]
        };
        unsafe {
            let _: () = msg_send![&*center, removeObserver: &*self.token];
            let _: () = msg_send![&*dist_center, removeObserver: &*self.theme_token];
        }
    }
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
    if name.contains("XDR") || name.contains("RETINA") {
        score_ips += 5;
    }
    if name.contains(" TN ") || name.ends_with(" TN") {
        score_tn += 10;
    }

    // EDR Check (macOS specific)
    if let Some(mtm) = MainThreadMarker::new() {
        unsafe {
            let screens = NSScreen::screens(mtm);
            let target_id = get_monitor_id(monitor).0 as u32;
            
            for i in 0..screens.count() {
                let screen = screens.objectAtIndex(i);
                let description = screen.deviceDescription();
                let screen_id_obj: Retained<NSObject> = msg_send![
                    &*description,
                    objectForKey: &*NSString::from_str("NSScreenNumber")
                ];
                let screen_id: u32 = msg_send![&*screen_id_obj, unsignedIntValue];
                
                if screen_id == target_id {
                    let edr: f64 = screen.maximumExtendedDynamicRangeColorComponentValue();
                    if edr > 2.0 {
                        score_oled += 8; // High EDR likely means OLED or high-end Mini-LED (treat as OLED for masking)
                    } else if edr > 1.0 {
                        score_ips += 3;
                    }
                    break;
                }
            }
        }
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

#[allow(dead_code)]
pub fn has_screen_capture_access() -> bool {
    preflight_screen_capture_access()
}

/// CGPreflightScreenCaptureAccess() のみを呼ぶ。
/// キャプチャの実試行（CGDisplayCreateImageForRect）は行わない。
/// → macOS 14+ で TCC ダイアログがトリガーされるのを防ぐ
pub fn preflight_screen_capture_access() -> bool {
    // macOS 10.15+ (Catalina and later)
    #[link(name = "CoreGraphics", kind = "framework")]
    extern "C" {
        fn CGPreflightScreenCaptureAccess() -> bool;
    }
    unsafe {
        CGPreflightScreenCaptureAccess()
    }
}

#[allow(dead_code)]
pub fn request_screen_capture_access() -> bool {
    // If we already have access (either via preflight or successful capture), return true.
    if has_screen_capture_access() {
        return true;
    }

    #[link(name = "CoreGraphics", kind = "framework")]
    extern "C" {
        fn CGRequestScreenCaptureAccess() -> bool;
    }
    unsafe {
        // This will trigger the system dialog if not already granted.
        CGRequestScreenCaptureAccess()
    }
}

pub fn is_dark_mode() -> bool {
    let defaults = NSUserDefaults::standardUserDefaults();
    let key = NSString::from_str("AppleInterfaceStyle");
    let style = defaults.stringForKey(&key);
    style.map(|s| s.to_string() == "Dark").unwrap_or(false)
}

pub fn set_dark_mode(enabled: bool) {
    #[cfg(test)]
    {
        let _ = enabled;
        return;
    }

    #[cfg(not(test))]
    {
        let state = if enabled { "true" } else { "false" };
        let cmd = format!("tell application \"System Events\" to tell appearance preferences to set dark mode to {}", state);
        
        // Try NSAppleScript first
        unsafe {
            if let Some(script) = NSAppleScript::initWithSource(NSAppleScript::alloc(), &NSString::from_str(&cmd)) {
                let mut error: *mut NSObject = std::ptr::null_mut();
                let _: *mut NSObject = msg_send![&*script, executeAndReturnError: &mut error];
                if error.is_null() {
                    return;
                } else {
                    logger!("NSAppleScript Error in set_dark_mode: {:?}", error);
                }
            }
        }

        // Fallback to osascript CLI if NSAppleScript fails
        let _ = std::process::Command::new("osascript")
            .arg("-e")
            .arg(&cmd)
            .output();
    }
}

pub fn get_brightness() -> f32 {
    let mut brightness = 0.5f32;
    unsafe {
        let matching = IOServiceMatching("IODisplayConnect\0".as_ptr() as *const i8);
        let mut iterator = 0u32;
        let kr = IOServiceGetMatchingServices(0, matching, &mut iterator);
        if kr == 0 {
            let mut service = IOIteratorNext(iterator);
            while service != 0 {
                let key = NSString::from_str("brightness");
                let mut b = 0.0f32;
                if IODisplayGetFloatParameter(service, 0, &*key, &mut b) == 0 {
                    brightness = b;
                    IOObjectRelease(service);
                    break;
                }
                IOObjectRelease(service);
                service = IOIteratorNext(iterator);
            }
            IOObjectRelease(iterator);
        }
    }
    brightness
}

pub fn set_brightness(level: f32) {
    unsafe {
        let matching = IOServiceMatching("IODisplayConnect\0".as_ptr() as *const i8);
        let mut iterator = 0u32;
        let kr = IOServiceGetMatchingServices(0, matching, &mut iterator);
        if kr == 0 {
            let mut service = IOIteratorNext(iterator);
            while service != 0 {
                let key = NSString::from_str("brightness");
                IODisplaySetFloatParameter(service, 0, &*key, level);
                IOObjectRelease(service);
                service = IOIteratorNext(iterator);
            }
            IOObjectRelease(iterator);
        }
    }
}

pub fn show_error_dialog(title: &str, message: &str) {
    if let Some(mtm) = MainThreadMarker::new() {
        let alert = NSAlert::new(mtm);
        alert.setMessageText(&NSString::from_str(title));
        alert.setInformativeText(&NSString::from_str(message));
        alert.setAlertStyle(objc2_app_kit::NSAlertStyle::Critical);
        alert.runModal();
    }
}

pub fn show_info_dialog(title: &str, message: &str) {
    if let Some(mtm) = MainThreadMarker::new() {
        let alert = NSAlert::new(mtm);
        alert.setMessageText(&NSString::from_str(title));
        alert.setInformativeText(&NSString::from_str(message));
        alert.setAlertStyle(objc2_app_kit::NSAlertStyle::Informational);
        alert.runModal();
    }
}

#[allow(dead_code)]
pub fn capture_primary_display() -> Result<DynamicImage, String> {
    capture_display(&MonitorId(0))
}

pub fn write_to_log_file(msg: &str) {
    use std::fs::OpenOptions;
    use std::io::Write;
    let path = "/tmp/softveil.log";
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let _ = writeln!(file, "[{}] {}", now, msg);
    }
}

pub fn send_notification(title: &str, subtitle: &str, body: &str) {
    #[cfg(test)]
    {
        let _ = (title, subtitle, body);
        return;
    }

    #[cfg(not(test))]
    {
        let source = format!(
            "display notification \"{}\" with title \"{}\" subtitle \"{}\"",
            body.replace("\"", "\\\""),
            title.replace("\"", "\\\""),
            subtitle.replace("\"", "\\\"")
        );
        unsafe {
            if let Some(script) = NSAppleScript::initWithSource(NSAppleScript::alloc(), &NSString::from_str(&source)) {
                let _: *mut NSObject = msg_send![&*script, executeAndReturnError: std::ptr::null_mut::<*mut NSObject>()];
            }
        }
    }
}

use core_graphics::display::CGDisplay;
use image::{RgbaImage, Rgba};

pub fn capture_display(monitor_id: &MonitorId) -> Result<DynamicImage, String> {
    let display_id = if monitor_id.0 == 0 {
        unsafe {
            #[link(name = "CoreGraphics", kind = "framework")]
            extern "C" {
                fn CGMainDisplayID() -> u32;
            }
            CGMainDisplayID()
        }
    } else {
        monitor_id.0 as u32
    };

    let image = match CGDisplay::new(display_id).image() {
        Some(img) => img,
        None => {
            logger!("CGDisplay::image() returned None for display_id: {}", display_id);
            // We only return error here. Higher-level logic (e.g. UserEvent::RunBenchmark)
            // can perform a more robust permission check if needed.
            return Err(format!("Failed to capture display {}", display_id));
        }
    };
    
    // If we succeeded, we clearly have access (or the OS is letting us anyway)
    // so we don't need to show the alert later if a single frame fails.
    // However, it's safer to just let the logic above handle it.

    let width = image.width();
    let height = image.height();
    let data = image.data();
    let raw_data = data.bytes();
    let bytes_per_row = image.bytes_per_row();

    // CGImage from CGDisplayCreateImage is typically 32bpp BGRA on macOS.
    let mut rgba = RgbaImage::new(width as u32, height as u32);
    
    // Safety check for data length
    if raw_data.len() < height * bytes_per_row {
        return Err("Incomplete display data".to_string());
    }

    for (y, row) in raw_data.chunks_exact(bytes_per_row).take(height).enumerate() {
        if row.len() >= width * 4 {
            for (x, pixel) in row.chunks_exact(4).take(width).enumerate() {
                // BGRA -> RGBA
                rgba.put_pixel(x as u32, y as u32, Rgba([pixel[2], pixel[1], pixel[0], pixel[3]]));
            }
        }
    }

    Ok(DynamicImage::ImageRgba8(rgba))
}
