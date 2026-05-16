use tao::monitor::MonitorHandle;
use tao::window::Window;
use tao::platform::macos::MonitorHandleExtMacOS;
use tao::platform::macos::WindowExtMacOS;
use crate::display_config::MonitorId;
use crate::platform::DisplayChangeEvent;
use objc2_app_kit::{NSWindow, NSWindowCollectionBehavior, NSColor};
use std::sync::{Mutex, mpsc};
use std::collections::HashMap;
use std::process::Command;
use serde_json::Value;
use objc2_foundation::{NSNotificationCenter, NSString, NSObject, NSNotification};
use objc2::{msg_send, rc::Retained};
use block2::StackBlock;

static MONITOR_NAME_CACHE: Mutex<Option<HashMap<u64, String>>> = Mutex::new(None);

pub fn get_monitor_id(monitor: &MonitorHandle) -> MonitorId {
    MonitorId(monitor.native_id() as u64)
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
        .args(&["SPDisplaysDataType", "-json"])
        .output();

    if let Ok(output) = output {
        if let Ok(val) = serde_json::from_slice::<Value>(&output.stdout) {
            if let Some(displays) = val["SPDisplaysDataType"].as_array() {
                for card in displays {
                    if let Some(ndrvs) = card["spdisplays_ndrvs"].as_array() {
                        for display in ndrvs {
                            let name = display["_name"].as_str().unwrap_or("Unknown");
                            let id_str = display["_spdisplays_displayID"].as_str().unwrap_or("");
                            if let Ok(id) = id_str.parse::<u64>() {
                                map.insert(id, name.to_string());
                            }
                        }
                    } else if let Some(_name) = card["_name"].as_str() {
                        // For some built-in displays, the structure might be different
                        // But usually ndrvs is there.
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

pub fn is_oled(_monitor: &MonitorHandle) -> bool {
    // Conceptual: In a real app, we might check model name or HDR capabilities
    false
}
