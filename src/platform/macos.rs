use tao::monitor::MonitorHandle;
use tao::window::Window;
use tao::platform::macos::MonitorHandleExtMacOS;
use tao::platform::macos::WindowExtMacOS;
use crate::display_config::{MonitorId, PanelType};
use crate::platform::DisplayChangeEvent;
use objc2_app_kit::{NSWindow, NSWindowCollectionBehavior, NSColor, NSScreen};
use objc2::{msg_send, rc::Retained, MainThreadMarker};
use std::sync::{Mutex, mpsc};
use std::collections::HashMap;
use std::process::Command;
use serde_json::Value;
use objc2_foundation::{NSNotificationCenter, NSString, NSObject, NSNotification};
use block2::StackBlock;
use std::thread;
use std::time::Duration;

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

pub struct HotplugGuard {
    token: Retained<NSObject>,
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
    
    let block = StackBlock::new(move |_notif: &NSNotification| {
        // Clear cache on hotplug
        if let Ok(mut cache) = MONITOR_NAME_CACHE.lock() {
            *cache = None;
        }
        
        // Prefetch monitor names in background (Fix #7)
        // Note: We don't update the cache directly here to avoid threading issues with UI.
        // The event loop will handle the refresh when it receives DisplayChangeEvent::Changed.
        let tx_clone = tx.clone();
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(800)); // Wait for OS to settle
            let _ = tx_clone.send(DisplayChangeEvent::Changed);
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
    
    HotplugGuard { token }
}

impl Drop for HotplugGuard {
    fn drop(&mut self) {
        let center = NSNotificationCenter::defaultCenter();
        unsafe {
            let _: () = msg_send![&*center, removeObserver: &*self.token];
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
                let screen_id_obj: Retained<NSObject> = msg_send![&*description, objectForKey: &*NSString::from_str("NSScreenNumber")];
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

pub fn has_screen_capture_access() -> bool {
    #[link(name = "CoreGraphics", kind = "framework")]
    extern "C" {
        fn CGPreflightScreenCaptureAccess() -> bool;
    }
    unsafe { CGPreflightScreenCaptureAccess() }
}

pub fn show_permission_alert(title: &str, message: &str) {
    let safe_title = title.replace('\\', "\\\\").replace('"', "\\\"");
    let safe_message = message.replace('\\', "\\\\").replace('"', "\\\"");
    let script = format!(
        "display alert \"{}\" message \"{}\" buttons {{\"OK\"}} default button \"OK\"",
        safe_title, safe_message
    );
    let _ = Command::new("osascript").args(["-e", &script]).status();
}
