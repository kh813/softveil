use std::collections::HashMap;
use tao::monitor::MonitorHandle;
use tao::dpi::{PhysicalPosition, PhysicalSize};
use crate::platform;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FilterMode {
    BlackLayer,
    VerticalLouver,
    FastVibration,
    AsymmetricCurve,
    AIOcrInterference,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PanelType {
    Unknown,
    Oled,
    LcdIps,
    LcdTn,
}

impl PanelType {
    pub fn to_str(&self) -> &'static str {
        match self {
            PanelType::Unknown => "Unknown",
            PanelType::Oled => "OLED",
            PanelType::LcdIps => "LCD IPS",
            PanelType::LcdTn => "LCD TN",
        }
    }

    pub fn recommended_filter_mode(&self) -> FilterMode {
        match self {
            PanelType::Oled => FilterMode::VerticalLouver,
            PanelType::LcdIps => FilterMode::FastVibration,
            PanelType::LcdTn => FilterMode::AsymmetricCurve,
            PanelType::Unknown => FilterMode::BlackLayer,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct MonitorId(pub u64);

impl MonitorId {
    pub fn from_monitor(monitor: &MonitorHandle) -> Self {
        platform::get_monitor_id(monitor)
    }

    pub fn to_string(&self) -> String {
        format!("0x{:x}", self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfig {
    pub enabled: bool,
    pub alpha: f32,
    pub filter_mode: FilterMode,
    pub panel_type: PanelType,
    pub filter_intensity: f32, // Phase 5: フィルター強度 (0.5 - 2.0)
    pub position_key: String,
}

impl DisplayConfig {
    pub fn default() -> Self {
        <Self as Default>::default()
    }

    pub fn alpha_u8(&self) -> u8 {
        (self.alpha * 255.0).round() as u8
    }

    pub fn make_position_key(pos: PhysicalPosition<i32>, size: PhysicalSize<u32>) -> String {
        format!("{}_{}_{}_{}", pos.x, pos.y, size.width, size.height)
    }
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            alpha: 0.30,
            filter_mode: FilterMode::BlackLayer,
            panel_type: PanelType::Unknown,
            filter_intensity: 1.0,
            position_key: String::new(),
        }
    }
}

pub struct DisconnectedCache {
    cache: HashMap<String, DisplayConfig>,
    order: Vec<String>,
    max_entries: usize,
}

impl DisconnectedCache {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            order: Vec::new(),
            max_entries: 8,
        }
    }

    pub fn store(&mut self, config: DisplayConfig) {
        let key = config.position_key.clone();
        if self.cache.contains_key(&key) {
            self.order.retain(|k| k != &key);
        }
        self.cache.insert(key.clone(), config);
        self.order.push(key);

        if self.order.len() > self.max_entries {
            let old_key = self.order.remove(0);
            self.cache.remove(&old_key);
        }
    }

    pub fn restore(&mut self, key: &str) -> Option<DisplayConfig> {
        if let Some(config) = self.cache.remove(key) {
            self.order.retain(|k| k != key);
            Some(config)
        } else {
            None
        }
    }
}
