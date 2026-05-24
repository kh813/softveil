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
- [x] **サブピクセル単位の描画制御 (Subpixel Rendering)** 🔁 ✅
    - [x] シェーダー内での `(x * 3.0)` 座標系による RGB 独立制御の実装 ✅
    - [x] 0.5px 相当の極細ライン・パターンの生成 ✅
- [x] **サブピクセル・コントラスト崩壊アルゴリズム** 🔁 ✅
    - [x] 斜め视野角での色偏移（Color Shift）を増幅させるサブピクセル・ディザリング ✅
    - [x] font-weight 200 相当の細いフォントを物理的に消失させる MTF 破壊パターン ✅
- [x] **UI への統合** 🔁 ✅
    - [x] フィルター形式に「Stealth Light (Subpixel)」を追加 ✅
    - [x] OS のライトモード設定との自動連動オプションの実装 ✅

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
- [x] **Stealth Light (Subpixel) の評価対応** (New) ✅

---

## Phase 10：動的最適化と次世代評価指標 🧪 ✅

**目的:** 実機ベンチマークに基づく自動パラメータフィッティングと、AIレベルの秘匿性実証。

- [ ] **OCR Jamming Metrics**: ベンチマークへの Tesseract 連携による認識成功率の計測 🚧
- [x] **Benchmark-Driven Auto-Optimization**: 実機評価に基づき、Obfuscation Index が最大となるパラメータの自動選定 ✅
- [ ] **Heuristic Recommended (Existing)** の精度向上と、動的最適化への統合 🚧

---

## Phase 11：プリセット管理システム 🧪 ✅

**目的:** ベンチマーク結果に基づく最適設定の保存、およびユーザーによる設定プリセットの管理機能の統合。

- [x] **プリセット用データ構造の実装 (`FilterSettings`)** 🔁 ✅
- [x] **プリセット管理ロジックの実装 (`AppState`)** 🔁 ✅
    - [x] ユーザー定義プリセットの保存・適用（個別/一括）・削除機能 ✅
    - [x] ベンチマーク結果からの「推奨プリセット」の自動生成（マルチモニター対応） ✅
- [x] **UI への統合 (`tray.rs`)** 🔁 ✅
    - [x] トレイメニューへの「設定プリセット」階層型サブメニューの追加 ✅
    - [x] モニター別適用および全画面一括適用の実装 ✅
- [x] **ベンチマーク完了時の自動通知・プリセット提案** 🔁 ✅
- [x] ベンチマーク中の視覚的フィードバックの実装 🔁 ✅
    - [x] 進行状況（プログレス）を通知する仕組みの導入 ✅
    - [x] トレイアイコンのツールチップおよびメニューへの進捗（%）表示 ✅
    - [x] macOS: 通知センターによる進捗通知の追加 ✅
    - [x] macOS: 実行前の厳格な権限チェックを緩和（実キャプチャ失敗時のみ警告） ✅
- [x] **ベンチマーク結果の詳細表示** 🔁 ✅
    - [x] 測定スコアや最適化結果をまとめたダイアログの表示 ✅
- [x] **ベンチマークの高速化（バッチ並列化）** 🔁 ✅
    - [x] 全モニター同時設定・同時待機による「モニター数に依存しない」測定時間の実現 ✅
    - [x] 画像解析処理のマルチスレッド並列化 ✅
- [x] **ユーザーマニュアルの更新 (`MANUAL.md`)** 🔁 ✅

---

## Phase 12：アルゴリズム・パラメーター改善 v2.0 🧪 ✅

**目的:** 実機フィードバックに基づく視覚的品質の向上と、シェーダーロジックの不備修正。

- [x] **ディスプレイパラメータの適正化 (display_config.rs)** ✅
    - [x] `NotebookFhd` / `ExternalGeneral` の `period_mm` を拡大し縞パターンを維持 ✅
    - [x] `default_intensity` を 1.0 に変更し、標準状態で縞が見えるように改善 ✅
