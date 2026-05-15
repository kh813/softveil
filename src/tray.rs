use crate::app::{AppState, FilterMode};
use crate::overlay::OverlayWindow;
use tray_icon::{TrayIcon, TrayIconBuilder, Icon};
use muda::{Menu, MenuItem, Submenu, PredefinedMenuItem, CheckMenuItem};

pub const MENU_ID_GLOBAL_TOGGLE: &str = "global_toggle";
pub const MENU_ID_DISPLAY_TOGGLE_PREFIX: &str = "display_toggle:";
pub const MENU_ID_ALPHA_PREFIX: &str = "alpha:";
pub const MENU_ID_MODE_PREFIX: &str = "mode:";
pub const MENU_ID_AUTO_START: &str = "auto_start";
pub const MENU_ID_AI_DETECTION: &str = "ai_detection";
pub const MENU_ID_QUIT: &str = "quit";

pub struct TrayHandle {
    icon: TrayIcon,
}

#[derive(Debug)]
pub enum TrayError {
    IconError(#[allow(dead_code)] String),
}

impl TrayHandle {
    pub fn new(state: &AppState, overlays: &[OverlayWindow]) -> Result<Self, TrayError> {
        let icon_bytes = if cfg!(target_os = "macos") {
            include_bytes!("../assets/icon_macos_template.png").as_slice()
        } else {
            include_bytes!("../assets/icon_windows.ico").as_slice()
        };

        let img = image::load_from_memory(icon_bytes).map_err(|e| TrayError::IconError(e.to_string()))?;
        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();
        let icon = Icon::from_rgba(rgba.into_raw(), width, height).map_err(|e| TrayError::IconError(e.to_string()))?;

        let menu = build_menu(state, overlays);

        let mut builder = TrayIconBuilder::new()
            .with_icon(icon)
            .with_menu(Box::new(menu))
            .with_tooltip("Softveil");

        #[cfg(target_os = "macos")]
        {
            // Note: Template icons on macOS should be black shapes with transparency.
            // Tao/tray-icon uses this to allow the system to tint the icon.
            builder = builder.with_icon_as_template(true);
        }

        let tray_icon = builder.build().map_err(|e| TrayError::IconError(e.to_string()))?;

        Ok(Self {
            icon: tray_icon,
        })
    }

    pub fn rebuild_menu(&self, state: &AppState, overlays: &[OverlayWindow]) {
        let menu = build_menu(state, overlays);
        self.icon.set_menu(Some(Box::new(menu)));
    }
}

fn build_menu(state: &AppState, overlays: &[OverlayWindow]) -> Menu {
    let menu = Menu::new();

    let global_toggle = CheckMenuItem::with_id(
        MENU_ID_GLOBAL_TOGGLE,
        "フィルター：すべてオン",
        true,
        state.all_displays_enabled(),
        None,
    );
    let _ = menu.append(&global_toggle);

    let _ = menu.append(&PredefinedMenuItem::separator());

    let display_submenu = Submenu::new("ディスプレイ設定", true);
    for overlay in overlays {
        let id_str = overlay.monitor_id.to_string();
        let enabled = state.displays.get(&overlay.monitor_id).map(|c| c.enabled).unwrap_or(true);
        let item = CheckMenuItem::with_id(
            format!("{}{}", MENU_ID_DISPLAY_TOGGLE_PREFIX, id_str),
            &overlay.monitor_name,
            true,
            enabled,
            None,
        );
        let _ = display_submenu.append(&item);
    }
    let _ = menu.append(&display_submenu);

    let alpha_submenu = Submenu::new("フィルター濃度", true);
    for i in 1..=9 {
        let alpha_pct = i * 10;
        // Find if any display has this alpha (just use default_config's alpha for global check)
        let is_checked = (state.default_config.alpha * 10.0).round() == i as f32;
        let item = CheckMenuItem::with_id(
            format!("{}{}", MENU_ID_ALPHA_PREFIX, alpha_pct),
            &format!("{}%", alpha_pct),
            true,
            is_checked,
            None,
        );
        let _ = alpha_submenu.append(&item);
    }
    let _ = menu.append(&alpha_submenu);

    let mode_submenu = Submenu::new("フィルター形式", true);
    let modes = [
        (FilterMode::BlackLayer, "単色レイヤー"),
        (FilterMode::Louver, "縦縞ルーバー"),
    ];
    for (mode, label) in modes {
        let item = CheckMenuItem::with_id(
            format!("{}{:?}", MENU_ID_MODE_PREFIX, mode),
            label,
            true,
            state.filter_mode == mode,
            None,
        );
        let _ = mode_submenu.append(&item);
    }
    let _ = menu.append(&mode_submenu);

    let _ = menu.append(&PredefinedMenuItem::separator());

    let auto_start_item = CheckMenuItem::with_id(
        MENU_ID_AUTO_START,
        "ログイン時に起動",
        true,
        state.auto_start,
        None,
    );
    let _ = menu.append(&auto_start_item);

    let ai_detection_item = CheckMenuItem::with_id(
        MENU_ID_AI_DETECTION,
        "AI 覗き見検知",
        true,
        state.ai_detection_enabled,
        None,
    );
    let _ = menu.append(&ai_detection_item);

    let _ = menu.append(&PredefinedMenuItem::separator());

    let quit_item = MenuItem::with_id(MENU_ID_QUIT, "Softveil を終了", true, None);
    let _ = menu.append(&quit_item);

    menu
}
