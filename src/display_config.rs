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

/// ディスプレイの用途・サイズカテゴリ
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DisplayCategory {
    /// 内蔵ディスプレイ (最大16インチ程度)、FHD 以下 (PPI < 180)
    NotebookFhd,
    /// 内蔵ディスプレイ (最大16インチ程度)、2K/QHD 以上 (PPI >= 180)
    NotebookHiDpi,
    /// 外付け大型モニター、27インチ以上かつ解像度 4K 以上
    ExternalLarge4K,
    /// その他の外付けモニター（FHD/QHD、24インチ前後等）
    ExternalGeneral,
    /// 判定不能（ホットプラグ直後など情報が揃わない場合）
    Unknown,
}

/// カテゴリごとの推奨フィルターパラメータ
#[derive(Debug, Clone, Copy)]
pub struct DisplayProfile {
    #[allow(dead_code)]
    pub category: DisplayCategory,
    /// 推定 PPI
    pub ppi: f32,
    /// 縞1周期の物理幅 [mm]。シェーダーはこれをピクセルに変換して使用する。
    pub period_mm: f32,
    /// 縞の遮蔽率 (0.0〜1.0)。`cover_ratio` に相当。
    pub cover_ratio: f32,
    /// スクロール速度 [mm/s]。シェーダー内でピクセル/秒に変換する。
    pub scroll_speed_mm_per_sec: f32,
    /// 位相反転周波数 [Hz]。FastVibration モードで使用。
    pub phase_flip_hz: f32,
}

impl DisplayProfile {
    /// カテゴリから推奨パラメータを生成する
    pub fn from_category(category: DisplayCategory) -> Self {
        match category {
            DisplayCategory::NotebookFhd => Self {
                category,
                ppi: 157.0,          // 14インチFHD 代表値
                period_mm: 0.96,     // 6px @ 157PPI に相当
                cover_ratio: 0.67,   // 67% 遮蔽
                scroll_speed_mm_per_sec: 48.0,  // 約 300px/s @ 157PPI
                phase_flip_hz: 30.0,
            },
            DisplayCategory::NotebookHiDpi => Self {
                category,
                ppi: 220.0,          // 14インチ QHD/Retina 代表値
                period_mm: 0.82,     // 視距離が近いため密度を上げる
                cover_ratio: 0.70,
                scroll_speed_mm_per_sec: 55.0,  // 視距離が近いため速く見せる
                phase_flip_hz: 30.0,
            },
            DisplayCategory::ExternalLarge4K => Self {
                category,
                ppi: 163.0,          // 27インチ 4K 代表値
                period_mm: 1.80,     // 視距離が遠いため周期を広げる
                cover_ratio: 0.62,   // 広い縞でも視覚ノイズを確保
                scroll_speed_mm_per_sec: 80.0,  // 視距離が遠いため速度を上げる
                phase_flip_hz: 25.0,
            },
            DisplayCategory::ExternalGeneral => Self {
                category,
                ppi: 92.0,           // 27インチ FHD 代表値
                period_mm: 1.30,
                cover_ratio: 0.65,
                scroll_speed_mm_per_sec: 60.0,
                phase_flip_hz: 28.0,
            },
            DisplayCategory::Unknown => Self {
                category,
                ppi: 110.0,
                period_mm: 1.20,
                cover_ratio: 0.65,
                scroll_speed_mm_per_sec: 50.0,
                phase_flip_hz: 30.0,
            },
        }
    }

    /// PPI から period_px (シェーダーに渡す値) を計算する
    pub fn period_px(&self, ppi: f32) -> f32 {
        let ppi = if ppi > 0.0 { ppi } else { self.ppi };
        self.period_mm * ppi / 25.4
    }

    /// scroll_speed_mm_per_sec から scroll_speed_px (シェーダーに渡す値) を計算する
    pub fn scroll_speed_px(&self, ppi: f32) -> f32 {
        let ppi = if ppi > 0.0 { ppi } else { self.ppi };
        self.scroll_speed_mm_per_sec * ppi / 25.4
    }
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

    /// 自動判定されたディスプレイカテゴリ。手動上書き可能。
    #[serde(default = "default_display_category")]
    pub display_category: DisplayCategory,
    /// 推定 PPI (自動計算)
    #[serde(default = "default_ppi")]
    pub ppi: f32,
}

fn default_display_category() -> DisplayCategory {
    DisplayCategory::Unknown
}

fn default_ppi() -> f32 {
    110.0
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
            display_category: DisplayCategory::Unknown,
            ppi: 110.0,
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
