# 覗き見防止フィルターアプリ「Softveil」基本仕様書

**バージョン:** 1.5
**作成日:** 2026年5月
**ステータス:** MVP ドラフト

**変更履歴:**

| バージョン | 変更内容 |
|-----------|---------|
| 1.0 | 初版作成 |
| 1.1 | ダブルクリック起動・メニューバー/システムトレイ常駐をMVPに格上げ。ライブラリを `tao` に決定 |
| 1.2 | ライブラリバージョン固定・`Cargo.toml` 全文追加。フォルダ構成・モジュール責務・関数仕様を新設（§5〜§7） |
| 1.3 | ホットプラグ対応・ディスプレイごとの個別設定をMVPに格上げ。`display_config.rs` モジュール新設。`AppState`・`OverlayWindow`・トレイメニュー構成を更新 |
| 1.4 | ビルド・配布形式テーブル追加（§2.4）。`Cargo.toml [package]` にバイナリ名・バンドルID明記。設計哲学（§12）・非設計事項（§13）を新設。未解決事項を Known Issues 形式に整理（§9） |
| 1.5 | F-10 にユースケース記述・グローバルOFFとの優先関係・デフォルト値の根拠を追記。F-07 メニュー例をノートPC+外付けモニターの具体例に更新 |

---

## 1. 目的・概要

### 1.1 背景と課題

カフェやオープンオフィスなど、公共の場でのPC作業において、周囲からの画面の覗き見はセキュリティリスクとなる。物理的な覗き見防止フィルムは貼り付けが手間であり、持ち運びや付け替えも煩わしい。

### 1.2 プロダクトの概要

**Softveil** は、macOS / Windows の画面全体に半透明のデジタルフィルター（ルーバー模様）をソフトウェアで実現するデスクトップアプリケーションである。

- 物理フィルムと異なり、**いつでもオン／オフの切り替えや濃度変更が可能**
- インストール後はバックグラウンドで常駐し、必要なときだけ有効化できる
- フィルター表示中も、背後にあるアプリの操作性を一切損なわない

### 1.3 スコープ（MVP）

本仕様書は **MVP（Minimum Viable Product）** を対象とする。

| 機能カテゴリ | MVP | Ver 2.0以降 |
|------------|-----|------------|
| フィルター表示（半透明黒レイヤー） | ✅ | ― |
| クリック透過・常時最前面 | ✅ | ― |
| .app / .exe ダブルクリック起動 | ✅ | ― |
| メニューバー（macOS）常駐アイコン | ✅ | ― |
| システムトレイ（Windows）常駐アイコン | ✅ | ― |
| マルチディスプレイ対応 | ✅ | ― |
| **ホットプラグ対応（接続・切断の自動検知）** | ✅ | ― |
| **ディスプレイごとの個別設定（Alpha・ON/OFF）** | ✅ | ― |
| 縦縞ルーバーパターン | ― | ✅ |
| 濃度スライダー UI | ― | ✅ |
| AI 覗き見検知 | ― | ✅（Ver 3.0） |

---

## 2. 動作環境・開発要件

### 2.1 開発環境

| 項目 | 内容 |
|------|------|
| 開発マシン | macOS (Apple Silicon / Intel) |
| 開発言語 | Rust (Edition 2021) |
| ビルドツール | Cargo |
| Windowsクロスコンパイル | `cross` クレート または GitHub Actions による CI ビルド |

### 2.2 ターゲット環境

| OS | バージョン | アーキテクチャ |
|----|-----------|---------------|
| macOS | 12 Monterey 以降 | Apple Silicon (aarch64) / Intel (x86_64) |
| Windows | 10 / 11 (64bit) | x86_64 |

### 2.3 ビルド・配布形式

| コンテキスト | プラットフォーム | 成果物 | 備考 |
|:---|:---|:---|:---|
| `make` (ローカルビルド) | macOS Apple Silicon | `Softveil.app` | `cargo build --release --target aarch64-apple-darwin` |
| `make` (ローカルビルド) | macOS Intel | `Softveil.app` | `cargo build --release --target x86_64-apple-darwin` |
| `make all` (macOS から) | macOS ユニバーサル | `Softveil.app` | `lipo` でユニバーサルバイナリ生成 |
| GitHub Actions `windows-latest` | Windows x64 | `Softveil.exe` | `x86_64-pc-windows-msvc`; リリースタグで起動 |

> **Windows ビルド**: MSVC SDK が必要なため macOS からのクロスコンパイル不可。GitHub Actions の `windows-latest` ランナーでビルドし、リリースアセットとしてアップロードする。

> **macOS Gatekeeper**: 公証（notarization）なしで配布すると初回起動時に警告が出る。配布方法（`.app` 直配布 / `.dmg` / 署名付き）は未解決事項 #4 を参照。

### 2.4 依存クレートとバージョン

#### Cargo.toml（全文）

```toml
[package]
name = "softveil"
version = "0.1.0"
edition = "2021"
description = "Software privacy filter for macOS and Windows"
# macOS .app バンドル識別子（Info.plist の CFBundleIdentifier と一致させること）
# → com.yourname.softveil
# Windows 多重起動ミューテックス名: "Local\\SoftveilMutex"

[[bin]]
name = "softveil"
path = "src/main.rs"

[dependencies]
# ── ウィンドウ管理・イベントループ ──────────────────────────────
tao = { version = "0.30", features = ["rwh_06"] }

# ── システムトレイ / メニューバー ───────────────────────────────
tray-icon = "0.19"
muda       = "0.15"

# ── グローバルショートカット ────────────────────────────────────
global-hotkey = "0.6"

# ── 2D 描画（フィルター塗りつぶし）────────────────────────────
softbuffer = "0.4"

# ── macOS ネイティブ API ────────────────────────────────────────
[target.'cfg(target_os = "macos")'.dependencies]
objc2             = "0.5"
objc2-app-kit     = { version = "0.2", features = ["NSWindow", "NSColor", "NSScreen"] }
objc2-foundation  = { version = "0.2", features = ["NSString"] }
core-graphics     = { version = "0.24", features = ["highsierra"] }

# ── Windows ネイティブ API ─────────────────────────────────────
[target.'cfg(target_os = "windows")'.dependencies]
windows-sys = { version = "0.59", features = [
    "Win32_Foundation",
    "Win32_UI_WindowsAndMessaging",
    "Win32_Graphics_Gdi",
    "Win32_System_Threading",
] }

[profile.release]
opt-level     = 3
lto           = true
codegen-units = 1
strip         = true     # バイナリからデバッグシンボルを除去してサイズ削減
```

