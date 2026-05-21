# Softveil 実装 ToDo リスト

**対応仕様書:** Softveil 基本仕様書 v1.6
**対象フェーズ:** Phase 4（Windows 完備・正式ビルド）

凡例: 🍎 = macOS 固有　🪟 = Windows 固有　🔁 = 両 OS 共通

---

## Phase 0：プロトタイプ (macOS) ✅
- [x] プロジェクト初期化
- [x] 基本データ構造 (`MonitorId`, `DisplayConfig`, `AppState`) の実装
- [x] macOS 固有のオーバーレイ設定実装

---

## Phase 1：MVP (macOS) ✅
- [x] 単一インスタンス制御
- [x] ホットプラグ検知（macOS）
- [x] トレイメニュー UI
- [x] グローバルショートカット (`Cmd+Shift+P`)

---

## Phase 2：Ver 2.0 拡張 ✅
- [x] 設定の永続化 (`confy`)
- [x] 濃度変更 UI (10% - 90%)
- [x] ログイン時自動起動 (`auto-launch`)
- [x] 縦縞ルーバーパターンの実装 (`tiny-skia`)

---

## Phase 3：AI 覗き見検知 ✅
- [x] `tract-onnx` によるローカル推論の実装
- [x] `nokhwa` によるカメラアクセス実装
- [x] 複数人数検知時の自動濃度強化ロジック

---
## Phase 4：Windows 完備・正式ビルド 🪟🔁 ✅

### STEP 4-1　Windows ホットプラグの実装 🪟 ✅
- [x] `src/platform/windows.rs` の `register_display_change_hook` を実装
- [x] Windows 実機でディスプレイ抜き差し時の追従を確認

### STEP 4-2　アイコン素材の正式生成と組み込み 🔁 ✅
- [x] `assets/softveil_icon.svg` からマルチサイズ PNG を生成するスクリプトを作成
- [x] 🍎 macOS: `assets/icon_macos.icns` を生成
- [x] 🪟 Windows: `assets/icon_windows.ico` を生成
- [x] `src/tray.rs` を更新: 埋め込みダミー画像から `include_bytes!` で正式アイコン読み込みへ変更
- [x] 🪟 `build.rs` を更新: `winres` を使用して `.exe` にアイコンを埋め込む

### STEP 4-3　GitHub Actions (CI) の構築 🔁 ✅
- [x] `.github/workflows/release.yml` を作成
- [x] macOS (Universal Binary) および Windows (x64) のビルドジョブを設定
- [x] ビルド成果物（`.app`, `.exe`）の圧縮とリリースアセットへのアップロード自動化
- [x] ONNX モデルファイルをバイナリに直接埋め込む仕組みへ移行

### STEP 4-4　Windows 固有の動作最適化と検証 🪟 ✅
- [x] DOS窓（コンソール）の非表示化 (`windows_subsystem`)
- [x] AIモデル (`.onnx`) のバイナリ埋め込みによるスタンドアロン化
- [x] トレイアイコンの右クリックメニューの挙動確認（Windows 標準準拠）
- [x] スマートスクリーン警告（非署名）に関する運用確認
- [x] macOS 上での Windows クロスコンパイル環境構築 (`mingw-w64`) ✅

### STEP 4-5　最終調整とリリース準備 🔁 ✅
- [x] 🍎 macOS: フルスクリーンアプリ上での表示安定化（既知のチラつきは Phase 5 以降で改善）
- [x] 🍎 macOS: ショートカット (`Cmd+Shift+P`) の動作確認
- [x] 🍎 macOS: `system_profiler` を使用したディスプレイ詳細名（型番等）の取得実装 ✅
- [x] `MANUAL.md` の最終推敲

### STEP 4-6　UI 設計の刷新：ディスプレイ別詳細設定 🔁 ✅
- [x] `DisplayConfig` に `filter_mode` を追加し、`AppState` からグローバル設定を削除 ✅
- [x] トレイメニューの階層構造を刷新（各ディスプレイ ▶ 有効/形式/濃度） ✅
- [x] ホットプラグ（接続・切断）検知時のメニュー自動更新を実装 ✅
- [x] 既存の設定ファイルの移行（互換性維持） ✅
- [x] マルチディスプレイ環境での各画面独立制御の検証 ✅

---


## フェーズ完了ログ

### ✅ Phase 0 Completion Log
...
### ✅ Phase 3 Completion Log
- **完了日**: 2026-05-15
- **作業者**: Gemini CLI
- **主な決定事項**: AI 覗き見検知の実装 (Tract + Nokhwa)

