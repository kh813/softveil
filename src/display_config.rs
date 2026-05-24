use std::collections::HashMap;
use tao::monitor::MonitorHandle;
use tao::dpi::{PhysicalPosition, PhysicalSize};
use crate::platform;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FilterMode {
    BlackLayer,
    VerticalLouver,
    AIOcrInterference,
    HighIntensitySPD,
    StealthDark,
    StealthLight,
    StealthLightSubpixel,
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
    /// 二方向（格子状）にするかどうか。false の場合は一方向（縦縞）。
    pub bidirectional: bool,
}

impl DisplayProfile {
    /// カテゴリとパネル種別から推奨パラメータを生成する
    pub fn from_config(category: DisplayCategory, panel_type: PanelType) -> Self {
        let mut profile = match category {
            DisplayCategory::NotebookFhd => Self {
                category,
                ppi: 157.0,
                period_mm: 0.50,
                cover_ratio: 0.75,               // 0.55 -> 0.75 (NarrowMask 対応)
                scroll_speed_mm_per_sec: 10.0,   // 0.0 -> 10.0（OCR 対策有効化。IPS 残像許容内）
                phase_flip_hz: 0.0,
                bidirectional: true,             // 横方向の覗き見保護を有効化
            },
            DisplayCategory::NotebookHiDpi => Self {
                category,
                ppi: 220.0,
                period_mm: 0.50,      // 0.40 -> 0.50（MacBook Air 224PPI で stripe=2.4px を確保）
                cover_ratio: 0.80,    // 0.55 -> 0.80（NarrowMask 対応: 暗い部分を大幅に増やす）
                scroll_speed_mm_per_sec: 5.0,
                phase_flip_hz: 0.0,
                bidirectional: true,
            },
            DisplayCategory::ExternalLarge4K => Self {
                category,
                ppi: 163.0,
                period_mm: 0.60,
                cover_ratio: 0.45,
                scroll_speed_mm_per_sec: 10.0,
                phase_flip_hz: 25.0,
                bidirectional: true,
            },
            DisplayCategory::ExternalGeneral => Self {
                category,
                ppi: 92.0,
                period_mm: 0.60,     // 0.40 -> 0.60 (period_px ≈ 2.2px, stripe≈1.0px, gap≈1.2px)
                cover_ratio: 0.45,
                scroll_speed_mm_per_sec: 0.0,
                phase_flip_hz: 28.0,
                bidirectional: false,
            },
            DisplayCategory::Unknown => Self {
                category,
                ppi: 110.0,
                period_mm: 0.50,
                cover_ratio: 0.50,
                scroll_speed_mm_per_sec: 5.0,
                phase_flip_hz: 30.0,
                bidirectional: true,
            },
        };

        // パネル種別による微調整
        if panel_type == PanelType::Oled {
            profile.cover_ratio += 0.05; // OLEDはコントラストが高いので遮蔽率を上げる
            profile.scroll_speed_mm_per_sec *= 1.2; // 応答速度が速いので速く動かせる
        }

        profile
    }

    pub fn recommended_filter_mode(&self, panel_type: PanelType) -> FilterMode {
        match panel_type {
            PanelType::Unknown => FilterMode::BlackLayer,
            _ => FilterMode::HighIntensitySPD,
        }
    }

    pub fn recommended_intensity(&self) -> f32 {
        match self.category {
            DisplayCategory::NotebookHiDpi => 0.75, // 高精細は密度高め
            DisplayCategory::ExternalLarge4K => 1.5, // 大型は密度低め（見やすさ重視）
            _ => 1.0,
        }
    }

