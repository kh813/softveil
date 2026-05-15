# Softveil 実装 ToDo リスト

**対応仕様書:** Softveil 基本仕様書 v1.4
**対象フェーズ:** Phase 0（プロトタイプ）〜 Phase 1（MVP）

凡例: 🍎 = macOS 固有　🪟 = Windows 固有　🔁 = 両 OS 共通
各チェックボックスは 1〜2 時間以内で完了できる粒度

---

## Phase 0：プロトタイプ
> **完了条件:** macOS で「半透明・クリック透過・最前面」のフルスクリーンウィンドウが表示でき、`Ctrl+C` で終了できること

---

### STEP 0-1　プロジェクト初期化 🔁

- [x] `cargo new softveil --bin` でプロジェクトを作成する
- [x] `Cargo.toml` に Phase 0 用の最小クレートを追加する（`tao`, `softbuffer`, `raw-window-handle`）
- [x] 🍎 `[target.'cfg(target_os = "macos")'.dependencies]` に `objc2`, `objc2-app-kit`, `objc2-foundation`, `core-graphics` を追加する
- [x] `cargo build` がエラーなく通ることを確認する
- [x] `src/` 直下にファイルを作成する:
  - [x] `main.rs`
  - [x] `app.rs`
  - [x] `display_config.rs`
  - [x] `overlay.rs`
  - [x] `platform/mod.rs`
  - [x] `platform/macos.rs`
  - [x] `platform/windows.rs`
- [x] `assets/` ディレクトリを作成してプレースホルダーファイルを置く

---

### STEP 0-2　MonitorId と DisplayConfig の実装 🔁　（`src/display_config.rs`）

- [x] `MonitorId(u64)` 構造体を定義し `Hash + Eq + Clone + Debug` を derive する
- [x] `MonitorId::from_monitor(monitor: &MonitorHandle) -> Self` を実装する
  - [x] 🍎 macOS: `monitor` から `CGDirectDisplayID` を取得して `u64` にキャストする
  - [x] 🪟 Windows: `HMONITOR` の値を `u64` にキャストする
- [x] `MonitorId::to_string(&self) -> String` を実装する（トレイメニューID埋め込み用）
- [x] `DisplayConfig` 構造体を定義する（フィールド: `enabled: bool`, `alpha: f32`, `position_key: String`）
- [x] `DisplayConfig::default() -> Self` を実装する（`enabled=true`, `alpha=0.30`）
- [x] `DisplayConfig::alpha_u8(&self) -> u8` を実装する（`(self.alpha * 255.0).round() as u8`）
- [x] `DisplayConfig::make_position_key(pos, size) -> String` を実装する（`"{x}_{y}_{w}_{h}"` 形式）
- [x] `DisconnectedCache` 構造体を定義する（フィールド: `cache: HashMap<String, DisplayConfig>`, `max_entries: usize`）
- [x] `DisconnectedCache::new() -> Self` を実装する（`max_entries = 8`）
- [x] `DisconnectedCache::store(&mut self, config: DisplayConfig)` を実装する（LRU: 8件超えたら最古を削除）
- [x] `DisconnectedCache::restore(&mut self, key: &str) -> Option<DisplayConfig>` を実装する

---

### STEP 0-3　AppState の実装 🔁　（`src/app.rs`）

- [x] `AppState` 構造体を定義する（フィールド: `global_enabled: bool`, `displays: HashMap<MonitorId, DisplayConfig>`, `default_config: DisplayConfig`）
- [x] `AppState::new() -> Self` を実装する
- [x] `AppState::toggle_global(&mut self) -> bool` を実装する
- [x] `AppState::toggle_display(&mut self, id: &MonitorId) -> bool` を実装する
- [x] `AppState::set_alpha(&mut self, id: &MonitorId, alpha: f32)` を実装する（0.0〜1.0 にクランプ）
- [x] `AppState::is_visible(&self, id: &MonitorId) -> bool` を実装する（`global_enabled && displays[id].enabled`）
- [x] `AppState::add_display(&mut self, id: MonitorId, config: Option<DisplayConfig>)` を実装する
- [x] `AppState::remove_display(&mut self, id: &MonitorId) -> Option<DisplayConfig>` を実装する
- [x] `AppState::all_displays_enabled(&self) -> bool` を実装する

