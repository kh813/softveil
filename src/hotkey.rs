use global_hotkey::{GlobalHotKeyManager, hotkey::{HotKey, Modifiers, Code}, GlobalHotKeyEvent};
use std::sync::mpsc;

#[derive(Debug)]
pub enum HotkeyEvent {
    ToggleGlobal,
}

#[derive(Debug)]
pub enum HotkeyError {
    RegistrationError,
}

pub struct HotkeyGuard {
    _manager: GlobalHotKeyManager,
}

pub fn register(tx: mpsc::Sender<HotkeyEvent>) -> Result<HotkeyGuard, HotkeyError> {
    let manager = GlobalHotKeyManager::new().map_err(|_| HotkeyError::RegistrationError)?;
    
    #[cfg(target_os = "macos")]
    let hotkey = HotKey::new(Some(Modifiers::SUPER | Modifiers::SHIFT), Code::KeyP);
    
    #[cfg(target_os = "windows")]
    let hotkey = HotKey::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyP);
    
    manager.register(hotkey).map_err(|_| HotkeyError::RegistrationError)?;
    
    let id = hotkey.id();
    
    std::thread::spawn(move || {
        let receiver = GlobalHotKeyEvent::receiver();
        loop {
            if let Ok(event) = receiver.recv() {
                if event.id == id && event.state == global_hotkey::HotKeyState::Pressed {
                    let _ = tx.send(HotkeyEvent::ToggleGlobal);
                }
            }
        }
    });

    Ok(HotkeyGuard { _manager: manager })
}