#### クレート選定の根拠

| クレート | バージョン | 選定理由 |
|---------|-----------|---------|
| `tao` | 0.30 | Tauri プロジェクト由来。`tray-icon` との統合実績が豊富。`rwh_06` feature で `raw-window-handle` 0.6 系と統一 |
| `tray-icon` | 0.19 | `tao` と同一エコシステム。macOS メニューバー・Windows トレイを同一 API で操作可能 |
| `muda` | 0.15 | `tray-icon` と同梱に近い関係。コンテキストメニューの構築に特化 |
| `global-hotkey` | 0.6 | `tray-icon` と同一作者。`tao` イベントループとの統合が自然 |
| `softbuffer` | 0.4 | GPU 不要で CPU のみで動作するフレームバッファ描画。フィルターは静的な単色塗りつぶしで十分なため `pixels` や `wgpu` より軽量 |
| `objc2` 系 | 0.5 / 0.2 | Safe Rust から Objective-C API を呼ぶ現行標準。旧 `objc` クレートより型安全 |
| `windows-sys` | 0.59 | FFI バインディング。`windows` クレートより軽量で必要な feature のみ有効化できる |

> **`tiny-skia` を外した理由:** フィルターはアルファ値付きの単色矩形塗りつぶしのみ（MVP）。`softbuffer` のフレームバッファに直接 RGBA 値を書き込む方が依存が少なく高速。縦縞ルーバー（Ver 2.0）の時点で `tiny-skia` を追加する。

---

## 3. 機能要件

### F-01　オーバーレイ・フィルター表示

- 起動時、接続されているすべてのディスプレイを個別に検出し、それぞれに対してフルスクリーンの「枠なし（Borderless）」ウィンドウを生成する
- デフォルトのフィルター表示は「単一の半透明黒レイヤー」とする（§4 参照）
- 将来拡張として「縦縞ルーバーパターン」を追加する（Ver 2.0）

### F-02　クリック透過（Pass-through Input）

- フィルターウィンドウが前面に表示された状態でも、マウスクリック・ドラッグ・スクロール・キーボード入力などすべての操作イベントを下のウィンドウへ素通りさせること
- **これはアプリの根幹要件であり、妥協不可**

### F-03　常時最前面（Always on Top）表示

- 他アプリがアクティブになった場合、新規ウィンドウが開いた場合でも、フィルターは常に最前面に位置すること
- スクリーンセーバーやロック画面との干渉は MVP では考慮対象外

### F-04　フィルターのオン／オフ切り替え

- メニューバー（macOS）/ システムトレイ（Windows）のコンテキストメニューから切り替えられること
- フィルター非表示時もアプリはバックグラウンドで常駐し続けること
- グローバルショートカット（`Cmd+Shift+P` / `Ctrl+Shift+P`）でも同様に切り替え可能であること

### F-05　マルチディスプレイ対応

- 接続ディスプレイ数を動的に取得し、各ディスプレイに独立したオーバーレイウィンドウを生成する
- 各ディスプレイは OS が発行する一意の **モニターID**（macOS: `CGDirectDisplayID`、Windows: `HMONITOR`）で識別し、設定と紐付ける

### F-06　ダブルクリック起動

- macOS では `.app` バンドル形式、Windows では `.exe` 形式で配布し、ターミナルなしで起動できること
- 起動後はただちにメニューバー / システムトレイにアイコンが現れ、フィルターが有効化されること
- 多重起動を防止する

### F-09　ホットプラグ対応（ディスプレイの接続・切断）

- **ディスプレイ接続時:** 新しいディスプレイを検知し、直ちにオーバーレイウィンドウを生成・表示する。このとき既存ディスプレイの設定は維持する
- **ディスプレイ切断時:** 該当ディスプレイのオーバーレイウィンドウを破棄し、`Vec` から除去する
- 検知方法はプラットフォームごとに異なる（§7.4 / §7.5 参照）
- ホットプラグ時に新規ディスプレイへ適用する設定はグローバルデフォルト値を使用する

### F-10　ディスプレイごとの個別設定

#### 想定ユースケース

カフェやオープンオフィスなど、複数ディスプレイを使う場面での典型的な使い方：

| 構成 | ノートPC画面 | 外付けモニター |
|:---|:---|:---|
| **プライバシー重視** | ON（周囲から見られる可能性あり） | ON |
| **作業効率重視** | OFF（手元で快適に見たい） | ON（来客・隣席から見える） |
| **プレゼン中** | OFF（手元の手順を見たい） | OFF（聴衆に見せる） |

デフォルトは**全ディスプレイON**。ユーザーがディスプレイごとに後から変更する想定。

#### 設定項目

各ディスプレイに対して以下の設定を独立して保持・変更できること。

| 設定項目 | 型 | デフォルト値 | 説明 |
|---------|---|------------|------|
| `enabled` | `bool` | `true` | そのディスプレイのフィルターのON/OFF |
| `alpha` | `f32` | `0.30` | フィルター濃度（0.0〜1.0） |

> **デフォルト `true` の根拠**: 起動直後はすべてのディスプレイを保護するほうが安全側。必要に応じてユーザーが特定ディスプレイをOFFにする操作フローを想定している。

#### グローバルOFFとの優先関係

```
is_visible(display) = global_enabled AND display.enabled
```

- **グローバルOFF**（`global_enabled = false`）のとき、個別設定に関わらず全フィルターが非表示になる
- グローバルをONに戻したとき、個別設定は変更されていないため、OFFにしていたディスプレイはOFFのまま維持される
- この2段階構造により「一時的に全部消す」と「特定ディスプレイだけ常時OFF」を使い分けられる

#### 操作方法

- 個別設定の変更はトレイ/メニューバーの**「ディスプレイ設定」サブメニュー**から行う（§F-07 参照）
- 全ディスプレイ一括ON/OFFは「フィルター：すべてオン/オフ」およびグローバルショートカットで操作する

#### 設定の引き継ぎ（ホットプラグ時）

ディスプレイが切断・再接続された場合、モニターIDが変化する可能性があるため、設定の引き継ぎは「同一位置・同一解像度」のヒューリスティックで試みる（完全一致しない場合はデフォルト値 `enabled=true, alpha=0.30` を適用する）。

### F-07　メニューバー / システムトレイ常駐（改訂）

アプリは常にバックグラウンドに常駐し、以下のコンテキストメニューを提供する。