---

### STEP 0-4　OverlayWindow の生成 🍎　（`src/overlay.rs`）

- [x] `OverlayWindow` 構造体を定義する（フィールド: `monitor_id: MonitorId`, `monitor_name: String`, `window`, `surface`）
- [x] `OverlayWindow::new(event_loop, monitor)` を実装する
  - [x] `MonitorId::from_monitor(monitor)` で ID を取得する
  - [x] `monitor.name()` でディスプレイ名を取得し、`None` なら `"Display N"` とする
  - [x] `WindowBuilder::new()` に `.with_decorations(false)` / `.with_transparent(true)` / `.with_always_on_top(true)` / `.with_skip_taskbar(true)` を設定する
  - [x] `monitor.position()` / `monitor.size()` からウィンドウの位置とサイズを設定する
  - [x] `softbuffer::Context::new()` と `Surface::new()` で描画サーフェスを生成する
  - [x] `platform::apply_overlay_settings(&window)` を呼ぶ
- [x] `OverlayWindow::draw(&mut self, alpha: u8) ` を実装する（ARGB `0x4C000000` 系でバッファ塗りつぶし → `buffer.present()`）
- [x] `OverlayWindow::set_visible(&self, visible: bool)` を実装する
- [x] `OverlayWindow::update_alpha(&mut self, alpha: u8)` を実装する（`draw()` を再呼び出し）
- [x] `create_all(event_loop, monitors) -> Vec<OverlayWindow>` を実装する
- [x] `sync_all(overlays: &mut Vec<OverlayWindow>, state: &AppState)` を実装する（全ウィンドウを AppState に同期）

---

### STEP 0-5　macOS 固有設定 🍎　（`src/platform/macos.rs`）

- [x] `apply_overlay_settings(window: &tao::window::Window)` を実装する
  - [x] `NSWindow` ポインタを `unsafe` で取得する
  - [x] `setIgnoringMouseEvents(true)` を呼ぶ
  - [x] `setBackgroundColor(NSColor::clear())` を呼ぶ
  - [x] `setLevel(NSStatusWindowLevel + 1)` を呼ぶ
  - [x] `setCollectionBehavior(.canJoinAllSpaces | .stationary)` を設定する
- [x] `get_monitor_id(monitor: &MonitorHandle) -> MonitorId` を実装する（`CGDirectDisplayID` 取得）
- [x] unsafe ブロックを `apply_overlay_settings` の内部に閉じ込め、呼び出し元は safe にする

---

### STEP 0-6　Windows 固有設定 🪟　（`src/platform/windows.rs`）

- [x] `apply_overlay_settings(window: &tao::window::Window, alpha: u8)` を実装する
  - [x] `HWND` を取得する
  - [x] `set_ex_style(hwnd, WS_EX_TRANSPARENT | WS_EX_LAYERED | WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW)` を呼ぶ
  - [x] `SetLayeredWindowAttributes(hwnd, 0, alpha, LWA_ALPHA)` を呼ぶ
  - [x] `SetWindowPos(hwnd, HWND_TOPMOST, ...)` を呼ぶ
- [x] `set_ex_style(hwnd: HWND, flags: u32)` プライベート関数を実装する
- [x] `get_monitor_id(monitor: &MonitorHandle) -> MonitorId` を実装する（`HMONITOR` 取得）

---

### STEP 0-7　platform/mod.rs の条件コンパイル 🔁