### ✅ Phase 4 Completion Log
- **完了日**: 2026-05-15
- **作業者**: Gemini CLI
- **主な決定事項**: Windows 完全対応、CI/CD 構築、AI モデルのバイナリ埋め込み、macOS フルスクリーン最適化

### ✅ Phase 1 Completion Log
- **完了日**: 2026-05-15
- **作業者**: Gemini CLI
- **主な決定事項**: MVP 機能の実装完了

### ✅ Phase 2 Completion Log
- **完了日**: 2026-05-15
- **作業者**: Gemini CLI
- **主な決定事項**: 永続化・ルーバーパターン実装

### ✅ Phase 3 Completion Log
- **完了日**: 2026-05-15
- **作業者**: Gemini CLI
- **主な決定事項**: AI 覗き見検知の実装 (Tract + Nokhwa)

---

## Phase 5：ソフトウェア定義型プライバシーディスプレイ (SPD) 🧪

**目的:** 物理特性（ガンマ偏移）のハックや視線追跡、ユースケース別の最適化による次世代秘匿技術。
**ステータス:** 実験的フェーズ（Phase 4 完了後に別ブランチで着手）

- [x] **GPU レンダリングエンジンへの刷新 (`wgpu`)** ✅
    - [x] `tiny-skia` から `wgpu` への移行による低負荷・高 FPS レンダリング ✅
    - [x] WGSL シェーダーによるピクセルパーフェクトな制御 ✅
- [x] **GPU シェーダーによる輝度干渉 (Luminance Interference) / LCD コントラスト妨害 (NEW)** ✅
    - [x] 🔁 共通: LCD IPS 向け視野角コントラスト崩壊型フィルターの実装 ✅
    - [x] 🔁 共通: 輝度圧縮グリッド、逆コントラストハッチ、動的位相ノイズの合成 ✅
    - [x] 🔁 共通: パネル種別「LCD IPS」時の推奨モード自動切り替え ✅
- [x] **ディスプレイサイズ・解像度適応型フィルター最適化 (NEW)** ✅
    - [x] 🔁 共通: 物理サイズ基準 (mm) のパラメータ定義と PPI 計算ロジック ✅
    - [x] 🍎 macOS: `CGDisplayIsBuiltin` / `CGDisplayScreenSize` による物理情報取得 ✅
    - [x] 🪟 Windows: `GetDeviceCaps` / `EnumDisplayDevices` による物理情報取得 ✅
    - [x] 🔁 共通: ディスプレイカテゴリ (Notebook/External) の自動判定 ✅
    - [x] 🔁 共通: カテゴリ別の推奨パラメータプロファイルの実装 ✅
    - [x] 🔁 共通: トレイメニューへの表示と手動変更機能の実装 ✅
    - [x] 🍎 macOS: HiDPI スケーリング環境での 4K 判定ロジックの改善 (v2.2) ✅
- [x] **高速動体マスキングの実装 (US9058509 手法)** ✅
    - [x] 60fps+ でのプライバシーパターンの高速振動・移動ロジック ✅
    - [x] フリッカー融合を利用した「正面からの透明性」と「斜めからの遮蔽」の両立 ✅
- [x] **非対称曲線パターンによる秘匿 (US10496831 手法)** ✅
    - [x] 斜めからの視認性を数学的に破壊するシームレスパターンの生成 ✅
- [ ] **高度な画面加工アルゴリズム (`crabgrab` / `ScreenCaptureKit`)** 🚧
    - [x] macOS/Windows での画面収録アクセス権限の要求・取得 ✅
    - [ ] **Semantic Privacy**: ウィンドウ内容のセマンティクス解析と特定領域（パスワード欄等）の重点保護 🚧
    - [x] **AI OCR 妨害**: スクリーンショットやカメラ撮影による文字認識を困難にする特殊テクスチャの重畳 ✅
- [ ] **視線追跡 (Gaze Tracking) との統合** 🚧
    - [ ] 顔検知データと連携したダイナミック・プライバシー・コーンの形成 🚧
- [x] **パネル種別 (LCD/OLED) の自動判別と適応型戦略 (NEW)** ✅
    - [x] 🪟 Windows: `EnumDisplayDevices` & EDID レジストリ解析の実装 ✅
    - [x] 🍎 macOS: `system_profiler` からのモデル名抽出とキーワードマッチの統合 ✅
    - [x] 🍎 macOS: `NSScreen` EDR 輝度による OLED/HDR 判定の統合 ✅
    - [x] 🔁 共通: キーワード & スコアリングによるパネル種別判定ロジック ✅
    - [x] 🔁 共通: LCD 向けアニメーション（垂直スクロール・位相反転）の WGSL 実装 ✅
    - [x] 🔁 共通: パネル種別に応じたフィルタプロファイルの自動適用 ✅
    - [x] 🔁 共通: UI での判定結果表示と手動変更機能の実装 ✅