**例: ノートPC画面はOFF、外付けモニターはONの状態**

```
[ ] フィルター：すべてオン          ← 一部OFFのため ✓ なし
─────────────────────────────────
ディスプレイ設定 ▶
  ├─ [ ] Built-in Display (2560×1600)   ← ノートPC画面: OFF
  └─ [✓] DELL U2723D (2560×1440)        ← 外付けモニター: ON
─────────────────────────────────
Softveil を終了
```

**全ディスプレイONの状態**

```
[✓] フィルター：すべてオン          ← 全てONのとき ✓
─────────────────────────────────
ディスプレイ設定 ▶
  ├─ [✓] Built-in Display (2560×1600)
  └─ [✓] DELL U2723D (2560×1440)
─────────────────────────────────
Softveil を終了
```

- ディスプレイ名は OS から取得した名前（例: "Built-in Display"、"DELL U2723D"）を使用する
- **「フィルター：すべてオン」のチェックマーク**は、全ディスプレイの `enabled` がすべて `true` のときのみ表示される（1つでも `false` があれば外れる）
- 個別ON/OFFのチェックマークはディスプレイごとの `enabled` 状態に連動する
- ディスプレイが接続・切断されるたびにサブメニューを動的に再構築する

**macOS 固有:** Dock にアイコンを表示しない（`LSUIElement = true`）
**Windows 固有:** タスクバーに表示しない（`WS_EX_TOOLWINDOW`）、右クリックでメニュー表示

### F-08　アプリケーションの完全終了

- コンテキストメニューの「終了」からのみ完全終了できる
- 終了時にすべてのオーバーレイウィンドウを破棄し、トレイアイコンを削除してからプロセスを終了する

---

## 4. UI・表示仕様

### 4.1 ウィンドウ設定

| 設定項目 | 値 |
|---------|---|
| タイトルバー | 非表示 |
| ウィンドウ枠 | 非表示 |
| タスクバー / Dock へのアイコン表示 | 非表示 |
| サイズ | 各ディスプレイの解像度に合わせてフルスクリーン |

### 4.2 フィルターパターン

#### パターン A：半透明黒レイヤー（MVP）

| パラメータ | デフォルト値 | 説明 |
|-----------|------------|------|
| 背景色 | `#000000` | 黒 |
| 不透明度（Alpha） | `0.30`（30%） | ユーザーの視認性とプライバシー保護のバランス点 |

フレームバッファへの書き込み値は ARGB 32bit 形式で `0x4C000000`（Alpha = 77 ≒ 30%）。

#### パターン B：縦縞ルーバー（Ver 2.0 拡張）

| パラメータ | デフォルト値 | 説明 |
|-----------|------------|------|
| 縞の幅 | 1px 黒 : 1px 透明 | 物理的なルーバーフィルムを模倣 |
| 不透明度 | 黒ピクセルのみ `0xFF` | 斜め視点からは黒のみ見えるルーバー効果 |

---

## 5. フォルダ構成

```
softveil/
├── Cargo.toml
├── Cargo.lock
├── build.rs                        # Windows: アイコン埋め込み（winres）
│
├── assets/
│   ├── icon.png                    # 元アイコン（1024×1024 推奨）
│   ├── icon_macos.icns             # macOS .app バンドル用
│   ├── icon_macos_template.png     # メニューバー用テンプレート画像（22×22 @2x, 白黒）
│   └── icon_windows.ico            # Windows トレイ用（16/32/48px マルチサイズ）
│
├── src/
│   ├── main.rs                     # エントリポイント。起動フロー全体を orchestrate
│   ├── app.rs                      # AppState 構造体。グローバル状態＋ディスプレイ別設定を管理
│   ├── display_config.rs           # DisplayConfig 構造体。ディスプレイ別設定の保持・引き継ぎロジック
│   ├── overlay.rs                  # オーバーレイウィンドウの生成・描画・ホットプラグ対応
│   ├── tray.rs                     # トレイ/メニューバーアイコン・ディスプレイ別サブメニュー
│   ├── hotkey.rs                   # グローバルショートカットの登録と監視
│   ├── single_instance.rs          # 多重起動防止（プラットフォーム別実装）
│   │
│   └── platform/
│       ├── mod.rs                  # プラットフォーム共通インターフェース定義
│       ├── macos.rs                # macOS 固有: NSWindow 操作・ホットプラグ検知
│       └── windows.rs              # Windows 固有: HWND 操作・WM_DISPLAYCHANGE 処理
│
├── resources/
│   └── windows/
│       └── app.rc                  # Windows リソースファイル（アイコン・バージョン情報）
│
└── package/
    ├── macos/
    │   └── Info.plist              # .app バンドル用（LSUIElement 等）
    └── windows/
        └── softveil.wxs     # WiX インストーラー定義（Ver 2.0 以降）
```

### ファイル責務サマリー

| ファイル | 責務 |
|---------|------|
| `main.rs` | 初期化順序の制御。各モジュールの呼び出し元。`tao` イベントループの起動 |
| `app.rs` | グローバルなフィルターON/OFF状態と、ディスプレイ別 `DisplayConfig` のマップを管理 |
| `display_config.rs` | ディスプレイ1枚分の設定（`enabled`・`alpha`）を保持。切断後の再接続時の設定引き継ぎロジック |
| `overlay.rs` | `tao` ウィンドウの生成・`softbuffer` 描画・ホットプラグ時の追加/削除処理 |
| `tray.rs` | `tray-icon` / `muda` を使ったアイコン・メニューのセットアップ。ディスプレイ別サブメニューの動的再構築 |
| `hotkey.rs` | `global-hotkey` によるキー登録とイベント受信スレッドの管理 |
| `single_instance.rs` | 多重起動チェック。macOS はロックファイル、Windows は名前付きミューテックス |
| `platform/macos.rs` | `NSWindow` 操作・`NSApplicationDidChangeScreenParametersNotification` によるホットプラグ検知 |
| `platform/windows.rs` | `HWND` 操作・`WM_DISPLAYCHANGE` メッセージによるホットプラグ検知 |

---

## 6. モジュール設計・実装方針

### 6.1 main.rs　─ エントリポイント

**責務:** 起動フロー全体の制御と `tao` イベントループの実行。

