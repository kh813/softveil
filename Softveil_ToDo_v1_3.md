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

## Phase 4：Windows 完備・正式ビルド 🪟🔁

### STEP 4-1　Windows ホットプラグの実装 🪟
- [ ] `src/platform/windows.rs` の `register_display_change_hook` を実装
    - [ ] 隠しウィンドウまたは既存オーバーレイのサブクラス化 (`SetWindowSubclass`) により `WM_DISPLAYCHANGE` を捕捉
    - [ ] `display_change_tx` 経由でメインループへ通知
- [ ] Windows 実機でディスプレイ抜き差し時の追従を確認

### STEP 4-2　アイコン素材の正式生成と組み込み 🔁 ✅
- [x] `assets/softveil_icon.svg` からマルチサイズ PNG を生成するスクリプトを作成
- [x] 🍎 macOS: `assets/icon_macos.icns` を生成
- [x] 🪟 Windows: `assets/icon_windows.ico` を生成
- [x] `src/tray.rs` を更新: 埋め込みダミー画像から `include_bytes!` で正式アイコン読み込みへ変更
- [x] 🪟 `build.rs` を更新: `winres` を使用して `.exe` にアイコンを埋め込む

### STEP 4-3　GitHub Actions (CI) の構築 🔁
- [ ] `.github/workflows/release.yml` を作成
- [ ] macOS (Universal Binary) および Windows (x64) のビルドジョブを設定
- [ ] ビルド成果物（`.app`, `.exe`）の圧縮とリリースアセットへのアップロード自動化
- [ ] ONNX モデルファイルをアセットとして適切に同梱する仕組みの検討

### STEP 4-4　Windows 固有の動作最適化と検証 🪟
- [ ] UAC（ユーザーアカウント制御）ダイアログ表示時や、管理者権限ウィンドウ上でのフィルター透過性を確認
- [ ] トレイアイコンの右クリックメニューの挙動が Windows 標準に準拠しているか確認
- [ ] アプリケーション終了時にトレイアイコンが即座に消えることを確認

### STEP 4-5　最終調整とリリース準備 🔁
- [ ] 🍎 macOS: フルスクリーンアプリ上での表示確認（`NSStatusWindowLevel + 1` の検証）
- [ ] 🍎 macOS: アクセシビリティ権限ダイアログの文言確認
- [ ] `MANUAL.md` の最終推敲

---

## フェーズ完了ログ

### ✅ Phase 0 Completion Log
- **完了日**: 2026-05-15
- **作業者**: Gemini CLI
- **主な決定事項**: macOS プロトタイプ完成

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

## 未解決事項の管理
- [ ] macOS: フルスクリーンアプリへのオーバーレイ (NSStatusWindowLevel + 1 で足りるか)
- [ ] Windows: 管理者権限ウィンドウ上でのマウスイベント透過
- [x] 両OS: アイコン正式生成スクリプトの作成