- [x] **シェーダーロジックの修正・向上 (shader.wgsl)** ✅
    - [x] `StealthLightSubpixel` (mode 6) の色差オフセット計算と RGB 出力の修正 ✅
    - [x] `StealthLight` (mode 5) の未使用変数 `sub_x` の削除 ✅
    - [x] `HighIntensitySPD` (mode 3) LCD のスクロール速度が設定に依存するよう修正 ✅
    - [x] `AIOcrInterference` (mode 2) のノイズ密度と輝度を向上 ✅
    - [x] `StealthDark` (mode 4) LCD の `glow` 強度をカテゴリ依存に調整 ✅
- [x] **ベンチマーク機能の信頼性向上とデッドロック解消** 🔁 ✅
    - [x] イベントループの `MainEventsCleared` ハンドラを統合し、アニメーション中もコマンドを処理 ✅
    - [x] macOS: `screencapture` CLI から `CoreGraphics` API (in-memory) へ移行し高速化 ✅
    - [x] Windows: `BitBlt` に `CAPTUREBLT` フラグを追加しオーバーレイを確実に捕捉 ✅
    - [x] macOS: バンドル実行時の書き込み権限エラーによるクラッシュを修正 ✅
    - [x] macOS: 不要な権限警告ダイアログの抑制（キャプチャ失敗時かつ権限不足時のみ表示） ✅
- [x] **macOS 安定性の更なる改善と診断機能 (New)** 🔁 ✅
    - [x] macOS: 権限判定ロジックの抜本的修正（既に許可されている場合の誤判定を修正） ✅
    - [x] macOS: ベンチマーク完了後のクラッシュ調査と修正 ✅
    - [x] 🔁 共通: プラットフォーム固有の一時ディレクトリへのログ出力機能の実装 ✅
        - macOS: `/tmp/softveil.log`
        - Windows: `%TEMP%\softveil.log`
- [x] **ベンチマークの安定性向上と診断機能の強化 (New)** 🔁 ✅
    - [x] macOS: 権限判定ロジックの抜本的修正（既許可時のダイアログ抑制を確実に） ✅
    - [x] macOS: ベンチマーク完了後のクラッシュ調査と修正 ✅
    - [x] 🔁 共通: プラットフォーム固有の一時ディレクトリへのログ出力機能の実装 ✅
        - macOS: `/tmp/softveil.log`
        - Windows: `C:\temp\softveil.log` (または適切な一時パス)
- [x] **変更指示書 v2.0 に基づくパラメーター微調整 (New)** 🔁 ✅
    - [x] MacBook Air (HiDpi): `period_mm` / `cover_ratio` 調整による縞の視認性向上 ✅
    - [x] FHD Notebook: `scroll_speed` 有効化による OCR 対策とクロスルーバー化 ✅
    - [x] OLED (MacBook Pro): `alpha_base` 調整による正面視認性の改善 ✅
- [x] **次世代 LCD 秘匿技術「NarrowMask」の実装 (New)** 🔁 ✅
    - [x] `shader.wgsl`: 1px 窓を残して完全黒 (alpha=1.0) で塞ぐ NarrowMask ロジックの実装 ✅
    - [x] `display_config.rs`: MacBook Air / FHD 向けの `cover_ratio` 最適化 (0.75 - 0.80) ✅
    - [x] ドキュメント更新 (仕様書, マニュアル, DEVLOG) ✅
- [x] **macOS 固有バグの修正 (New)** 🍎 ✅
    - [x] 🍎 macOS: 「終了」メニューが機能しない問題を修正（イベントハンドラ追加） ✅
    - [x] 🍎 macOS: 権限許可済みでも警告が出る偽陰性問題を修正（実キャプチャによる確定判定） ✅
    - [x] 🍎 macOS: ベンチマーク中にメニューが消える問題を修正（進捗時のメニュー再構築を抑制） ✅
    - [x] 🍎 macOS: ステルスダークでダークモードに切り替わらない問題を修正（AppleScript 同期処理の見直し） ✅