```
fn main()
  │
  ├─ single_instance::acquire()        → 多重起動チェック（失敗したら即 exit）
  ├─ let event_loop = EventLoop::new() → tao イベントループ生成
  ├─ let (tx, rx) = mpsc::channel()    → ショートカット通知チャネル生成
  ├─ hotkey::register(tx)              → ショートカット登録（別スレッド起動）
  ├─ let monitors = event_loop.available_monitors().collect()
  ├─ let mut state = AppState::new()
  ├─ let mut overlays = overlay::create_all(&event_loop, &monitors)
  ├─ tray::setup(&event_loop)          → トレイ/メニューバー初期化
  └─ event_loop.run(|event, _, flow| {
        // tray イベント処理
        // hotkey チャネル受信処理
        // 終了処理
     })
```

**実装方針:**
- `event_loop.run()` のクロージャ内でのみウィンドウ操作を行う（`tao` の制約）
- `AppState` は `Rc<RefCell<AppState>>` でクロージャにキャプチャする（シングルスレッドで十分）
- パニックによる異常終了時もトレイアイコンが残らないよう `std::panic::set_hook` で後処理を登録する

---

### 6.2 app.rs　─ アプリ状態管理

**責務:** グローバルなフィルターON/OFF状態と、ディスプレイIDをキーとした設定マップを管理する。

```rust
use std::collections::HashMap;
use crate::display_config::{DisplayConfig, MonitorId};

pub struct AppState {
    /// 全ディスプレイ一括ON/OFFのグローバルスイッチ
    /// false のとき、個別設定に関係なく全フィルターを非表示にする
    pub global_enabled: bool,

    /// ディスプレイIDをキーとした個別設定マップ
    /// キー: MonitorId（プラットフォーム固有のモニター識別子をラップした型）
    pub displays: HashMap<MonitorId, DisplayConfig>,

    /// 新規ディスプレイ接続時に適用するデフォルト設定
    pub default_config: DisplayConfig,
}

impl AppState {
    pub fn new() -> Self

    /// グローバルスイッチをトグルし、新しい global_enabled を返す
    pub fn toggle_global(&mut self) -> bool

    /// 特定ディスプレイのON/OFFをトグルし、新しい enabled を返す
    pub fn toggle_display(&mut self, id: &MonitorId) -> bool

    /// 特定ディスプレイのAlpha値を設定する（0.0〜1.0 にクランプ）
    pub fn set_alpha(&mut self, id: &MonitorId, alpha: f32)

    /// あるディスプレイを実際に描画すべきか判定する
    /// global_enabled && displays[id].enabled の両方が true のときのみ true
    pub fn is_visible(&self, id: &MonitorId) -> bool

    /// 新規ディスプレイ追加時に DisplayConfig を登録する
    /// すでに同IDが存在する場合は何もしない（再接続時は keep_or_restore を使う）
    pub fn add_display(&mut self, id: MonitorId, config: Option<DisplayConfig>)

    /// ディスプレイ切断時にマップから除去し、設定を返す（再接続時の引き継ぎ用）
    pub fn remove_display(&mut self, id: &MonitorId) -> Option<DisplayConfig>

    /// 全ディスプレイがONかどうか（トレイのグローバルチェックマーク表示用）
    pub fn all_displays_enabled(&self) -> bool
}
```

**実装方針:**
- `AppState` 自体はロジックのみを持ち、ウィンドウ操作は `overlay.rs` が担う
- `global_enabled = false` のとき `is_visible()` は常に `false` を返し、個別設定を上書きする
- `Rc<RefCell<AppState>>` でイベントループクロージャにキャプチャする

---

### 6.3 overlay.rs　─ オーバーレイウィンドウ管理

**責務:** 各ディスプレイへのウィンドウ生成・フィルター描画・ホットプラグ時の追加/削除。

```rust
use crate::display_config::MonitorId;

pub struct OverlayWindow {
    pub monitor_id: MonitorId,            // このウィンドウが属するディスプレイのID
    pub monitor_name: String,             // OS から取得したディスプレイ名（トレイメニュー表示用）
    pub window: tao::window::Window,
    surface: softbuffer::Surface<...>,
}

impl OverlayWindow {
    /// ディスプレイ1枚に対応したオーバーレイウィンドウを生成し、
    /// プラットフォーム固有の設定（クリック透過・最前面）を適用する
    pub fn new(
        event_loop: &EventLoop<()>,
        monitor: &MonitorHandle,
    ) -> Result<Self, OverlayError>

    /// フレームバッファに alpha 値付きの黒を塗りつぶして描画する
    pub fn draw(&mut self, alpha: u8) -> Result<(), OverlayError>

    /// ウィンドウの表示・非表示を切り替える
    pub fn set_visible(&self, visible: bool)

    /// alpha 値を変更して即座に再描画する
    pub fn update_alpha(&mut self, alpha: u8) -> Result<(), OverlayError>
}

/// 全ディスプレイのオーバーレイを一括生成する
pub fn create_all(
    event_loop: &EventLoop<()>,
    monitors: &[MonitorHandle],
) -> Vec<OverlayWindow>

/// ホットプラグ：新規ディスプレイのオーバーレイを追加生成してVecに追加する
/// 戻り値: 追加した OverlayWindow の monitor_id
pub fn add_display(
    overlays: &mut Vec<OverlayWindow>,
    event_loop: &EventLoop<()>,
    monitor: &MonitorHandle,
    visible: bool,
    alpha: u8,
) -> Result<MonitorId, OverlayError>

/// ホットプラグ：切断されたディスプレイのオーバーレイを Vec から削除する
pub fn remove_display(
    overlays: &mut Vec<OverlayWindow>,
    id: &MonitorId,
)

/// AppState を参照して全ウィンドウの表示状態・Alpha値を一括同期する
pub fn sync_all(overlays: &mut Vec<OverlayWindow>, state: &AppState)
```

**実装方針:**
- `monitor_id` は `platform::get_monitor_id(&monitor)` で取得する（OS固有の数値をラップ）
- `monitor_name` は `monitor.name()` から取得し、`None` の場合は `"Display N"` とする
- ホットプラグ検知は `platform/` モジュールが行い、`main.rs` のイベントループへカスタムイベントとして通知する（後述 §6.9）
- `sync_all` は設定変更のたびに呼ぶことで、`AppState` との乖離を防ぐ

---

### 6.4 tray.rs　─ トレイ / メニューバー UI

**責務:** アイコン表示・コンテキストメニューのセットアップと、ディスプレイ別サブメニューの動的再構築。

