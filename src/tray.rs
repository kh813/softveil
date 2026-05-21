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
pub const MENU_ID_OVERRIDE_PERIOD_PREFIX: &str = "ov_period:";
pub const MENU_ID_OVERRIDE_COVER_PREFIX: &str = "ov_cover:";
pub const MENU_ID_OVERRIDE_SPEED_PREFIX: &str = "ov_speed:";
pub const MENU_ID_RESET_RECOMMENDED: &str = "reset_recommended:";
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
        #[cfg(target_os = "macos")]
        {
            // macOS UI updates must happen on the main thread to avoid menu displacement.
            // This acts as a runtime check (assertion in debug mode).
            if objc2::MainThreadMarker::new().is_none() {
                #[cfg(debug_assertions)]
                panic!("Tray menu rebuild called from non-main thread on macOS! This is a regression.");
                #[cfg(not(debug_assertions))]
                return; // Silently fail in release but prevent displacement
            }
        }
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
            (FilterMode::HighIntensitySPD,    "SPD プロテクト ✦ (推奨)"),
            (FilterMode::StealthDark,         "ステルス・ダーク (LLCC)"),
            (FilterMode::StealthLight,        "ステルス・ライト (HLCC)"),
            (FilterMode::VerticalLouver,      "標準ルーバー"),
            (FilterMode::AIOcrInterference,   "AI OCR 妨害"),
            (FilterMode::BlackLayer,          "単色（輝度を抑える）"),
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
        for i in 1..=10 {
            let alpha_pct = i * 10;
            let target_alpha = alpha_pct as f32 / 100.0;
            let is_checked = (config.alpha - target_alpha).abs() < 0.01;
            let item = CheckMenuItem::with_id(
                format!("{}{}:{}", MENU_ID_ALPHA_PREFIX, id_str, alpha_pct),
                format!("{}%", alpha_pct),
                true,
                is_checked,
                None,
            );
            let _ = alpha_submenu.append(&item);
        }
        let _ = display_menu.append(&alpha_submenu);

        let fine_tune_submenu = Submenu::new("高度な微調整 (フリッカー対策)", true);
        
        // 1. 縞の太さ (Period)
        let period_submenu = Submenu::new("縞の太さ (Period)", true);
        let periods = [
            (None, "自動 (推奨)"),
            (Some(0.8f32), "細い (0.8mm)"),
            (Some(1.2f32), "標準 (1.2mm)"),
            (Some(1.8f32), "太い (1.8mm)"),
            (Some(2.5f32), "極太 (2.5mm)"),
        ];
        for (val, label) in periods {
            let is_checked = config.override_period_mm == val;
            let item = CheckMenuItem::with_id(
                format!("{}{}:{:?}", MENU_ID_OVERRIDE_PERIOD_PREFIX, id_str, val),
                label,
                true,
                is_checked,
                None,
            );
            let _ = period_submenu.append(&item);
        }
        let _ = fine_tune_submenu.append(&period_submenu);

        // 2. 遮蔽率 (Cover Ratio)
        let cover_submenu = Submenu::new("遮蔽率 (角度プライバシー)", true);
        let covers = [
            (None, "自動 (推奨)"),
            (Some(0.50f32), "低 (50%)"),
            (Some(0.70f32), "標準 (70%)"),
            (Some(0.85f32), "高 (85%)"),
            (Some(0.95f32), "最高 (95%)"),
        ];
        for (val, label) in covers {
            let is_checked = config.override_cover_ratio == val;
            let item = CheckMenuItem::with_id(
                format!("{}{}:{:?}", MENU_ID_OVERRIDE_COVER_PREFIX, id_str, val),
                label,
                true,
                is_checked,
                None,
            );
            let _ = cover_submenu.append(&item);
        }
        let _ = fine_tune_submenu.append(&cover_submenu);

        // 3. スクロール速度 (Scroll Speed)
        let speed_submenu = Submenu::new("スクロール速度 (静止画〜高速)", true);
        let speeds = [
            (None, "自動 (推奨)"),
            (Some(0.0f32), "静止 (0mm/s) - フリッカーなし"),
            (Some(5.0f32), "極低速 (5mm/s)"),
            (Some(20.0f32), "低速 (20mm/s)"),
            (Some(50.0f32), "標準 (50mm/s)"),
        ];
        for (val, label) in speeds {
            let is_checked = config.override_scroll_speed == val;
            let item = CheckMenuItem::with_id(
                format!("{}{}:{:?}", MENU_ID_OVERRIDE_SPEED_PREFIX, id_str, val),
                label,
                true,
                is_checked,
                None,
            );
            let _ = speed_submenu.append(&item);
        }
        let _ = fine_tune_submenu.append(&speed_submenu);

        let _ = display_menu.append(&fine_tune_submenu);

        let _ = display_menu.append(&PredefinedMenuItem::separator());

        let reset_item = MenuItem::with_id(
            format!("{}{}", MENU_ID_RESET_RECOMMENDED, id_str),
            "おすすめ設定を適用 (自動判定)",
            true,
            None,
        );
        let _ = display_menu.append(&reset_item);

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
