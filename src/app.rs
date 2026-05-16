use std::collections::HashMap;
use crate::display_config::{DisplayConfig, MonitorId, FilterMode, DisplayCategory};
use serde::{Serialize, Deserialize};

const APP_NAME: &str = "softveil";

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub global_enabled: bool,
    pub default_alpha: f32,
    pub default_filter_mode: FilterMode,
    pub auto_start: bool,
    pub ai_detection_enabled: bool,
    pub display_settings: HashMap<String, DisplayConfig>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            global_enabled: true,
            default_alpha: 0.30,
            default_filter_mode: FilterMode::BlackLayer,
            auto_start: false,
            ai_detection_enabled: false,
            display_settings: HashMap::new(),
        }
    }
}

pub struct AppState {
    pub global_enabled: bool,
    pub displays: HashMap<MonitorId, DisplayConfig>,
    pub default_config: DisplayConfig,
    pub auto_start: bool,
    pub ai_detection_enabled: bool,
    pub ai_peeper_detected: bool,
    stored_display_settings: HashMap<String, DisplayConfig>,
}

impl AppState {
    pub fn new() -> Self {
        let config: AppConfig = confy::load(APP_NAME, None).unwrap_or_default();

        Self {
            global_enabled: config.global_enabled,
            displays: HashMap::new(),
            default_config: DisplayConfig {
                enabled: true,
                alpha: config.default_alpha,
                filter_mode: config.default_filter_mode,
                panel_type: crate::display_config::PanelType::Unknown,
                filter_intensity: 1.0,
                position_key: String::new(),
                display_category: DisplayCategory::Unknown,
                ppi: 110.0,
            },
            auto_start: config.auto_start,
            ai_detection_enabled: config.ai_detection_enabled,
            ai_peeper_detected: false,
            stored_display_settings: config.display_settings,
        }
    }

    pub fn save(&self) {
        let mut display_settings = self.stored_display_settings.clone();
        for config in self.displays.values() {
            if !config.position_key.is_empty() {
                display_settings.insert(config.position_key.clone(), config.clone());
            }
        }

        let config = AppConfig {
            global_enabled: self.global_enabled,
            default_alpha: self.default_config.alpha,
            default_filter_mode: self.default_config.filter_mode,
            auto_start: self.auto_start,
            ai_detection_enabled: self.ai_detection_enabled,
            display_settings,
        };

        let _ = confy::store(APP_NAME, None, config);
    }

    pub fn toggle_ai_detection(&mut self) -> bool {
        self.ai_detection_enabled = !self.ai_detection_enabled;
        if !self.ai_detection_enabled {
            self.ai_peeper_detected = false;
        }
        self.ai_detection_enabled
    }

    pub fn set_peeper_detected(&mut self, detected: bool) {
        self.ai_peeper_detected = detected;
    }

    pub fn set_filter_mode(&mut self, id: &MonitorId, mode: FilterMode) {
        if let Some(config) = self.displays.get_mut(id) {
            config.filter_mode = mode;
        }
    }

    pub fn toggle_global(&mut self) -> bool {
        self.global_enabled = !self.global_enabled;
        self.global_enabled
    }

    pub fn toggle_display(&mut self, id: &MonitorId) -> bool {
        if let Some(config) = self.displays.get_mut(id) {
            config.enabled = !config.enabled;
            config.enabled
        } else {
            false
        }
    }

    pub fn toggle_auto_start(&mut self) -> bool {
        self.auto_start = !self.auto_start;
        self.auto_start
    }

    pub fn set_display_alpha(&mut self, id: &MonitorId, alpha: f32) {
        if let Some(config) = self.displays.get_mut(id) {
            config.alpha = alpha.clamp(0.0, 1.0);
        }
    }

    pub fn is_visible(&self, id: &MonitorId) -> bool {
        if !self.global_enabled {
            return false;
        }
        self.displays.get(id).map(|c| c.enabled).unwrap_or(false)
    }

    pub fn filter_mode(&self, id: &MonitorId) -> FilterMode {
        self.displays.get(id).map(|c| c.filter_mode).unwrap_or(FilterMode::BlackLayer)
    }

    pub fn filter_intensity(&self, id: &MonitorId) -> f32 {
        self.displays.get(id).map(|c| c.filter_intensity).unwrap_or(1.0)
    }

    pub fn set_filter_intensity(&mut self, id: &MonitorId, intensity: f32) {
        if let Some(config) = self.displays.get_mut(id) {
            config.filter_intensity = intensity.clamp(0.1, 5.0);
        }
    }

    pub fn panel_type(&self, id: &MonitorId) -> crate::display_config::PanelType {
        self.displays.get(id).map(|c| c.panel_type).unwrap_or(crate::display_config::PanelType::Unknown)
    }

    pub fn set_panel_type(&mut self, id: &MonitorId, panel_type: crate::display_config::PanelType) {
        if let Some(config) = self.displays.get_mut(id) {
            let old_panel = config.panel_type;
            config.panel_type = panel_type;
            
            // If the filter mode is default (BlackLayer) or we are transitioning from Unknown,
            // apply the recommended filter mode for the new panel type.
            if config.filter_mode == FilterMode::BlackLayer || old_panel == crate::display_config::PanelType::Unknown {
                config.filter_mode = panel_type.recommended_filter_mode();
            }
        }
    }

    pub fn display_category(&self, id: &MonitorId) -> DisplayCategory {
        self.displays.get(id)
            .map(|c| c.display_category)
            .unwrap_or(DisplayCategory::Unknown)
    }

    pub fn set_display_category(&mut self, id: &MonitorId, category: DisplayCategory, ppi: f32) {
        if let Some(config) = self.displays.get_mut(id) {
            config.display_category = category;
            config.ppi = ppi;
        }
    }

    pub fn add_display(&mut self, id: MonitorId, config: Option<DisplayConfig>) {
        let config = config.unwrap_or_else(|| self.default_config.clone());
        self.displays.insert(id, config);
    }

    pub fn add_display_with_pos_and_profile(
        &mut self,
        id: MonitorId,
        pos_key: String,
        category: DisplayCategory,
        ppi: f32,
    ) {
        let mut config = self.stored_display_settings.get(&pos_key).cloned().unwrap_or_else(|| {
            self.default_config.clone()
        });
        config.position_key = pos_key;

        // 保存済み設定が Unknown の場合のみ自動検出値で上書き
        if config.display_category == DisplayCategory::Unknown {
            config.display_category = category;
            config.ppi = ppi;
        }

        // パネル種別の推奨フィルターモードも Unknown の場合に適用
        if config.filter_mode == FilterMode::BlackLayer {
            config.filter_mode = config.panel_type.recommended_filter_mode();
        }

        self.displays.insert(id, config);
    }

    pub fn remove_display(&mut self, id: &MonitorId) -> Option<DisplayConfig> {
        self.displays.remove(id)
    }

    pub fn all_displays_enabled(&self) -> bool {
        if !self.global_enabled {
            return false;
        }
        if self.displays.is_empty() {
            return false;
        }
        self.displays.values().all(|c| c.enabled)
    }

    pub fn effective_alpha_u8(&self, id: &MonitorId) -> u8 {
        if self.ai_peeper_detected {
            return 204; // 80% alpha
        }
        self.displays.get(id).map(|c| c.alpha_u8()).unwrap_or(77)
    }
}
