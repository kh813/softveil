use std::collections::HashMap;
use crate::display_config::{DisplayConfig, MonitorId};
use serde::{Serialize, Deserialize};

const APP_NAME: &str = "softveil";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FilterMode {
    BlackLayer,
    Louver,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub global_enabled: bool,
    pub default_alpha: f32,
    pub auto_start: bool,
    pub filter_mode: FilterMode,
    pub ai_detection_enabled: bool,
    pub display_settings: HashMap<String, DisplayConfig>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            global_enabled: true,
            default_alpha: 0.30,
            auto_start: false,
            filter_mode: FilterMode::BlackLayer,
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
    pub filter_mode: FilterMode,
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
                position_key: String::new(),
            },
            auto_start: config.auto_start,
            filter_mode: config.filter_mode,
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
            auto_start: self.auto_start,
            filter_mode: self.filter_mode,
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
    pub fn set_filter_mode(&mut self, mode: FilterMode) {
        self.filter_mode = mode;
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

    pub fn set_global_alpha(&mut self, alpha: f32) {
        let alpha = alpha.clamp(0.0, 1.0);
        self.default_config.alpha = alpha;
        for config in self.displays.values_mut() {
            config.alpha = alpha;
        }
    }

    pub fn is_visible(&self, id: &MonitorId) -> bool {
        if !self.global_enabled {
            return false;
        }
        self.displays.get(id).map(|c| c.enabled).unwrap_or(false)
    }

    pub fn add_display(&mut self, id: MonitorId, config: Option<DisplayConfig>) {
        let config = config.unwrap_or_else(|| self.default_config.clone());
        self.displays.insert(id, config);
    }

    pub fn add_display_with_pos(&mut self, id: MonitorId, pos_key: String) {
        let mut config = self.stored_display_settings.get(&pos_key).cloned().unwrap_or_else(|| {
            self.default_config.clone()
        });
        config.position_key = pos_key;
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