```rust
pub const MENU_ID_GLOBAL_TOGGLE:   &str = "global_toggle";
pub const MENU_ID_DISPLAY_TOGGLE:  &str = "display_toggle:";  // suffix: MonitorId文字列
pub const MENU_ID_QUIT:            &str = "quit";

pub struct TrayHandle {
    _icon: tray_icon::TrayIcon,
    menu: muda::Menu,   // 再構築のため保持
}

impl TrayHandle {
    /// トレイアイコンと初期メニューを生成して返す
    pub fn new(state: &AppState, overlays: &[OverlayWindow]) -> Result<Self, TrayError>

    /// ディスプレイ構成の変化（ホットプラグ等）に応じてメニュー全体を再構築する
    pub fn rebuild_menu(&self, state: &AppState, overlays: &[OverlayWindow])

    /// グローバルチェックマークのみを更新する（ホットプラグなしのトグル操作用）
    pub fn update_global_check(&self, all_enabled: bool)

    /// 特定ディスプレイのチェックマークを更新する
    pub fn update_display_check(&self, id: &MonitorId, enabled: bool)
}
```

**メニュー構成（`muda` で構築）:**

```
[✓] フィルター：すべてオン               ← MENU_ID_GLOBAL_TOGGLE
──────────────────────────────────────
ディスプレイ設定 ▶ （Submenu）
  [✓] Built-in Display (2560×1600)     ← MENU_ID_DISPLAY_TOGGLE:"0x1a2b3c"
  [✓] DELL U2723D (2560×1440)          ← MENU_ID_DISPLAY_TOGGLE:"0x4d5e6f"
──────────────────────────────────────
Softveil を終了                    ← MENU_ID_QUIT
```

**実装方針:**
- `MENU_ID_DISPLAY_TOGGLE` の suffix として `MonitorId` を文字列化したものを付与し、受信側でパースしてどのディスプレイか判定する
- ホットプラグ発生時は `rebuild_menu()` を呼び出してサブメニュー全体を差し替える
- `muda` のメニュー項目は `CheckMenuItem` を使い、`set_checked()` でチェックマークを更新する

---

### 6.5 hotkey.rs　─ グローバルショートカット

**責務:** `global-hotkey` によるキー登録と、別スレッドでのイベント受信。

```rust
/// グローバルショートカットを登録し、キーが押されるたびに
/// `tx` にユニットを送信するスレッドを起動する。
/// 戻り値の `HotkeyGuard` を drop するとスレッドが停止し登録が解除される。
pub fn register(tx: mpsc::Sender<HotkeyEvent>) -> Result<HotkeyGuard, HotkeyError>

pub enum HotkeyEvent {
    ToggleFilter,
}

pub struct HotkeyGuard {
    _manager: GlobalHotKeyManager,  // drop で登録解除
}
```

**実装方針:**
- キーコード: macOS = `Modifiers::SUPER | Modifiers::SHIFT + Code::KeyP`、Windows = `Modifiers::CONTROL | Modifiers::SHIFT + Code::KeyP`
- `GlobalHotKeyEvent::receiver()` をスレッド内でブロッキング受信し、`tx.send()` でメインループに通知する
- macOS では初回起動時にアクセシビリティ権限ダイアログが出る。ユーザーへの案内はトレイメニューの Tooltip に文言を入れる方針（→ 未解決事項 #2）

---

### 6.6 single_instance.rs　─ 多重起動防止

**責務:** 2つ目以降のプロセス起動を検知して即座に終了させる。

```rust
/// 多重起動チェックを行い、ロックを取得して `SingleInstanceGuard` を返す。
/// すでに起動中であれば `Err(AlreadyRunning)` を返す。
/// `SingleInstanceGuard` を drop するとロックが解放される。
pub fn acquire() -> Result<SingleInstanceGuard, SingleInstanceError>

pub struct SingleInstanceGuard {
    // macOS: ロックファイルの File ハンドル（drop で unlock）
    // Windows: HANDLE（drop で CloseHandle）
    inner: PlatformGuard,
}
```

**macOS 実装:**
- `$TMPDIR/softveil.lock` に `O_CREAT | O_EXCL` でファイル作成（アトミック）
- `flock(LOCK_EX | LOCK_NB)` でロック取得。失敗（`EWOULDBLOCK`）なら既起動と判断
- `Drop` で `flock(LOCK_UN)` とファイル削除

**Windows 実装:**
- `CreateMutexW(NULL, TRUE, "Local\\SoftveilMutex")` で名前付きミューテックス作成
- `GetLastError() == ERROR_ALREADY_EXISTS` なら既起動と判断
- `Drop` で `ReleaseMutex` / `CloseHandle`

---

### 6.7 platform/macos.rs　─ macOS 固有処理

**責務:** `objc2` 経由で `NSWindow` に macOS 固有のウィンドウプロパティを設定する。

```rust
/// ウィンドウに以下をまとめて適用する:
///   - ignoringMouseEvents = true（クリック透過）
///   - backgroundColor = クリア（softbuffer の描画を透過させる）
///   - level = NSStatusWindowLevel + 1（常時最前面）
///   - collectionBehavior に .canJoinAllSpaces | .stationary を追加（Spaces 対応）
pub fn apply_overlay_settings(window: &tao::window::Window)

/// ウィンドウレベルを定数で返す。フルスクリーン対応の検証結果に応じて変更する。
/// 現在値: NSStatusWindowLevel (25)
/// 候補値: kCGScreenSaverWindowLevel (1000) ─ フルスクリーンアプリの上に出たい場合
pub fn overlay_window_level() -> i64
```

**実装方針:**
- `raw-window-handle` の `HasWindowHandle::window_handle()` から `AppKitWindowHandle` を取り出し、`NSWindow` ポインタを unsafe で取得する
- `objc2-app-kit` の型付き API を使い、できる限り unsafe ブロックを最小化する
- `collectionBehavior` に `.canJoinAllSpaces` を追加することで、macOS の全 Space（仮想デスクトップ）でフィルターが表示されるようにする

---

### 6.8 platform/windows.rs　─ Windows 固有処理

**責務:** `windows-sys` 経由で `HWND` に Windows 固有のウィンドウスタイルを設定する。

```rust
/// HWND に以下をまとめて適用する:
///   - WS_EX_TRANSPARENT | WS_EX_LAYERED | WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW を付与
///   - SetLayeredWindowAttributes で Alpha = 指定値 を設定
///   - SetWindowPos で HWND_TOPMOST に配置
pub fn apply_overlay_settings(window: &tao::window::Window, alpha: u8)

/// 既存の拡張スタイルに追加フラグを OR で合成して SetWindowLongPtrW を呼ぶ
fn set_ex_style(hwnd: HWND, additional_flags: u32)
```

