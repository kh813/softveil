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

### STEP 4-5　最終調整とリリース準備 🔁 ✅
- [x] 🍎 macOS: フルスクリーンアプリ上での表示安定化（既知のチラつきは Phase 5 以降で改善）
- [x] 🍎 macOS: ショートカット (`Cmd+Shift+P`) の動作確認
- [x] `MANUAL.md` の最終推敲

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

- [ ] **GPU レンダリングエンジンへの刷新 (`wgpu`)**
    - [ ] `tiny-skia` から `wgpu` への移行による低負荷・高 FPS レンダリング
    - [ ] WGSL シェーダーによるピクセルパーフェクトな制御
- [ ] **高速動体マスキングの実装 (US9058509 手法)**
    - [ ] 60fps+ でのプライバシーパターンの高速振動・移動ロジック
    - [ ] フリッカー融合を利用した「正面からの透明性」と「斜めからの遮蔽」の両立
- [ ] **非対称曲線パターンによる秘匿 (US10496831 手法)**
    - [ ] 斜めからの視認性を数学的に破壊するシームレスパターンの生成
    - [ ] 視野角 45 度以上・距離 23 インチ以上での判読不能テスト
- [ ] **高度な画面加工アルゴリズム (`crabgrab` / `ScreenCaptureKit`)**
    - [ ] リアルタイム画面キャプチャに基づく Semantic Blur（文字のみの攪乱）
    - [ ] 視野角依存ノイズの GPU 合成
- [ ] **視線追跡 (Gaze Tracking) との統合**
    - [ ] ユーザーの視線位置に基づいた動的な保護領域の最適化
- [ ] **パネル種別 (LCD/OLED) の自動判別と適応型戦略**
    - [ ] Windows: `QueryDisplayConfig` によるパネル技術の取得
    - [ ] macOS: HDR 輝度・色域・モデル名による OLED 推定ロジック
    - [ ] OLED 向けの焼き付き防止 (Anti-Burn-in) 振動パターンの強制
    - [ ] パネル特性に応じたパターン密度・移動速度の自動調整


---