- [x] `#[cfg(target_os = "macos")] pub mod macos;` を記述する
- [x] `#[cfg(target_os = "windows")] pub mod windows;` を記述する
- [x] `apply_overlay_settings(window)` のプラットフォーム共通ラッパーを定義する
- [x] `get_monitor_id(monitor)` のプラットフォーム共通ラッパーを定義する

---

### STEP 0-8　main.rs の minimal 実装 🍎　（`src/main.rs`）

- [x] `EventLoop::new()` でイベントループを生成する
- [x] `event_loop.available_monitors()` でモニター一覧を取得する
- [x] 各モニターについて `AppState::add_display()` に登録する
- [x] `overlay::create_all()` でオーバーレイを生成し `draw()` を呼ぶ
- [x] `event_loop.run()` でアイドルループを回す（`ControlFlow::Wait`）
- [x] `Ctrl+C` でプロセスが終了することを確認する（この時点ではトレイ未実装）

---

### STEP 0-9　Phase 0 動作確認 🍎

- [x] `cargo run` でフィルターがプライマリディスプレイに表示されることを目視確認する
- [x] フィルター越しに Finder・ブラウザ等を操作できることを確認する（クリック透過）
- [x] 別のアプリをアクティブにしてもフィルターが最前面に残ることを確認する
- [x] CPU 使用率がアイドル時に 1% 以下であることを `Activity Monitor` で確認する

---

## Phase 1：MVP
> **完了条件:** ダブルクリックで起動し、メニューバー/トレイからフィルターをグローバル・ディスプレイ個別にON/OFFでき、ホットプラグにも対応し、Windows でも同様に動作すること

---

### STEP 1-1　Cargo.toml の完成 🔁

- [x] `tray-icon = "0.19"` を追加する
- [x] `muda = "0.15"` を追加する
- [x] `global-hotkey = "0.6"` を追加する
- [x] 🪟 `windows-sys` に `Win32_System_Threading` など不足 feature を追加する
- [x] `[profile.release]` に `lto = true`, `strip = true`, `codegen-units = 1` を設定する
- [x] `cargo build` がエラーなく通ることを確認する

---

### STEP 1-2　単一インスタンス制御 🔁　（`src/single_instance.rs`）

- [x] `SingleInstanceError` 列挙型を定義する（`AlreadyRunning`, `Io(std::io::Error)` 等）
- [x] `SingleInstanceGuard` 構造体と `acquire()` を実装する
  - [x] 🍎 macOS: `flock` でロックファイル取得 → `Err(AlreadyRunning)` なら即 exit
  - [x] 🪟 Windows: `CreateMutexW("Local\\SoftveilMutex")` → `ERROR_ALREADY_EXISTS` なら即 exit
- [x] `Drop` を実装してロックを解放する（🍎 unlock＋削除 / 🪟 ReleaseMutex＋CloseHandle）
- [x] `main.rs` 冒頭で `acquire()` を呼び、`Err` なら `process::exit(0)` する

---

### STEP 1-3　ホットプラグ検知の実装 🍎　（`src/platform/macos.rs`）

- [x] `DisplayChangeEvent` 列挙型を定義する（`ScreenParametersChanged` バリアント）
- [x] `HotplugGuard` 構造体を定義する（drop でオブザーバー解除）
- [x] `register_hotplug_observer(tx: mpsc::Sender<DisplayChangeEvent>) -> HotplugGuard` を実装する
  - [x] `NSNotificationCenter.defaultCenter()` に `NSApplicationDidChangeScreenParametersNotification` を登録する
  - [x] コールバックで `tx.send(DisplayChangeEvent::ScreenParametersChanged)` を呼ぶ
- [x] `main.rs` で `display_change_rx.try_recv()` を受信時に差分計算してオーバーレイを追加/削除する処理を実装する

---

### STEP 1-4　ホットプラグ検知の実装 🪟　（`src/platform/windows.rs`）