- [x] **パネル種別 (LCD/OLED) の基本対応** ✅
    - [x] OLED 向けの焼き付き防止 (Anti-Burn-in) 振動パターンの実装 ✅
    - [x] **OLED 向け高密度ルーバー (High-Density Louver) の実装** ✅
- [ ] **ユースケース別最適化 (Context-Aware Optimization)** 🚧
    - [ ] **モバイルモード**: バッテリー残量に連動した人物検知サンプリングレートの自動調整 🚧
    - [ ] **プロフェッショナルモード**: 据置利用時の高負荷・高品質な輝度干渉パターンの適用 🚧
- [ ] **GPU シェーダーによる輝度干渉 (Luminance Interference)** 🚧
    - [ ] 液晶のガンマ偏移を利用した、斜め方向からのコントラスト動的破壊 🚧
    - [ ] **Dynamic Gamma Shifting**: コンテンツ解析に基づく逆位相ガンマ補正の実装 🚧
- [ ] **次世代アドバンスド SPD 技術の検証 (Expansion)** 🚧
    - [ ] **Subpixel UHD Jamming**: R/G/B サブピクセル単位の干渉パターンの実装 🚧
    - [ ] **Chromatic Aberration Simulation**: 意図的な色ズレによる斜め方向の色偏移増幅 🚧
    - [ ] **Semantic Clarity Reduction**: 特定領域（文字・フォーム）の重点ガード 🚧

## Phase 6：統合型ステルス・スイッチの実装 🧪 ✅

**目的:** フィルタ形式の選択に連動した OS 環境制御（テーマ・輝度）と、ダーク環境専用の物理構造ハックの統合。

- [x] **OS 設定制御ブリッジの実装** ✅
    - [x] **State Snapshot**: モード変更前の OS 設定（テーマ、輝度）を一時保存するロジックの実装 ✅
    - [x] 🍎 macOS: `NSAppearance` によるテーマ切替、`CoreGraphics` による輝度操作の実装 ✅
    - [x] 🪟 Windows: レジストリ (`AppsUseLightTheme`) および WMI による輝度操作の実装 ✅
    - [x] 🔁 共通: アプリ終了時の「設定自動復元」メカニズムの構築 ✅
- [x] **ステルス専用コントラスト破壊フィルタの開発 (shader.wgsl)** ✅
    - [x] **Low-Luma Contrast Collapse (LLCC)**: 背景黒を持ち上げる「ベースグロー」重畳ロジックの実装 ✅
    - [x] **Static UHF Dithering**: 170 PPI 最適化 1x1 固定ディザパターンの実装（動きを排除） ✅
    - [x] **Narrow Aperture Enhancement (UNA)**: OLED 向け極小サブピクセル開口ロジックのブラッシュアップ ✅
- [x] **UI/UX への統合** ✅
    - [x] トレイメニューの「フィルター形式」に「Stealth Dark (LLCC)」を追加 ✅
    - [x] 自動適応: OS 側ですでにダークモードが選択されている場合、標準モードからステルス最適化フィルタへ自動移行するロジック ✅
    - [x] モード切替時の滑らかな輝度遷移（フェード）演出の実装 ✅


## Phase 7：Windows 最適化とリファクタリング (Phase 6+ Improvement) 🧪

**目的:** Windows 環境での視覚的品質の向上、GPU 負荷の低減、およびコードの堅牢性強化。

- [ ] **Windows 格子サイズと視覚的品質の改善** 🪟
    - [x] `scale_factor` (DPR) をシェーダーの物理パラメータ計算に正しく反映 ✅
    - [x] Windows FHD ノート向けの `period_mm` デフォルト値を 0.20mm に調整 ✅
    - [x] Windows FHD ノートのデフォルトスクロール速度を 0.0mm/s（静止）に変更 ✅
    - [x] Windows FHD ノートでは `bidirectional` (格子状) をデフォルト OFF（縦縞のみ）に設定 ✅
- [ ] **GPU 負荷の低減 (Intel N200 等の省電力 GPU 対応)** 🔁
    - [x] wgpu サーフェス設定を `PresentMode::Fifo` (VSync 有効) に変更 ✅
    - [x] フィルター OFF かつアニメーション不要時に `ControlFlow::Wait` へ移行し描画を完全停止 ✅
    - [x] フレームリミッターの実装 (静止/低速時は 30fps 上限) ✅
    - [x] シェーダー内の重い演算（三角関数等）の最適化・近似 ✅
    - [x] Windows での wgpu バックエンドを DX11/DX12 に優先順位付け ✅