**実装方針:**
- `raw-window-handle` の `Win32WindowHandle` から `HWND` を取り出す
- すべての Win32 API 呼び出しは `unsafe` ブロックに集約し、呼び出し元から unsafe を隠蔽する
- `WS_EX_TRANSPARENT` だけでは管理者権限ウィンドウの前面で透過しない場合があるが、MVP では対応しない（→ 未解決事項 #6）
- `SetLayeredWindowAttributes` の `dwFlags` には `LWA_ALPHA` を使用し、`LWA_COLORKEY` は使用しない

---

### 6.9 display_config.rs　─ ディスプレイ別設定

**責務:** ディスプレイ1枚分の設定値の保持と、再接続時の設定引き継ぎロジック。

```rust
/// プラットフォーム固有のモニター識別子をラップした型
/// macOS: CGDirectDisplayID (u32)
/// Windows: HMONITOR (isize)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MonitorId(pub u64);

impl MonitorId {
    /// MonitorHandle からプラットフォーム固有IDを取り出して生成する
    pub fn from_monitor(monitor: &MonitorHandle) -> Self
    pub fn to_string(&self) -> String  // トレイメニューIDへの埋め込み用
}

/// ディスプレイ1枚の設定
#[derive(Debug, Clone)]
pub struct DisplayConfig {
    pub enabled: bool,             // このディスプレイのフィルターON/OFF
    pub alpha: f32,                // フィルター濃度（0.0〜1.0）
    /// 再接続時の設定引き継ぎに使うヒューリスティックキー
    /// 値: "{x}_{y}_{width}_{height}" 形式の文字列
    pub position_key: String,
}

impl DisplayConfig {
    pub fn default() -> Self        // enabled=true, alpha=0.30
    pub fn alpha_u8(&self) -> u8   // (alpha * 255.0).round() as u8

    /// 位置・解像度から position_key を生成する
    pub fn make_position_key(pos: PhysicalPosition<i32>, size: PhysicalSize<u32>) -> String
}

/// 切断済みディスプレイの設定キャッシュ（再接続時の引き継ぎ用）
/// キー: position_key、値: DisplayConfig
pub struct DisconnectedCache {
    cache: HashMap<String, DisplayConfig>,
}

impl DisconnectedCache {
    pub fn new() -> Self

    /// 切断時に設定を保存する
    pub fn store(&mut self, config: DisplayConfig)

    /// 再接続時に position_key でマッチする設定を取り出す（取り出したらキャッシュから削除）
    pub fn restore(&mut self, position_key: &str) -> Option<DisplayConfig>
}
```

**実装方針:**
- `MonitorId` は `Hash + Eq` を実装し `HashMap` のキーとして使えるようにする
- 再接続時の設定引き継ぎは「同一の `position_key`」を条件とする。同じ座標・解像度で再接続した場合のみ引き継ぐ。それ以外はデフォルト値を適用する
- `DisconnectedCache` のエントリ数は最大 8 件に制限し、古いものから削除するLRU方式とする

---

### 6.10 platform/macos.rs　─ ホットプラグ検知（macOS）

**追加責務:** `NSApplicationDidChangeScreenParametersNotification` を監視してディスプレイ変化を検知する。

```rust
/// NSNotificationCenter に画面変更通知のオブザーバーを登録する。
/// 変化が発生したら tx にイベントを送信する。
/// 戻り値の Guard を drop するとオブザーバーが解除される。
pub fn register_hotplug_observer(tx: mpsc::Sender<DisplayChangeEvent>) -> HotplugGuard

pub enum DisplayChangeEvent {
    /// 現在接続されているモニター一覧が変化した（追加・削除どちらの場合も同じイベント）
    ScreenParametersChanged,
}
```

**実装方針:**
- `NSNotificationCenter.defaultCenter()` に `NSApplicationDidChangeScreenParametersNotification` を登録する
- 通知を受けたら `tx.send(DisplayChangeEvent::ScreenParametersChanged)` を呼ぶ
- `main.rs` のイベントループで `rx.try_recv()` し、受信時に `event_loop.available_monitors()` と現在の `overlays` の差分を計算して追加/削除を行う

---

### 6.11 platform/windows.rs　─ ホットプラグ検知（Windows）

**追加責務:** `WM_DISPLAYCHANGE` メッセージを受信してディスプレイ変化を検知する。

```rust
/// tao の WindowEvent::Occluded 等ではなく WM_DISPLAYCHANGE を直接処理するため、
/// サブクラス化（SetWindowSubclass）またはメッセージフックを設定する。
/// 変化が発生したら tx にイベントを送信する。
pub fn register_display_change_hook(
    hwnd: HWND,
    tx: mpsc::Sender<DisplayChangeEvent>,
) -> HotplugGuard

pub enum DisplayChangeEvent {
    DisplayChanged,
}
```

**実装方針:**
- `tao` は `WM_DISPLAYCHANGE` を直接 `Event` として露出しないため、`SetWindowSubclass` でウィンドウプロシージャをサブクラス化して受信する
- macOS と同様、受信時に `event_loop.available_monitors()` と現在の `overlays` の差分を計算する
- サブクラス化対象は最初のオーバーレイウィンドウの HWND を使用する

---

## 7. システム設計・アーキテクチャ

### 7.1 コンポーネント構成

```
┌──────────────────────────────────────────────────────────────────┐
│                       main.rs                                    │
│  初期化オーケストレーション / tao イベントループ                  │
│                                                                  │
│  ┌─────────────────┐  ┌──────────────────────┐  ┌────────────┐  │
│  │ single_instance │  │       app.rs         │  │ hotkey.rs  │  │
│  │ 多重起動防止    │  │  AppState            │  │ HotKey 監視│  │
│  └─────────────────┘  │  ├ global_enabled    │  └─────┬──────┘  │
│                        │  └ displays: HashMap │        │ mpsc    │
│  ┌─────────────────┐  │    MonitorId →       │  ┌─────▼──────┐  │
│  │display_config.rs│◀─┤    DisplayConfig     │  │ (チャネル) │  │
│  │ MonitorId       │  └──────────┬───────────┘  └─────┬──────┘  │
│  │ DisplayConfig   │             │ sync_all()          │         │
│  │ DisconnectedCache│  ┌─────────▼─────────────────────▼──────┐  │
│  └─────────────────┘  │          overlay.rs                   │  │
│                        │   OverlayWindow × N 枚               │  │
│                        │   monitor_id / softbuffer 描画        │  │
│                        │   add_display / remove_display        │  │
│                        └───────────────────────────────────────┘  │
│                                                                  │
│  ┌──────────────────────────────────────────────────────────────┐ │
│  │          tray.rs                                             │ │
│  │   TrayHandle（tray-icon + muda）                            │ │
│  │   ・グローバルトグル  ・ディスプレイ別サブメニュー           │ │
│  │   ・rebuild_menu() でホットプラグ時に再構築                  │ │
│  └──────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────┬────────────────────────────────┘
                                  │  OS ネイティブ API
                   ┌──────────────┴──────────────┐
          ┌────────▼────────┐          ┌──────────▼──────────┐
          │ platform/       │          │ platform/           │
          │ macos.rs        │          │ windows.rs          │
          │ NSWindow 操作   │          │ HWND 操作           │
          │ ScreenParams    │          │ WM_DISPLAYCHANGE    │
          │ Notification    │          │ サブクラス化        │
          └─────────────────┘          └─────────────────────┘
```

