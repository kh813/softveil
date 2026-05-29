use crate::app::AppState;
use crate::display_config::FilterMode;
use crate::overlay::OverlayWindow;
use crate::display_config::MonitorId;
use tray_icon::{TrayIcon, TrayIconBuilder, Icon};
use muda::{Menu, MenuItem, Submenu, PredefinedMenuItem, CheckMenuItem};
use std::sync::Mutex;
use std::collections::HashMap;
use crate::i18n::t;

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
pub const MENU_ID_AI_MODE_OFF: &str = "ai_mode_off";
pub const MENU_ID_AI_MODE_VIGILANCE: &str = "ai_mode_vigilance";
pub const MENU_ID_AI_MODE_ENHANCED: &str = "ai_mode_enhanced";
pub const MENU_ID_PRESET_APPLY_PREFIX: &str = "preset_apply:";
pub const MENU_ID_PRESET_DELETE_PREFIX: &str = "preset_delete:";
pub const MENU_ID_PRESET_SAVE_CURRENT: &str = "preset_save_current";
pub const MENU_ID_RUN_BENCHMARK_PREFIX: &str = "run_benchmark:";
pub const MENU_ID_RUN_BENCHMARK_ALL: &str = "run_benchmark_all";
pub const MENU_ID_QUIT: &str = "quit";

pub struct TrayHandle {
    icon: TrayIcon,
    // Store references to menu items that need surgical updates (e.g. progress labels)
    // Key: Option<MonitorId> where None is Global, Some is per-monitor
    benchmark_items: Mutex<HashMap<Option<MonitorId>, MenuItem>>,
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

        let (menu, benchmark_items) = build_menu(state, overlays);

        let builder = TrayIconBuilder::new()
            .with_icon(icon)
            .with_menu(Box::new(menu))
            .with_tooltip("Softveil");

        #[cfg(target_os = "macos")]
        let builder = builder.with_icon_as_template(true);

        let tray_icon = builder.build().map_err(|e| TrayError::IconError(e.to_string()))?;

        Ok(Self {
            icon: tray_icon,
            benchmark_items: Mutex::new(benchmark_items),
        })
    }

    pub fn rebuild_menu(&self, state: &AppState, overlays: &[OverlayWindow]) {
        #[cfg(target_os = "macos")]
        {
            if objc2::MainThreadMarker::new().is_none() {
                #[cfg(debug_assertions)]
                panic!("Tray menu rebuild called from non-main thread on macOS!");
                #[cfg(not(debug_assertions))]
                return;
            }
        }
        let (menu, benchmark_items) = build_menu(state, overlays);
        self.icon.set_menu(Some(Box::new(menu)));
        if let Ok(mut map) = self.benchmark_items.lock() {
            *map = benchmark_items;
        }
    }

    pub fn update_benchmark_progress(&self, progress: f32, monitor_id: Option<MonitorId>) {
        if let Ok(map) = self.benchmark_items.lock() {
            if let Some(item) = map.get(&monitor_id) {
                let pct = (progress * 100.0).round() as u32;
                let base_text = if monitor_id.is_none() {
                    t("全画面を最適化する", "Optimize All Screens")
                } else {
                    t("この画面を個別で最適化", "Optimize This Screen")
                };
                let _ = item.set_text(format!("{} ({}%) ...", base_text, pct));
            }
        }
    }

    pub fn set_benchmark_running(&self, running: bool) {
        if let Ok(map) = self.benchmark_items.lock() {
            for (id, item) in map.iter() {
                let _ = item.set_enabled(!running);
                if !running {
                    let base_text = if id.is_none() {
                        t("全画面を最適化する", "Optimize All Screens")
                    } else {
                        t("この画面を個別で最適化", "Optimize This Screen")
                    };
                    let _ = item.set_text(base_text);
                }
            }
        }
    }

    pub fn set_tooltip(&self, tooltip: &str) {
        let _ = self.icon.set_tooltip(Some(tooltip));
    }
}