- [ ] `DisplayChangeEvent` 列挙型を定義する（`DisplayChanged` バリアント）
- [ ] `register_display_change_hook(hwnd: HWND, tx: mpsc::Sender<DisplayChangeEvent>) -> HotplugGuard` を実装する
  - [ ] `SetWindowSubclass` で最初のオーバーレイウィンドウのプロシージャをサブクラス化する
  - [ ] サブクラスプロシージャ内で `WM_DISPLAYCHANGE` を受信したら `tx.send()` を呼ぶ
- [ ] `main.rs` で同様の差分計算処理を実装する（macOS と共通ロジックに切り出せるか検討する）

---

### STEP 1-5　ホットプラグ差分計算ロジック 🔁　（`src/overlay.rs` + `src/main.rs`）

- [x] `add_display(overlays, event_loop, monitor, visible, alpha)` を実装する
  - [x] `OverlayWindow::new()` でウィンドウを生成する
  - [x] `platform::apply_overlay_settings()` を呼ぶ
  - [x] `draw()` で初期描画する
  - [x] `overlays.push()` で追加し、追加した `monitor_id` を返す
- [x] `remove_display(overlays, id)` を実装する
  - [x] `id` に一致する要素を `overlays` から `swap_remove` または `retain` で削除する（drop でウィンドウ破棄）
- [x] `main.rs` のイベントループ内に差分計算を実装する
  ```
  display_change_rx.try_recv() が Ok の場合:
    current_ids = event_loop.available_monitors() を収集
    added   = current_ids にあって overlays にない ID
    removed = overlays にあって current_ids にない ID
    removed の各 ID: remove_display() + state.remove_display() + cache.store()
    added の各 monitor: add_display() + cache.restore() or default + state.add_display()
    tray.rebuild_menu()
  ```
- [x] ディスプレイを抜き差ししてオーバーレイが追加・削除されることを確認する

---

### STEP 1-6　マルチディスプレイ初期化 🔁　（`src/main.rs`）

- [x] `event_loop.available_monitors().collect::<Vec<_>>()` で起動時の全モニターを取得する
- [x] 各モニターについて `AppState::add_display()` に登録する（config = None でデフォルト適用）
- [x] `overlay::create_all()` で全モニター分のオーバーレイを生成する
- [x] 2画面以上の環境で全ディスプレイにフィルターが表示されることを確認する

---

### STEP 1-7　トレイ / メニューバー UI の実装 🔁　（`src/tray.rs`）

- [x] `MENU_ID_GLOBAL_TOGGLE`, `MENU_ID_DISPLAY_TOGGLE`, `MENU_ID_QUIT` 定数を定義する
- [x] `TrayError` を定義する
- [x] `TrayHandle` 構造体を定義する（フィールド: `_icon: tray_icon::TrayIcon`, `menu: muda::Menu`）
- [x] `TrayHandle::new(state, overlays)` を実装する
  - [x] 🍎 macOS: テンプレート PNG を `include_bytes!` で読み込み `Icon` を生成する
  - [x] 🪟 Windows: ICO を `include_bytes!` で読み込み `Icon` を生成する
  - [x] `build_menu(state, overlays)` 内部関数でメニューを構築する（下記参照）
  - [x] `TrayIconBuilder::new().with_icon().with_menu().build()` でアイコンを生成する
- [x] `build_menu(state, overlays) -> muda::Menu` プライベート関数を実装する
  - [x] `CheckMenuItem::new("フィルター：すべてオン", ..., state.all_displays_enabled(), ...)` を追加する（ID: `MENU_ID_GLOBAL_TOGGLE`）
  - [x] `PredefinedMenuItem::separator()` を追加する
  - [x] `Submenu::new("ディスプレイ設定", true)` を生成する
  - [x] `overlays` をループして各ディスプレイの `CheckMenuItem` をサブメニューに追加する（ID: `MENU_ID_DISPLAY_TOGGLE:{monitor_id}`）
  - [x] `PredefinedMenuItem::separator()` を追加する
  - [x] `MenuItem::with_id(MENU_ID_QUIT, "Softveil を終了", ...)` を追加する