### 7.2 起動フロー

```
.app / .exe ダブルクリック
        │
        ▼
single_instance::acquire()
        │ Err → 既に起動中 → process::exit(0)
        │ Ok
        ▼
EventLoop::new()  +  mpsc チャネル × 2 生成
  (hotkey_tx/rx, display_change_tx/rx)
        │
        ▼
hotkey::register(hotkey_tx)         ← 別スレッドでキー監視開始
        │
        ▼
overlay::create_all(monitors)
        │ 各ディスプレイに OverlayWindow を生成
        │ platform::apply_overlay_settings() で透過・最前面を設定
        │ draw(alpha) でフィルターを初期描画
        │ AppState に各 MonitorId を登録
        │
        ▼
platform::register_hotplug_observer(display_change_tx)
        │                           ← ディスプレイ変化の通知を登録
        ▼
tray::TrayHandle::new(&state, &overlays)
        │
        ▼
event_loop.run(|event, elwt, flow| {

    // ① トレイ・メニューイベント処理
    MenuEvent::receiver().try_recv() {
        MENU_ID_GLOBAL_TOGGLE  → state.toggle_global() → overlay::sync_all()
                                 → tray.update_global_check()
        MENU_ID_DISPLAY_TOGGLE → id をパース → state.toggle_display(id)
                                 → overlay 該当ウィンドウ.set_visible()
                                 → tray.update_display_check(id)
        MENU_ID_QUIT           → 終了シーケンスへ
    }

    // ② ホットキーイベント処理
    hotkey_rx.try_recv() → MENU_ID_GLOBAL_TOGGLE と同じ処理

    // ③ ホットプラグイベント処理
    display_change_rx.try_recv() {
        DisplayChangeEvent → 現在の available_monitors() を取得
                           → overlays にない ID → add_display()
                                               → DisconnectedCache.restore() or default
                                               → AppState.add_display()
                           → available_monitors() にない ID → remove_display()
                                                            → AppState.remove_display()
                                                            → DisconnectedCache.store()
                           → tray.rebuild_menu(&state, &overlays)
    }

    // ④ 終了処理
    overlays drop → tray drop → _single_instance_guard drop → flow.set_exit()
})
```

### 7.3 スレッド構成

```
┌─────────────────────────────────────┐
│  メインスレッド                      │
│  tao イベントループ                   │
│  ・ウィンドウ生成・描画               │
│  ・トレイイベント処理                 │
│  ・ホットキー受信（hotkey_rx）        │
│  ・ホットプラグ受信（display_rx）     │
└──────┬──────────────────────┬────────┘
       │ mpsc::Sender          │ mpsc::Sender
       │ <HotkeyEvent>         │ <DisplayChangeEvent>
┌──────▼──────────┐   ┌────────▼────────────────┐
│ ホットキー       │   │ ホットプラグ監視スレッド  │
│ 監視スレッド    │   │ macOS: NSNotification    │
│ GlobalHotKey    │   │ Windows: SetWindowSubclass│
│ receiver()      │   │ → tx.send()              │
└─────────────────┘   └──────────────────────────┘
```

### 7.4 macOS 固有実装詳細

1. `tao` でボーダーレス・透過ウィンドウを生成する
2. `raw-window-handle` から `NSWindow` のポインタを取得する
3. `ignoringMouseEvents = true` でクリックを透過させる
4. `setBackgroundColor(NSColor.clear)` でウィンドウ背景を透明にする
5. `setLevel(NSStatusWindowLevel + 1)` で最前面に固定する
6. `collectionBehavior = .canJoinAllSpaces | .stationary` で全 Space に表示させる
7. `softbuffer` のフレームバッファに ARGB `0x4C000000` を塗りつぶして描画する
8. `NSApplicationDidChangeScreenParametersNotification` を `NSNotificationCenter` に登録してホットプラグを検知する

> **注意:** `NSStatusWindowLevel` より上のレベルに設定すると、フルスクリーンアプリに隠れる場合がある（→ 未解決事項 #3）

### 7.5 Windows 固有実装詳細

1. `tao` でボーダーレス・透過ウィンドウを生成する
2. `raw-window-handle` から `HWND` を取得する
3. `GetWindowLongPtrW(GWL_EXSTYLE)` で現在のスタイルを取得する
4. `SetWindowLongPtrW` で `WS_EX_TRANSPARENT | WS_EX_LAYERED | WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW` を付与する
5. `SetLayeredWindowAttributes(hwnd, 0, alpha, LWA_ALPHA)` で透明度を設定する
6. `SetWindowPos(hwnd, HWND_TOPMOST, ...)` で最前面に固定する
7. `SetWindowSubclass` で最初のオーバーレイウィンドウのプロシージャをサブクラス化し、`WM_DISPLAYCHANGE` を受信してホットプラグを検知する

> **注意:** `WS_EX_NOACTIVATE` を付与することで、フィルターをクリックしてもフォーカスが奪われなくなる（→ 未解決事項 #6）

---

## 8. 非機能要件

| 要件 | 目標値 |
|------|-------|
| 起動時間 | 2秒以内（フィルター表示開始まで） |
| CPU使用率 | アイドル時 1% 以下（静的フィルターのため再描画なし） |
| メモリ使用量 | 50MB 以下 |
| バイナリサイズ | 10MB 以下（リリースビルド、`strip = true`） |

---

## 9. 未解決事項 / Known Issues