- [ ] **Windows システム統合の高度化** 🪟
    - [x] PowerShell 呼び出し時のコンソール瞬間表示の抑止 (`CREATE_NO_WINDOW`) ✅
    - [x] PowerShell 依存を排除し、`windows-rs` による WMI/COM 直接呼び出しへの移行 ✅
    - [x] レジストリ変更後の設定反映を `SendMessageTimeout` によるブロードキャストに変更 ✅
    - [x] マニフェストによる Per-Monitor DPI Aware v2 の明示的な宣言 ✅
    - [ ] 内蔵ディスプレイ判定に `DEVPKEY_Device_LocationInfo` (ACPIバス) 等のより確実な手法を導入 🚧
- [ ] **コードの堅牢性と保守性の向上** 🔁
    - [x] `Uniforms` 構造体のアライメントとサイズのコンパイル時検証 (`static_assertions`) ✅
    - [x] `confy` (serde) の `#[serde(default)]` 付与による設定ファイルの下位互換性確保 ✅
    - [ ] `crabgrab` 等の重い依存関係の遅延初期化（Lazy Init） 🚧
    - [x] macOS: `osascript` 廃止完了（Native API & NSAppleScript 移行） ✅

---

## Phase 8：ステルス・ライト & サブピクセル・ハック 🧪 ✅

**目的:** ライトモードへの最適化と、物理的なサブピクセル構造を突いた超高精細な秘匿技術の実装。

- [x] **ステルス・ライト (Stealth Light) モードの開発** 🔁 ✅
    - [x] **HLCC (High-Luma Contrast Collapse)** アルゴリズムの WGSL 実装 ✅
    - [x] サブピクセル単位の輝度注入（RGB 独立制御）によるエッジ破壊 ✅
    - [x] 背景 #FAFAFA / 文字 #7A7A7A 相当のコントラスト目標値に基づくベールロジックの実装 ✅
    - [x] ライトモード時の「眩しさ」と「秘匿性」の動的バランス調整 ✅
    - [x] **パラメータ・チューニング**: ベンチマーク結果と実機目視に基づく `veil` 濃度と `fine_line` 周期の最適化 (New) ✅
- [ ] **サブピクセル単位の描画制御 (Subpixel Rendering)** 🔁
    - [ ] シェーダー内での `(x * 3.0)` 座標系による RGB 独立制御の実装
    - [ ] 0.5px 相当の極細ライン・パターンの生成
- [ ] **サブピクセル・コントラスト崩壊アルゴリズム** 🔁
    - [ ] 斜め视野角での色偏移（Color Shift）を増幅させるサブピクセル・ディザリング
    - [ ] font-weight 200 相当の細いフォントを物理的に消失させる MTF 破壊パターン
- [ ] **UI への統合** 🔁
    - [ ] フィルター形式に「Stealth Light (Subpixel)」を追加
    - [ ] OS のライトモード設定との自動連動オプションの実装

---

## Phase 9：機械的ベンチマークと品質評価 🧪 ✅

**目的:** 目視に頼らない客観的なプライバシー保護性能の評価。

- [x] **ベンチマークモード (`--benchmark`) の実装** ✅
    - [x] 各フィルター・各 Alpha 値での自動キャプチャループの実装 ✅
    - [x] Windows (PowerShell) / macOS (screencapture) での画面取得統合 ✅
- [x] **デジタル斜め視野シミュレーション (Digital Oblique Simulation)** ✅
    - [x] 水平圧縮 (cos 45°) と IPS Glow (黒浮き) の画像変換ロジックの実装 ✅
- [x] **秘匿性指標 (Obfuscation Index) の算出** ✅
    - [x] CRR (Contrast Reduction Ratio) の計算ロジック ✅
    - [x] Edge Diffusion (高周波ジャミング強度) の解析ロジック ✅
    - [x] `BENCHMARK_REPORT.md` への自動レポート出力 ✅

---

## Phase 10：動的最適化と次世代評価指標 🧪

**目的:** 実機ベンチマークに基づく自動パラメータフィッティングと、AIレベルの秘匿性実証。

- [ ] **OCR Jamming Metrics**: ベンチマークへの Tesseract 連携による認識成功率の計測 🚧
- [ ] **Benchmark-Driven Auto-Optimization**: 実機評価に基づき、Obfuscation Index が最大となるパラメータの自動選定 🚧
- [ ] **Heuristic Recommended (Existing)** の精度向上と、動的最適化への統合 🚧

---