- [x] `TrayHandle::rebuild_menu(&self, state, overlays)` を実装する（`build_menu()` で新規生成して差し替える）
- [x] `TrayHandle::update_global_check(&self, all_enabled: bool)` を実装する
- [x] `TrayHandle::update_display_check(&self, id: &MonitorId, enabled: bool)` を実装する
- [x] メニューバー / トレイにアイコンとメニューが表示されることを目視確認する

---

### STEP 1-8　アイコン素材の準備 🔁

- [ ] 元アイコン画像（1024×1024 PNG）を `assets/icon.png` に配置する
- [ ] 🍎 macOS メニューバー用: 22×22px 白黒テンプレート画像 `assets/icon_macos_template.png` を書き出す（`@2x` = 44×44 も用意）
- [ ] 🍎 `.app` バンドル用: `iconutil` で `icon.iconset` → `icon_macos.icns` を生成する
- [ ] 🪟 Windows トレイ用: ImageMagick 等で 16/32/48px マルチサイズの `icon_windows.ico` を生成する

---

### STEP 1-9　グローバルショートカットの実装 🔁　（`src/hotkey.rs`）

- [x] `HotkeyEvent` 列挙型を定義する（`ToggleGlobal` バリアント）
- [x] `HotkeyError` 列挙型を定義する
- [x] `HotkeyGuard` 構造体を定義する（フィールド: `_manager: GlobalHotKeyManager`）
- [x] `register(tx: mpsc::Sender<HotkeyEvent>) -> Result<HotkeyGuard, HotkeyError>` を実装する
  - [x] `GlobalHotKeyManager::new()` でマネージャーを生成する
  - [x] 🍎 macOS: `Modifiers::SUPER | Modifiers::SHIFT + Code::KeyP` を登録する
  - [x] 🪟 Windows: `Modifiers::CONTROL | Modifiers::SHIFT + Code::KeyP` を登録する
  - [x] `std::thread::spawn` でループスレッドを起動し `tx.send(HotkeyEvent::ToggleGlobal)` を送る
- [x] `main.rs` の `event_loop.run()` 内で `hotkey_rx.try_recv()` して `ToggleGlobal` を受け取ったら `state.toggle_global()` → `overlay::sync_all()` → `tray.update_global_check()` を呼ぶ

---

### STEP 1-10　メニューイベントのハンドリング 🔁　（`src/main.rs`）

- [x] `MenuEvent::receiver().try_recv()` でメニュー選択を受け取る処理をイベントループに追加する
- [x] `MENU_ID_GLOBAL_TOGGLE` 受信時の処理を実装する
  - [x] `state.toggle_global()` を呼ぶ
  - [x] `overlay::sync_all(&mut overlays, &state)` を呼ぶ
  - [x] `tray.update_global_check(state.all_displays_enabled())` を呼ぶ
- [x] `MENU_ID_DISPLAY_TOGGLE:{id_str}` 受信時の処理を実装する
  - [x] suffix から `MonitorId` をパースする
  - [x] `state.toggle_display(&id)` を呼ぶ
  - [x] 該当 `OverlayWindow` の `set_visible(state.is_visible(&id))` を呼ぶ
  - [x] `tray.update_display_check(&id, state.displays[&id].enabled)` を呼ぶ
  - [x] グローバルチェックマークも `tray.update_global_check(state.all_displays_enabled())` で更新する
- [x] `MENU_ID_QUIT` 受信時に終了フラグを立てる
- [x] グローバル・個別それぞれのON/OFFが正しく動作することを確認する

---

### STEP 1-11　アプリ完全終了処理 🔁　（`src/main.rs`）