| # | 状態 | 項目 | 内容 | 優先度 |
|---|------|------|------|-------|
| 1 | ✅ 解決済 | ~~ウィンドウライブラリ選定~~ | → **`tao` に決定** | ― |
| 2 | 🔲 未着手 | macOS アクセシビリティ権限 | グローバルショートカットの初回起動時に表示されるダイアログの案内文言を決定する | 中 |
| 3 | 🔲 未着手 | macOS フルスクリーン対応 | `NSStatusWindowLevel + 1` でフルスクリーンアプリの上にも表示できるか検証が必要 | 中 |
| 4 | 🔲 未着手 | 配布形式（macOS） | `.app` バンドル直配布か `.dmg` か。公証なしだと Gatekeeper 警告が出る | 中 |
| 5 | 🔲 未着手 | 配布形式（Windows） | `.exe` 単体か MSI インストーラーか | 中 |
| 6 | 🔲 未着手 | Windows UAC との兼用 | 管理者権限ウィンドウの前面に出せるか | 低 |
| 7 | 🔲 未着手 | コード署名 / 公証 | 配布を想定するなら Apple / Microsoft の署名が必要 | MVP外 |

> **運用ルール**: 実装中に発生した問題・判断はこのテーブルに追記し、DEVLOG.md に詳細を記録する。解決済みの項目は状態を ✅ に更新し、対応 commit / DEVLOG エントリへのリンクを備考に追記する。

---

## 10. 今後の拡張プラン

### 10.1 Ver 2.0

- 濃度スライダー UI（`AppState.alpha` を活用）
- 縦縞ルーバーパターン描画（`tiny-skia` 追加）
- プリセット保存（`confy` クレート等で設定ファイル永続化）
- ログイン時自動起動（macOS: `LaunchAgent`、Windows: レジストリ）

### 10.2 Ver 3.0

- 内蔵カメラで背後の顔を検知したら自動でフィルターを濃くする AI 覗き見検知
- カメラ映像はローカル処理のみ（`tract` クレート + ONNX モデル）

---

## 11. 開発ロードマップ

```
Phase 0 (プロトタイプ)
 └─ macOS で半透明・クリック透過のフルスクリーンウィンドウを表示

Phase 1 (MVP)
 ├─ .app / .exe ダブルクリック起動
 ├─ メニューバー（macOS）/ システムトレイ（Windows）常駐
 ├─ コンテキストメニューでのフィルター ON/OFF・終了
 ├─ グローバルショートカットによる ON/OFF
 └─ Windows クロスコンパイル対応・マルチディスプレイ対応

Phase 2
 ├─ 濃度スライダー UI
 ├─ プリセット保存
 └─ ログイン時自動起動オプション

Phase 3
 └─ AI 覗き見検知
```

---

## 付録：用語集

| 用語 | 説明 |
|------|------|
| ルーバー | 細い羽板を等間隔に並べたブラインド状のフィルター。正面からは見えるが斜めからは遮光される |
| クリック透過 | ウィンドウがマウスイベントをキャプチャせず、背後のウィンドウへ伝達する動作 |
| HWND | Windows のウィンドウハンドル。Win32 API でウィンドウを識別する整数値 |
| HMONITOR | Windows のモニターハンドル。接続ディスプレイを識別する整数値 |
| NSWindow | macOS の Cocoa フレームワークにおけるウィンドウクラス |
| CGDirectDisplayID | macOS のディスプレイ識別子（`u32`）。接続・切断をまたいで同一性が保証される |
| MonitorId | 本アプリ内でプラットフォーム固有のモニター識別子を統一的に扱うラッパー型 |
| ホットプラグ | PCの稼働中にディスプレイを接続・切断する操作 |
| position_key | ディスプレイの位置と解像度から生成した文字列キー。切断後の再接続時に設定を引き継ぐために使用する |
| DisconnectedCache | 切断済みディスプレイの設定を一時保存するキャッシュ。再接続時の設定復元に使用する |
| WS_EX_TRANSPARENT | Windows 拡張ウィンドウスタイル。マウスイベントを透過させる |
| WS_EX_LAYERED | Windows 拡張ウィンドウスタイル。半透明・色キー合成を有効にする |
| WS_EX_TOOLWINDOW | Windows 拡張ウィンドウスタイル。タスクバーへのボタン表示を抑制する |
| WS_EX_NOACTIVATE | Windows 拡張ウィンドウスタイル。クリック時にフォーカスを奪わない |
| WM_DISPLAYCHANGE | Windows メッセージ。ディスプレイの接続・解像度変更時にウィンドウプロシージャへ送られる |
| HWND_TOPMOST | `SetWindowPos` に渡すフラグ。常時最前面表示を指定する |
| LSUIElement | macOS の `Info.plist` キー。`true` にすると Dock およびアプリスイッチャーへの表示を抑制する |
| collectionBehavior | macOS のウィンドウが複数の Space / フルスクリーンにどう振る舞うかを制御するプロパティ |
| NSApplicationDidChangeScreenParametersNotification | macOS のディスプレイ構成変化を通知する NSNotification 名 |
| システムトレイ | Windows の通知領域（タスクバー右端）。常駐アプリのアイコンを置く場所 |
| メニューバー | macOS 画面上端のバー。常駐アプリはここにアイコンを置く |
| LWA_ALPHA | `SetLayeredWindowAttributes` のフラグ。Alpha 値でウィンドウ全体の透明度を指定する |

---

## 12. 設計哲学

- **ゼロ UI 哲学**: ユーザーはフィルターが「そこにいる」ことを意識しない。常駐するが邪魔しない
- **フラットな構造**: モジュールの深い入れ子を避ける。`src/` 直下＋ `platform/` の2階層に収める
- **プラットフォーム差分の局所化**: OS固有コードは `platform/macos.rs` / `platform/windows.rs` のみに閉じ込め、呼び出し元は共通インターフェース経由でのみアクセスする
- **パニック時の後始末**: `std::panic::set_hook` でトレイアイコンの残存を防ぐ
- **再描画しない**: フィルターは静的な単色レイヤーであり、アイドル時の再描画は一切行わない。CPU使用率 1% 以下を維持する
- **クリーンルーム実装**: コードはすべてオリジナル。ユーザーが唯一の著作権者となる

---

## 13. 非設計事項（将来も実装しない）

- ウィンドウ単位のフィルター（アプリ全体のオーバーレイのみ）
- 音声・動画の録画抑止（OS レベルの DRM と競合するため）
- プラグインシステム
- ネットワーク通信・テレメトリ