    pub fn recommended_alpha(&self) -> f32 {
        0.30 // 標準的な濃度
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

    /// LCD コントラストジャマー用: intensity スケール係数
    pub fn intensity_scale(&self) -> f32 {
        match self.category {
            DisplayCategory::NotebookFhd     => 1.0,
            DisplayCategory::NotebookHiDpi   => 0.8,
            DisplayCategory::ExternalLarge4K => 1.4,
            DisplayCategory::ExternalGeneral => 1.1,
            DisplayCategory::Unknown         => 1.0,
        }
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
    pub fn to_str(self) -> &'static str {
        match self {
            PanelType::Unknown => "Unknown",
            PanelType::Oled => "OLED",
            PanelType::LcdIps => "LCD IPS",
            PanelType::LcdTn => "LCD TN",
        }
    }
}

/// フィルター設定一式をまとめたデータ構造
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FilterSettings {
    pub alpha: f32,
    pub filter_mode: FilterMode,
    pub filter_intensity: f32,
    pub override_period_mm: Option<f32>,
    pub override_cover_ratio: Option<f32>,
    pub override_scroll_speed: Option<f32>,
}

/// 名前付きのプリセット設定
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Preset {
    pub name: String,
    pub settings: FilterSettings,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct MonitorId(pub u64);

impl MonitorId {
    pub fn from_monitor(monitor: &MonitorHandle) -> Self {
        platform::get_monitor_id(monitor)
    }
}

impl std::fmt::Display for MonitorId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{:x}", self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfig {
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    #[serde(default = "default_alpha")]
    pub alpha: f32,
    #[serde(default = "default_filter_mode")]
    pub filter_mode: FilterMode,
    #[serde(default = "default_panel_type")]
    pub panel_type: PanelType,
    #[serde(default = "default_intensity")]
    pub filter_intensity: f32, // Phase 5: フィルター強度 (0.5 - 2.0)
    pub position_key: String,

    /// 自動判定されたディスプレイカテゴリ。手動上書き可能。
    #[serde(default = "default_display_category")]
    pub display_category: DisplayCategory,
    /// 推定 PPI (自動計算)
    #[serde(default = "default_ppi")]
    pub ppi: f32,

    // --- Phase 6: Manual Overrides ---
    /// 縞1周期の物理幅 [mm] の上書き
    #[serde(default)]
    pub override_period_mm: Option<f32>,
    /// 遮蔽率 (0.0〜1.0) の上書き
    #[serde(default)]
    pub override_cover_ratio: Option<f32>,
    /// スクロール速度 [mm/s] の上書き
    #[serde(default)]
    pub override_scroll_speed: Option<f32>,
}

fn default_enabled() -> bool { true }
fn default_alpha() -> f32 { 0.3 }
fn default_filter_mode() -> FilterMode { FilterMode::VerticalLouver }
fn default_panel_type() -> PanelType { PanelType::Unknown }
fn default_intensity() -> f32 { 1.0 }

fn default_display_category() -> DisplayCategory {
    DisplayCategory::Unknown
}

fn default_ppi() -> f32 {
    110.0
}

impl DisplayConfig {
    pub fn alpha_u8(&self) -> u8 {
        (self.alpha * 255.0).round() as u8
    }

    pub fn make_position_key(pos: PhysicalPosition<i32>, size: PhysicalSize<u32>) -> String {
        format!("{}_{}_{}_{}", pos.x, pos.y, size.width, size.height)
    }

    /// 現在の設定（自動判定 + 手動上書き）を反映した Profile を取得する
    pub fn get_effective_profile(&self) -> DisplayProfile {
        let mut profile = DisplayProfile::from_config(self.display_category, self.panel_type);
        
        if let Some(val) = self.override_period_mm {
            profile.period_mm = val;
        }
        if let Some(val) = self.override_cover_ratio {
            profile.cover_ratio = val;
        }
        if let Some(val) = self.override_scroll_speed {
            profile.scroll_speed_mm_per_sec = val;
        }
        
        profile
    }

    /// 現在のフィルター設定を FilterSettings として取得する
    pub fn get_settings(&self) -> FilterSettings {
        FilterSettings {
            alpha: self.alpha,
            filter_mode: self.filter_mode,
            filter_intensity: self.filter_intensity,
            override_period_mm: self.override_period_mm,
            override_cover_ratio: self.override_cover_ratio,
            override_scroll_speed: self.override_scroll_speed,
        }
    }

    /// FilterSettings を現在の設定に適用する
    pub fn apply_settings(&mut self, settings: &FilterSettings) {
        self.alpha = settings.alpha;
        self.filter_mode = settings.filter_mode;
        self.filter_intensity = settings.filter_intensity;
        self.override_period_mm = settings.override_period_mm;
        self.override_cover_ratio = settings.override_cover_ratio;
        self.override_scroll_speed = settings.override_scroll_speed;
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
            override_period_mm: None,
            override_cover_ratio: None,
            override_scroll_speed: None,
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