pub fn build_menu(state: &AppState, overlays: &[OverlayWindow]) -> (Menu, HashMap<Option<MonitorId>, MenuItem>) {
    let menu = Menu::new();
    let mut benchmark_items = HashMap::new();

    let global_toggle = CheckMenuItem::with_id(
        MENU_ID_GLOBAL_TOGGLE,
        t("フィルター：すべてオン", "Filter: All On"),
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
            crate::display_config::DisplayCategory::NotebookFhd      => t("ノートPC FHD", "Notebook FHD"),
            crate::display_config::DisplayCategory::NotebookHiDpi    => t("ノートPC 高解像度", "Notebook HiDPI"),
            crate::display_config::DisplayCategory::ExternalLarge4K  => t("外付け大型 4K", "External Large 4K"),
            crate::display_config::DisplayCategory::ExternalGeneral  => t("外付け 標準", "External Standard"),
            crate::display_config::DisplayCategory::Unknown          => t("不明", "Unknown"),
        };
        
        // ── グループ1: 基本・ON/OFF ──
        let category_info_item = MenuItem::with_id(
            "category_info",
            format!("{}: {} (PPI: {:.0})", t("画面タイプ", "Display Type"), category_label, config.ppi),
            false,
            None,
        );
        let _ = display_menu.append(&category_info_item);

        let toggle_item = CheckMenuItem::with_id(
            format!("{}{}", MENU_ID_DISPLAY_TOGGLE_PREFIX, id_str),
            t("フィルターを有効", "Enable Filter"),
            true,
            config.enabled,
            None,
        );
        let _ = display_menu.append(&toggle_item);

        let _ = display_menu.append(&PredefinedMenuItem::separator());

        // ── グループ2: フィルター調整 ──
        let mode_submenu = Submenu::new(t("フィルター形式", "Filter Mode"), true);
        let modes = [
            (FilterMode::HighIntensitySPD,    t("SPD プロテクト ✦ (推奨)", "SPD Protect ✦ (Rec.)")),
            (FilterMode::StealthDark,         t("ステルス・ダーク (LLCC)", "Stealth Dark (LLCC)")),
            (FilterMode::StealthLight,        t("ステルス・ライト (Subpixel)", "Stealth Light (Subpixel)")),
            (FilterMode::VerticalLouver,      t("標準ルーバー", "Standard Louver")),
            (FilterMode::OcrJammer,           t("OCR妨害 (カメラ撮影対策)", "OCR Jammer (Anti-Camera)")),
            (FilterMode::BlackLayer,          t("単色（輝度を抑える）", "Solid Color (Dim)")),
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

        let alpha_submenu = Submenu::new(t("フィルター濃度", "Filter Alpha"), true);
        for i in 0..=10 {
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

        let intensity_submenu = Submenu::new(t("フィルター強度", "Filter Intensity"), true);
        let intensities = [
            (0.5f32, t("最高 (密度高)", "Highest (High Density)")),
            (0.75f32, t("高", "High")),
            (1.0f32, t("標準", "Standard")),
            (1.5f32, t("低", "Low")),
            (2.0f32, t("最低 (密度低)", "Lowest (Low Density)")),
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

        let _ = display_menu.append(&PredefinedMenuItem::separator());

        // ── グループ3: プリセット・リセット ──
        let display_preset_submenu = Submenu::new(t("設定プリセット", "Presets"), true);
        if state.presets.is_empty() {
             let _ = display_preset_submenu.append(&MenuItem::with_id("empty", t("(プリセットなし)", "(No Presets)"), false, None));
        } else {
            let apply_submenu = Submenu::new(t("プリセット適用", "Apply Preset"), true);
            for preset in &state.presets {
                let is_active = config.matches_settings(&preset.settings);
                let item = CheckMenuItem::with_id(
                    format!("{}{}:{}", MENU_ID_PRESET_APPLY_PREFIX, id_str, preset.name),
                    preset.name.clone(),
                    true,
                    is_active,
                    None,
                );
                let _ = apply_submenu.append(&item);
            }
            let _ = display_preset_submenu.append(&apply_submenu);

            let delete_submenu = Submenu::new(t("プリセット削除", "Delete Preset"), true);
            for preset in &state.presets {
                let item = MenuItem::with_id(
                    format!("{}{}", MENU_ID_PRESET_DELETE_PREFIX, preset.name),
                    preset.name.clone(),
                    true,
                    None,
                );
                let _ = delete_submenu.append(&item);
            }
            let _ = display_preset_submenu.append(&delete_submenu);
        }
        let _ = display_preset_submenu.append(&MenuItem::with_id(
            format!("{}{}", MENU_ID_PRESET_SAVE_CURRENT, id_str),
            t("現在の設定を保存...", "Save Current Settings..."),
            true,
            None
        ));
        let _ = display_menu.append(&display_preset_submenu);

        let reset_item = MenuItem::with_id(
            format!("{}{}", MENU_ID_RESET_RECOMMENDED, id_str),
            t("おすすめ設定に戻す", "Reset to Recommended"),
            true,
            None,
        );
        let _ = display_menu.append(&reset_item);

        let _ = display_menu.append(&PredefinedMenuItem::separator());

        // ── グループ4: 詳細設定 ──
        let fine_tune_submenu = Submenu::new(t("高度な微調整", "Advanced Fine-Tuning"), true);
        
        let period_submenu = Submenu::new(t("縞の太さ (Period)", "Stripe Thickness (Period)"), true);
        let periods = [
            (None, t("自動 (推奨)", "Auto (Recommended)")),
            (Some(0.8f32), t("細い (0.8mm)", "Thin (0.8mm)")),
            (Some(1.2f32), t("標準 (1.2mm)", "Standard (1.2mm)")),
            (Some(1.8f32), t("太い (1.8mm)", "Thick (1.8mm)")),
            (Some(2.5f32), t("極太 (2.5mm)", "Very Thick (2.5mm)")),
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

        let cover_submenu = Submenu::new(t("遮蔽率 (Cover Ratio)", "Cover Ratio"), true);
        let covers = [
            (None, t("自動 (推奨)", "Auto (Recommended)")),
            (Some(0.50f32), t("低 (50%)", "Low (50%)")),
            (Some(0.70f32), t("標準 (70%)", "Standard (70%)")),
            (Some(0.85f32), t("高 (85%)", "High (85%)")),
            (Some(0.95f32), t("最高 (95%)", "Highest (95%)")),
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

        let speed_submenu = Submenu::new(t("スクロール速度", "Scroll Speed"), true);
        let speeds = [
            (None, t("自動 (推奨)", "Auto (Recommended)")),
            (Some(0.0f32), t("静止 (0mm/s)", "Static (0mm/s)")),
            (Some(5.0f32), t("極低速 (5mm/s)", "Very Slow (5mm/s)")),
            (Some(20.0f32), t("低速 (20mm/s)", "Slow (20mm/s)")),
            (Some(50.0f32), t("標準 (50mm/s)", "Standard (50mm/s)")),
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

        let panel_submenu = Submenu::new(format!("{} ({})", t("パネル種別を変更", "Change Panel Type"), config.panel_type.to_str()), true);
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

        let category_submenu = Submenu::new(t("画面タイプを変更", "Change Display Type"), true);
        let categories = [
            (crate::display_config::DisplayCategory::NotebookFhd,     t("ノートPC FHD", "Notebook FHD")),
            (crate::display_config::DisplayCategory::NotebookHiDpi,   t("ノートPC 高解像度", "Notebook HiDPI")),
            (crate::display_config::DisplayCategory::ExternalLarge4K, t("外付け大型 4K", "External Large 4K")),
            (crate::display_config::DisplayCategory::ExternalGeneral, t("外付け 標準", "External Standard")),
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

        let _ = menu.append(&display_menu);
    }

    let _ = menu.append(&PredefinedMenuItem::separator());

    // ── グローバル最適化 (ベンチマーク) サブメニューへの統合 ──
    let optimize_submenu = Submenu::new(t("最適化 (ベンチマーク)", "Optimization (Benchmark)"), true);
    
    let benchmark_label = if let Some(progress) = state.benchmark_progress {
        format!("{} ({:.0}%) ...", t("全画面を最適化中", "Optimizing all screens"), progress * 100.0)
    } else {
        t("全画面を最適化する", "Optimize All Screens").to_string()
    };
    let run_benchmark_all_item = MenuItem::with_id(
        MENU_ID_RUN_BENCHMARK_ALL, 
        benchmark_label, 
        state.benchmark_progress.is_none(), 
        None
    );
    let _ = optimize_submenu.append(&run_benchmark_all_item);
    benchmark_items.insert(None, run_benchmark_all_item);

    let _ = optimize_submenu.append(&PredefinedMenuItem::separator());

    for overlay in overlays {
        let id_str = overlay.monitor_id.to_string();
        let optimize_item = MenuItem::with_id(
            format!("{}{}", MENU_ID_RUN_BENCHMARK_PREFIX, id_str),
            format!("{} {}", overlay.monitor_name, t("を個別で最適化", "Optimize (Individual)")),
            state.benchmark_progress.is_none(),
            None,
        );
        let _ = optimize_submenu.append(&optimize_item);
        benchmark_items.insert(Some(overlay.monitor_id), optimize_item);
    }
    let _ = menu.append(&optimize_submenu);

    let _ = menu.append(&PredefinedMenuItem::separator());

    let (ai_status_text, is_off, is_vigilance, is_enhanced) = if state.ai_vigilance_mode {
        (t("✓ Vigilance", "✓ Vigilance"), false, true, false)
    } else if state.ai_detection_enabled {
        (t("✓ 常時フィルター強化", "✓ Enhanced Filter"), false, false, true)
    } else {
        (t("オフ", "Off"), true, false, false)
    };

    let ai_submenu = Submenu::new(format!("{} ▶ [{}]", t("AI 覗き見検知", "AI Peep Prevention"), ai_status_text), true);

    let ai_off_item = CheckMenuItem::with_id(
        MENU_ID_AI_MODE_OFF,
        t("オフ", "Off"),
        true,
        is_off,
        None,
    );
    let _ = ai_submenu.append(&ai_off_item);

    let ai_vigilance_item = CheckMenuItem::with_id(
        MENU_ID_AI_MODE_VIGILANCE,
        t("Vigilance（検知時のみ展開）", "Vigilance (Deploy on Detect)"),
        true,
        is_vigilance,
        None,
    );
    let _ = ai_submenu.append(&ai_vigilance_item);

    let ai_enhanced_item = CheckMenuItem::with_id(
        MENU_ID_AI_MODE_ENHANCED,
        t("常時フィルター強化", "Enhanced Filter Always On"),
        true,
        is_enhanced,
        None,
    );
    let _ = ai_submenu.append(&ai_enhanced_item);

    let _ = menu.append(&ai_submenu);

    let auto_start_item = CheckMenuItem::with_id(
        MENU_ID_AUTO_START,
        t("ログイン時に起動", "Start at Login"),
        true,
        state.auto_start,
        None,
    );
    let _ = menu.append(&auto_start_item);

    let _ = menu.append(&PredefinedMenuItem::separator());

    let quit_item = MenuItem::with_id(MENU_ID_QUIT, t("Softveil を終了", "Quit Softveil"), true, None);
    let _ = menu.append(&quit_item);

    (menu, benchmark_items)
}
