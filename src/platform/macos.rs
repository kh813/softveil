use tao::monitor::MonitorHandle;
use tao::window::Window;
use tao::platform::macos::MonitorHandleExtMacOS;
use tao::platform::macos::WindowExtMacOS;
use crate::display_config::MonitorId;
use crate::platform::DisplayChangeEvent;
use objc2_app_kit::{NSWindow, NSWindowCollectionBehavior, NSColor};
use std::sync::mpsc;
use objc2_foundation::{NSNotificationCenter, NSString, NSObject, NSNotification};
use objc2::{msg_send, rc::Retained};
use block2::StackBlock;

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

pub fn register_hotplug_observer(tx: mpsc::Sender<DisplayChangeEvent>) -> HotplugGuard {
    let center = NSNotificationCenter::defaultCenter();
    let notification_name = NSString::from_str("NSApplicationDidChangeScreenParametersNotification");
    
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