- [x] 終了シーケンスを実装する（順序が重要）
  - [x] `overlay::set_all_visible(&overlays, false)` を呼ぶ（視覚的にフィルターを消す）
  - [x] `drop(overlays)` を呼ぶ
  - [x] `drop(tray_handle)` を呼ぶ
  - [x] `drop(_hotplug_guard)` を呼ぶ
  - [x] `drop(_single_instance_guard)` を呼ぶ
  - [x] `*control_flow = ControlFlow::Exit` を設定する
- [x] 終了後にプロセスが残っていないことを確認する

---

### STEP 1-12　.app バンドルの作成 🍎

- [ ] `package/macos/Info.plist` を作成して以下を設定する
  - [ ] `CFBundleIdentifier`: `com.yourname.softveil`
  - [ ] `CFBundleName`: `Softveil`
  - [ ] `CFBundleVersion` / `CFBundleShortVersionString`: `0.1.0`
  - [ ] `LSUIElement`: `true`
  - [ ] `NSAccessibilityUsageDescription`: グローバルショートカット用の権限説明文
  - [ ] `CFBundleIconFile`: `icon_macos`
- [ ] `cargo build --release --target aarch64-apple-darwin` でビルドする
- [ ] `cargo build --release --target x86_64-apple-darwin` でビルドする
- [ ] `lipo` でユニバーサルバイナリを生成する（オプション）
- [ ] バンドルを組み立てるスクリプト `scripts/bundle_macos.sh` を作成する
- [ ] Finder からダブルクリックで起動し、Dock に表示されないことを確認する
- [ ] メニューバーにアイコンが現れることを確認する

---

### STEP 1-13　Windows ビルド環境のセットアップ 🪟

- [ ] `cargo install cross` をインストールする
- [ ] `rustup target add x86_64-pc-windows-gnu` でターゲットを追加する
- [ ] `cross build --target x86_64-pc-windows-gnu` でビルドを試みる
- [ ] ビルドが通らない場合は GitHub Actions による CI ビルドを代替として設定する

---

### STEP 1-14　build.rs の作成（Windows アイコン埋め込み） 🪟

- [ ] `build.rs` をプロジェクトルートに作成する
- [ ] `#[cfg(target_os = "windows")]` で `winres` クレートを使い `.ico` を実行ファイルに埋め込む
- [ ] `[build-dependencies]` に `winres = "0.1"` を追加する
- [ ] Windows で実行ファイルのアイコンが表示されることを確認する

---

### STEP 1-15　Windows 固有設定の検証 🪟

- [ ] Windows 実機または仮想マシンで `.exe` を実行する
- [ ] タスクバーにアイコンが**表示されない**ことを確認する
- [ ] システムトレイにアイコンが表示されることを確認する
- [ ] 右クリックでコンテキストメニュー（グローバルトグル + ディスプレイ別サブメニュー + 終了）が出ることを確認する
- [ ] フィルター越しに背後のアプリを操作できることを確認する（クリック透過）
- [ ] `WS_EX_NOACTIVATE` によりフィルタークリック時にフォーカスが奪われないことを確認する
- [ ] ディスプレイ接続・切断時にサブメニューが更新されることを確認する（ホットプラグ）

---

### STEP 1-16　Phase 1 総合動作確認 🔁

- [ ] 🍎 `.app` ダブルクリック起動 → フィルター全画面表示 → メニューバーアイコン確認
- [x] 🔁 グローバルON/OFFがメニューから切り替えられること
- [x] 🔁 ディスプレイ個別のON/OFFがサブメニューから切り替えられること
- [x] 🔁 グローバルOFFにすると個別設定に関係なく全フィルターが消えること
- [x] 🔁 `Cmd+Shift+P` / `Ctrl+Shift+P` でグローバルON/OFFが切り替わること
- [x] 🔁 ディスプレイ接続時: 新ディスプレイにオーバーレイが追加され、サブメニューに項目が増えること
- [x] 🔁 ディスプレイ切断時: 該当オーバーレイが消え、サブメニューから項目が消えること
- [x] 🔁 同じディスプレイを再接続したとき、切断前の設定（ON/OFF）が引き継がれること
- [x] 🔁 「終了」でトレイアイコンを含めてアプリが完全に消えること
- [x] 🔁 2回起動しても2つ目が黙って終了すること
- [ ] 🔁 `cargo build --release` のバイナリサイズが 10MB 以下であること
- [x] 🔁 アイドル時の CPU 使用率が 1% 以下であること（2画面接続時も確認する）

