use crate::app::AppState;
use crate::display_config::FilterMode;
use crate::overlay::OverlayWindow;
use tray_icon::{TrayIcon, TrayIconBuilder, Icon};
use muda::{Menu, MenuItem, Submenu, PredefinedMenuItem, CheckMenuItem};

pub const MENU_ID_GLOBAL_TOGGLE: &str = "global_toggle";
pub const MENU_ID_DISPLAY_TOGGLE_PREFIX: &str = "display_toggle:";
pub const MENU_ID_ALPHA_PREFIX: &str = "alpha:";
pub const MENU_ID_MODE_PREFIX: &str = "mode:";
pub const MENU_ID_PANEL_PREFIX: &str = "panel:";
pub const MENU_ID_CATEGORY_PREFIX: &str = "category:";
pub const MENU_ID_INTENSITY_PREFIX: &str = "intensity:";
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

        let builder = TrayIconBuilder::new()
            .with_icon(icon)
            .with_menu(Box::new(menu))
            .with_tooltip("Softveil");

        #[cfg(target_os = "macos")]
        let builder = builder.with_icon_as_template(true);

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

    for overlay in overlays {
        let id_str = overlay.monitor_id.to_string();
        let display_menu = Submenu::new(&overlay.monitor_name, true);
        
        let config = state.displays.get(&overlay.monitor_id).cloned().unwrap_or_default();

        let category_label = match config.display_category {
            crate::display_config::DisplayCategory::NotebookFhd      => "ノートPC FHD",
            crate::display_config::DisplayCategory::NotebookHiDpi    => "ノートPC 高解像度",
            crate::display_config::DisplayCategory::ExternalLarge4K  => "外付け大型 4K",
            crate::display_config::DisplayCategory::ExternalGeneral  => "外付け 標準",
            crate::display_config::DisplayCategory::Unknown          => "不明",
        };
        let category_info_item = MenuItem::with_id(
            "category_info",
            format!("画面タイプ: {} (PPI: {:.0})", category_label, config.ppi),
            false,
            None,
        );
        let _ = display_menu.append(&category_info_item);

        let toggle_item = CheckMenuItem::with_id(
            format!("{}{}", MENU_ID_DISPLAY_TOGGLE_PREFIX, id_str),
            "フィルターを有効",
            true,
            config.enabled,
            None,
        );
        let _ = display_menu.append(&toggle_item);

        let category_submenu = Submenu::new("画面タイプを変更", true);
        let categories = [
            (crate::display_config::DisplayCategory::NotebookFhd,     "ノートPC FHD (14インチ 1080p)"),
            (crate::display_config::DisplayCategory::NotebookHiDpi,   "ノートPC 高解像度 (14インチ 2K/Retina)"),
            (crate::display_config::DisplayCategory::ExternalLarge4K, "外付け大型 4K (27〜32インチ 4K)"),
            (crate::display_config::DisplayCategory::ExternalGeneral, "外付け 標準 (24インチ FHD/QHD)"),
        ];
        for (cat, label) in categories {
            let item = CheckMenuItem::with_id(
                format!("{}{}:{:?}", MENU_ID_CATEGORY_PREFIX, id_str, cat),
                label,
                true,
                config.display_category == cat,
                None,
            );
            let _ = category_submenu.append(&item);
        }
        let _ = display_menu.append(&category_submenu);

        let panel_submenu = Submenu::new(format!("パネル種別 ({})", config.panel_type.to_str()), true);
        let panels = [
            (crate::display_config::PanelType::Unknown, "Unknown (不明)"),
            (crate::display_config::PanelType::Oled, "OLED (有機EL)"),
            (crate::display_config::PanelType::LcdIps, "LCD IPS (液晶)"),
            (crate::display_config::PanelType::LcdTn, "LCD TN (液晶)"),
        ];
        for (panel, label) in panels {
            let item = CheckMenuItem::with_id(
                format!("{}{}:{:?}", MENU_ID_PANEL_PREFIX, id_str, panel),
                label,
                true,
                config.panel_type == panel,
                None,
            );
            let _ = panel_submenu.append(&item);
        }
        let _ = display_menu.append(&panel_submenu);

        let mode_submenu = Submenu::new("フィルター形式", true);
        let modes = [
            (FilterMode::BlackLayer, "単色レイヤー"),
            (FilterMode::VerticalLouver, "縦縞ルーバー"),
            (FilterMode::FastVibration, "高速動体マスキング"),
            (FilterMode::AsymmetricCurve, "非対称曲線パターン"),
            (FilterMode::AIOcrInterference, "AI OCR 妨害テクスチャ"),
        ];
        for (mode, label) in modes {
            let item = CheckMenuItem::with_id(
                format!("{}{}:{:?}", MENU_ID_MODE_PREFIX, id_str, mode),
                label,
                true,
                config.filter_mode == mode,
                None,
            );
            let _ = mode_submenu.append(&item);
        }
        let _ = display_menu.append(&mode_submenu);

        let intensity_submenu = Submenu::new("フィルター強度", true);
        let intensities = [
            (0.5f32, "最高 (密度高)"),
            (0.75f32, "高"),
            (1.0f32, "標準"),
            (1.5f32, "低"),
            (2.0f32, "最低 (密度低)"),
        ];
        for (intensity, label) in intensities {
            let is_checked = (config.filter_intensity * 100.0).round() == (intensity * 100.0).round();
            let item = CheckMenuItem::with_id(
                format!("{}{}:{}", MENU_ID_INTENSITY_PREFIX, id_str, intensity),
                label,
                true,
                is_checked,
                None,
            );
            let _ = intensity_submenu.append(&item);
        }
        let _ = display_menu.append(&intensity_submenu);

        let alpha_submenu = Submenu::new("フィルター濃度", true);
        for i in 1..=9 {
            let alpha_pct = i * 10;
            let is_checked = (config.alpha * 10.0).round() == i as f32;
            let item = CheckMenuItem::with_id(
                format!("{}{}:{}", MENU_ID_ALPHA_PREFIX, id_str, alpha_pct),
                &format!("{}%", alpha_pct),
                true,
                is_checked,
                None,
            );
            let _ = alpha_submenu.append(&item);
        }
        let _ = display_menu.append(&alpha_submenu);

        let _ = menu.append(&display_menu);
    }

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