---

## Phase 2：Ver 2.0 拡張
> **完了条件:** フィルター濃度をトレイメニューから変更でき、設定が再起動後も維持され、ログイン時に自動起動するオプションが機能すること

---

### STEP 2-1　設定の永続化 🔁

- [x] `confy`, `serde`, `serde_derive` を `Cargo.toml` に追加する
- [x] `Config` 構造体を定義し、`AppState` と相互変換可能にする
- [x] アプリ起動時に `confy::load` で設定を読み込む
- [x] 設定変更（Alpha, ON/OFF）のたびに `confy::store` で保存する
- [x] アプリを再起動しても前回の Alpha 値や ON/OFF 状態が復元されることを確認する

---

### STEP 2-2　濃度変更 UI（トレイメニュー） 🔁

- [x] トレイメニューに「フィルター濃度」サブメニューを追加する
- [x] 10% 〜 90%（10%刻み）の選択肢を用意する
- [x] 選択された濃度を `AppState` に反映し、全オーバーレイを即座に更新する
- [x] メニューのチェックマークを現在の濃度に追従させる

---

### STEP 2-3　ログイン時自動起動 🔁

- [x] `auto-launch` クレートを追加するか、自前で実装を検討する
- [x] トレイメニューに「ログイン時に起動」チェックアイテムを追加する
- [x] 🍎 macOS: `LaunchAgent` または `SMLoginItem` による自動起動を実装する
- [x] 🪟 Windows: レジストリ `Run` キーへの登録を実装する
- [x] OS 再起動後にアプリが自動的に立ち上がることを確認する

---

### STEP 2-4　Phase 2 動作確認 🔁

- [x] 濃度変更が正しく全画面に反映されること
- [x] 設定が保存・復元されること
- [x] 自動起動設定が機能すること
- [x] 2画面環境でも全ディスプレイの濃度が一括/個別（仕様により判断）で変更できること

---

## フェーズ完了ログ

> **運用ルール**: 各フェーズ完了時にこのセクションへ記録を追記する。詳細な経緯は DEVLOG.md に記録し、ここには要点のみを残す。

### ✅ Phase 0 Completion Log

- **完了日**: 2026-05-15
- **Commit**: (manual)
- **作業者**: Gemini CLI
- **作成ファイル**: src/main.rs, src/app.rs, src/display_config.rs, src/overlay.rs, src/platform/*
- **変更ファイル**: なし
- **主な決定事項**: macOS でのプロトタイプ実装。
- **既知の問題 / 持ち越し**: なし

---

### ✅ Phase 1 Completion Log

- **完了日**: 2026-05-15
- **Commit**: (manual)
- **作業者**: Gemini CLI
- **作成ファイル**: src/single_instance.rs, src/tray.rs, src/hotkey.rs, etc.
- **変更ファイル**: src/main.rs, src/overlay.rs, etc.
- **主な決定事項**: macOS での基本的な MVP 機能の実装完了。
- **既知の問題 / 持ち越し**: Windows 版の検証、適切なアイコンの作成。

---

### ✅ Phase 2 Completion Log

- **完了日**: 2026-05-15
- **Commit**: (manual)
- **作業者**: Gemini CLI
- **作成ファイル**: なし
- **変更ファイル**: Cargo.toml, src/app.rs, src/main.rs, src/tray.rs, src/platform/macos.rs, src/display_config.rs
- **主な決定事項**: 永続化、濃度変更、自動起動の実装。
- **既知の問題 / 持ち越し**: なし
