#![windows_subsystem = "windows"]

mod app;
mod display_config;
mod overlay;
mod platform;
mod single_instance;
mod tray;
mod hotkey;
mod ai_detection;
mod benchmark;

#[cfg(test)]
mod tests;

use app::AppState;
use display_config::{MonitorId, DisplayConfig, DisconnectedCache, FilterMode, DisplayCategory};
use tray::{
    TrayHandle, MENU_ID_GLOBAL_TOGGLE, MENU_ID_DISPLAY_TOGGLE_PREFIX, MENU_ID_ALPHA_PREFIX, 
    MENU_ID_MODE_PREFIX, MENU_ID_PANEL_PREFIX, MENU_ID_CATEGORY_PREFIX, MENU_ID_INTENSITY_PREFIX, 
    MENU_ID_OVERRIDE_PERIOD_PREFIX, MENU_ID_OVERRIDE_COVER_PREFIX, MENU_ID_OVERRIDE_SPEED_PREFIX,
    MENU_ID_RESET_RECOMMENDED, MENU_ID_AUTO_START, MENU_ID_AI_DETECTION,
    MENU_ID_PRESET_APPLY_PREFIX, MENU_ID_PRESET_DELETE_PREFIX, MENU_ID_PRESET_SAVE_CURRENT, MENU_ID_PRESET_CLEAR_ALL,
    MENU_ID_RUN_BENCHMARK, MENU_ID_QUIT
};
use ai_detection::{AIDetectionCommand, start_detection_thread};
use auto_launch::AutoLaunchBuilder;
use hotkey::HotkeyEvent;
use tao::event_loop::ControlFlow;
use tao::event::Event;
use muda::MenuEvent;
use std::sync::mpsc;

#[derive(Debug)]
pub enum UserEvent {
    Hotkey(HotkeyEvent),
    AIDetected(bool),
    DisplayChange,
    RunBenchmark,
    ProcessBenchmarkCommand,
    BenchmarkProgress(f32, String),
    BenchmarkFinished(String),
}

#[derive(Debug)]
pub enum BenchmarkCommand {
    Sync,
    Capture(MonitorId, mpsc::Sender<Result<image::DynamicImage, String>>),
    CaptureBatch(Vec<MonitorId>, mpsc::Sender<Vec<(MonitorId, Result<image::DynamicImage, String>)>>),
    SetTestSettings(MonitorId, FilterMode, f32, Option<f32>, Option<f32>, Option<f32>),
    SetBatchSettings(Vec<(MonitorId, FilterMode, f32, Option<f32>, Option<f32>, Option<f32>)>),
    Progress(f32, String),
    Finished(Vec<crate::display_config::Preset>, String),
}

#[macro_export]
macro_rules! logger {
    ($($arg:tt)*) => {
        let msg = format!($($arg)*);
        println!("{}", msg);
        $crate::platform::write_to_log_file(&msg);
    }
}

fn main() {
    logger!("--- Softveil Starting ---");
    let args: Vec<String> = std::env::args().collect();
    let is_benchmark = args.contains(&"--benchmark".to_string());

    #[cfg(target_os = "windows")]
    platform::windows::enable_dpi_awareness();

    let _guard = match single_instance::acquire() {
        Ok(guard) => {
            println!("Single instance lock acquired.");
            guard
        },
        Err(e) => {
            eprintln!("Softveil is already running or failed to start: {:?}", e);
            std::process::exit(1);
        }
    };

    let event_loop = tao::event_loop::EventLoopBuilder::<UserEvent>::with_user_event().build();
    let proxy = event_loop.create_proxy();

    let monitors: Vec<_> = event_loop.available_monitors().collect();
    println!("Found {} monitors.", monitors.len());
    
    let mut state = AppState::new();
    let mut cache = DisconnectedCache::new();

    let (bench_cmd_tx, bench_cmd_rx) = mpsc::channel::<BenchmarkCommand>();
    let mut current_bench_resp_tx: Option<mpsc::Sender<()>> = None;

    // Phase 5: Check Screen Capture access (without automatic prompt)
    #[cfg(target_os = "macos")]
    {
        if !platform::has_screen_capture_access() {
            logger!("Screen capture access is not granted. Advanced Phase 5 features will be limited.");
        } else {
            logger!("Screen capture access is granted.");
        }
    }
    let gpu = match pollster::block_on(overlay::GpuContext::new()) {
        Some(ctx) => std::sync::Arc::new(ctx),
        None => {
            let msg = "GPUの初期化に失敗しました。グラフィックドライバーが最新であることを確認してください。\nFailed to initialize GPU. Please ensure your graphics drivers are up to date.";
            eprintln!("{}", msg);
            platform::show_error_dialog("Softveil Error", msg);
            std::process::exit(1);
        }
    };
    
    // Initial display registration
    for monitor in &monitors {
        let id = MonitorId::from_monitor(monitor);
        let pos_key = DisplayConfig::make_position_key(monitor.position(), monitor.size());
        
        // 【追加】カテゴリと PPI を自動検出
        let (category, ppi) = platform::detect_display_category(monitor);
        println!(
            "Monitor {:?}: category={:?}, ppi={:.1}, pos_key={}",
            id, category, ppi, pos_key
        );

        state.add_display_with_pos_and_profile(id, pos_key, category, ppi);
    }
    
    let mut overlays = overlay::create_all(&event_loop, monitors, &state, &gpu);
    println!("Created {} overlay windows.", overlays.len());
    
    // Sync detected panel types back to state if they are Unknown
    for overlay in &overlays {
        if state.panel_type(&overlay.monitor_id) == crate::display_config::PanelType::Unknown {
            state.set_panel_type(&overlay.monitor_id, overlay.panel_type);
        }
    }
    
    // Sync initial overlays with loaded state
    state.check_stealth_transition();
    overlay::sync_all(&mut overlays, &state, &gpu);
    
    if is_benchmark {
        benchmark::run_benchmark(gpu.clone(), state, &mut overlays);
        std::process::exit(0);
    }

    let tray_handle = match TrayHandle::new(&state, &overlays) {
        Ok(t) => {
            println!("Tray icon created.");
            Some(t)
        },
        Err(e) => {
            eprintln!("Failed to create tray icon: {:?}", e);
            None
        }
    };
    
    let (hotkey_tx, hotkey_rx) = mpsc::channel();
    let _hotkey_guard = match hotkey::register(hotkey_tx) {
        Ok(guard) => Some(guard),
        Err(e) => {
            eprintln!("Failed to register hotkey: {:?}", e);
            platform::show_error_dialog(
                "Softveil Warning",
                "グローバルショートカット（Ctrl+Shift+P）の登録に失敗しました。他のアプリと競合している可能性があります。\nFailed to register global hotkey (Ctrl+Shift+P). It might be in use by another application."
            );
            None
        }
    };

    // Spawn a thread to forward hotkey events to the event loop
    let proxy_for_hotkey = proxy.clone();
    std::thread::spawn(move || {
        while let Ok(event) = hotkey_rx.recv() {
            let _ = proxy_for_hotkey.send_event(UserEvent::Hotkey(event));
        }
    });

    let (display_change_tx, display_change_rx) = mpsc::channel();
    let _hotplug_guard = platform::register_hotplug_handler(display_change_tx);

    // Spawn a thread to forward display change events to the event loop
    let proxy_for_display = proxy.clone();
    std::thread::spawn(move || {
        while display_change_rx.recv().is_ok() {
            // Batch multiple events into one
            while display_change_rx.try_recv().is_ok() {}
            let _ = proxy_for_display.send_event(UserEvent::DisplayChange);
        }
    });

    let app_path = std::env::current_exe().unwrap_or_default();
    let auto_launcher = if !app_path.as_os_str().is_empty() {
        AutoLaunchBuilder::new()
            .set_app_name("Softveil")
            .set_app_path(&app_path.to_string_lossy())
            .set_use_launch_agent(true) // For macOS
            .build()
            .ok()
    } else {
        None
    };

    if let Some(ref al) = auto_launcher {
        if state.auto_start {
            let _ = al.enable();
        } else {
            let _ = al.disable();
        }
    }

    let (ai_cmd_tx, ai_cmd_rx) = mpsc::channel();
    let (ai_event_tx, ai_event_rx) = mpsc::channel();
    start_detection_thread(ai_cmd_rx, ai_event_tx);

    if state.ai_detection_enabled {
        let _ = ai_cmd_tx.send(AIDetectionCommand::Start);
    }

    // Spawn a thread to forward AI detection events to the event loop
    let proxy_for_ai = proxy.clone();
    std::thread::spawn(move || {
        while let Ok(detected) = ai_event_rx.recv() {
            let _ = proxy_for_ai.send_event(UserEvent::AIDetected(detected));
        }
    });

    let menu_channel = MenuEvent::receiver();

    let mut last_frame_time = std::time::Instant::now();

    event_loop.run(move |event, event_loop_target, control_flow| {
        let mut needs_animation = calc_needs_animation(&overlays, &state);
        *control_flow = if needs_animation {
            ControlFlow::Poll
        } else {
            ControlFlow::Wait
        };

        match event {
            Event::MainEventsCleared => {
                // 1. Handle animation if needed
                if needs_animation {
                    let now = std::time::Instant::now();
                    // 16ms (60fps) 以上経過していたら描画する
                    // モードによっては 33ms (30fps) でも良いが、まずは一律 60fps 上限とする
                    if now.duration_since(last_frame_time) >= std::time::Duration::from_millis(16) {
                        last_frame_time = now;
                        for overlay in overlays.iter_mut() {
                            if state.is_visible(&overlay.monitor_id) {
                                let alpha = state.effective_alpha_u8(&overlay.monitor_id);
                                let _ = overlay.draw(&gpu, &state, alpha);
                            }
                        }
                    }
                }

                // 2. Handle commands from benchmark thread
                while let Ok(cmd) = bench_cmd_rx.try_recv() {
                    match cmd {
                        BenchmarkCommand::Sync => {
                            overlay::sync_all(&mut overlays, &state, &gpu);
                            if let Some(ref tx) = current_bench_resp_tx {
                                let _ = tx.send(());
                            }
                        }
                        BenchmarkCommand::Capture(id, tx) => {
                            let res = platform::capture_display(&id);
                            let _ = tx.send(res);
                        }
                        BenchmarkCommand::CaptureBatch(ids, tx) => {
                            let mut results = Vec::new();
                            for id in ids {
                                let res = platform::capture_display(&id);
                                results.push((id, res));
                            }
                            let _ = tx.send(results);
                        }
                        BenchmarkCommand::SetTestSettings(id, mode, alpha, period, cover, speed) => {
                            state.set_filter_mode(&id, mode);
                            state.set_display_alpha(&id, alpha);
                            state.set_override_period(&id, period);
                            state.set_override_cover_ratio(&id, cover);
                            state.set_override_scroll_speed(&id, speed);
                            overlay::sync_all(&mut overlays, &state, &gpu);
                            if let Some(ref tx) = current_bench_resp_tx {
                                let _ = tx.send(());
                            }
                        }
                        BenchmarkCommand::SetBatchSettings(batch) => {
                            for (id, mode, alpha, period, cover, speed) in batch {
                                state.set_filter_mode(&id, mode);
                                state.set_display_alpha(&id, alpha);
                                state.set_override_period(&id, period);
                                state.set_override_cover_ratio(&id, cover);
                                state.set_override_scroll_speed(&id, speed);
                            }
                            overlay::sync_all(&mut overlays, &state, &gpu);
                            if let Some(ref tx) = current_bench_resp_tx {
                                let _ = tx.send(());
                            }
                        }
                        BenchmarkCommand::Progress(progress, message) => {
                            let _ = proxy.send_event(UserEvent::BenchmarkProgress(progress, message));
                        }
                        BenchmarkCommand::Finished(new_presets, summary) => {
                            println!("Benchmark finished. Received {} new presets.", new_presets.len());
                            for preset in new_presets {
                                state.save_preset(preset.name, preset.settings);
                            }
                            if let Some(ref t) = tray_handle {
                                t.rebuild_menu(&state, &overlays);
                            }
                            let _ = proxy.send_event(UserEvent::BenchmarkFinished(summary));
                        }
                    }
                }
            }
            Event::RedrawRequested(window_id)
                // Pollモード中はMainEventsClearedで描画するため不要
                if !needs_animation => {
                    if let Some(overlay) = overlays.iter_mut().find(|o| o.window.id() == window_id) {
                        let alpha = state.effective_alpha_u8(&overlay.monitor_id);
                        let _ = overlay.draw(&gpu, &state, alpha);
                    }
                }
            Event::WindowEvent {
                event: tao::event::WindowEvent::Resized(size),
                window_id,
                ..
            } => {
                if let Some(overlay) = overlays.iter_mut().find(|o| o.window.id() == window_id) {
                    overlay.resize(&gpu, size.width, size.height);
                }
            }
            // Handle Hotkey events via UserEvent
            Event::UserEvent(UserEvent::Hotkey(HotkeyEvent::ToggleGlobal)) => {
                println!("Event: Toggle Global (Hotkey)");
                state.toggle_global();
                state.save();
                overlay::sync_all(&mut overlays, &state, &gpu);
                needs_animation = calc_needs_animation(&overlays, &state);
                if let Some(ref t) = tray_handle {
                    t.rebuild_menu(&state, &overlays);
                }
            }
            Event::UserEvent(UserEvent::AIDetected(detected))
                if state.ai_peeper_detected != detected => {
                    println!("Event: AI Peeper Detected = {}", detected);
                    state.set_peeper_detected(detected);
                    overlay::sync_all(&mut overlays, &state, &gpu);
                    needs_animation = calc_needs_animation(&overlays, &state);
                }
            Event::UserEvent(UserEvent::RunBenchmark) => {
                #[cfg(target_os = "macos")]
                {
                    // Use request_screen_capture_access which handles the check and request natively.
                    // If it returns false, it means we don't have access yet.
                    if !platform::request_screen_capture_access() {
                        platform::show_error_dialog(
                            "「画面収録」の許可が必要です",
                            "ベンチマーク機能には画面収録の権限が必要です。\nシステム設定 > プライバシーとセキュリティ > 画面収録 で Softveil を許可してください。",
                        );
                        // Do not proceed if we are reasonably sure we don't have access
                        return;
                    }
                }
                logger!("Starting benchmark from UI...");
                state.benchmark_progress = Some(0.0);
                if let Some(ref t) = tray_handle {
                    t.rebuild_menu(&state, &overlays);
                }

                platform::send_notification("Softveil", "ベンチマーク開始", "画面の最適化測定を開始しました。完了までしばらくお待ちください。");
                
                let monitor_info: Vec<(MonitorId, String)> = overlays.iter()
                    .map(|o| (o.monitor_id, o.monitor_name.clone()))
                    .collect();
                
                let mut original_settings = std::collections::HashMap::new();
                for overlay in &overlays {
                    if let Some(config) = state.displays.get(&overlay.monitor_id) {
                        original_settings.insert(overlay.monitor_id, config.get_settings());
                    }
                }

                let (resp_tx, resp_rx) = mpsc::channel();
                current_bench_resp_tx = Some(resp_tx);

                let cmd_tx = bench_cmd_tx.clone();
                let proxy_clone = proxy.clone();
                
                std::thread::spawn(move || {
                    benchmark::run_benchmark_threaded(monitor_info, cmd_tx, resp_rx, original_settings, proxy_clone);
                });
            }
            Event::UserEvent(UserEvent::ProcessBenchmarkCommand) => {
                // Do nothing, just wakeup and hit MainEventsCleared
            }
            Event::UserEvent(UserEvent::BenchmarkProgress(progress, ref message)) => {
                println!("Benchmark Progress: {:.0}% - {}", progress * 100.0, message);
                let old_progress = state.benchmark_progress.unwrap_or(0.0);
                state.benchmark_progress = Some(progress);
                
                if let Some(ref t) = tray_handle {
                    t.set_tooltip(&format!("Softveil (ベンチマーク中: {:.0}%)", progress * 100.0));
                    
                    // Rebuild menu only on significant steps to avoid constant closure on macOS,
                    // but enough to show it's moving if the user re-opens it.
                    if (progress * 10.0).floor() > (old_progress * 10.0).floor() || progress == 0.0 {
                         t.rebuild_menu(&state, &overlays);
                    }
                }

                // 25% ごとに通知
                if (progress * 4.0).floor() > (old_progress * 4.0).floor() {
                    platform::send_notification("Softveil", "最適化進行中", &format!("進捗: {:.0}%", progress * 100.0));
                }
            }
            Event::UserEvent(UserEvent::BenchmarkFinished(ref summary)) => {
                state.benchmark_progress = None;
                if let Some(ref t) = tray_handle {
                    t.set_tooltip("Softveil");
                    t.rebuild_menu(&state, &overlays);
                }

                platform::send_notification("Softveil", "最適化完了", "全モニターの性能測定が完了しました。");
                
                crate::platform::show_info_dialog(
                    "最適化完了 / Optimization Complete",
                    &format!("全モニターの性能測定と最適化が完了しました。\n\n【結果の要約】\n{}\n\n設定プリセットメニューから適用可能です。", summary)
                );
            }
            Event::UserEvent(UserEvent::DisplayChange) => {
                let current_monitors: Vec<_> = event_loop_target.available_monitors().collect();
                let current_ids: Vec<MonitorId> = current_monitors.iter().map(MonitorId::from_monitor).collect();
                
                let mut existing_ids: Vec<MonitorId> = overlays.iter().map(|o| o.monitor_id).collect();
                existing_ids.sort();
                let mut new_ids = current_ids.clone();
                new_ids.sort();

                if existing_ids != new_ids {
                    println!("Display configuration changed. Recalculating... Current IDs: {:?}, New IDs: {:?}", existing_ids, new_ids);
                    
                    let mut removed_ids = Vec::new();
                    for overlay in &overlays {
                        if !current_ids.contains(&overlay.monitor_id) {
                            removed_ids.push(overlay.monitor_id);
                        }
                    }
                    
                    for id in removed_ids {
                        println!("Removing display: {:?}", id);
                        if let Some(config) = state.remove_display(&id) {
                            cache.store(config);
                        }
                        overlay::remove_display(&mut overlays, &id);
                    }
                    
                    for monitor in current_monitors {
                        let id = MonitorId::from_monitor(&monitor);
                        if !overlays.iter().any(|o| o.monitor_id == id) {
                            let pos_key = DisplayConfig::make_position_key(monitor.position(), monitor.size());
                            let (category, ppi) = platform::detect_display_category(&monitor);
                            
                            println!("Adding display: {:?}, category={:?}, ppi={:.1}", id, category, ppi);

                            if let Some(config) = cache.restore(&pos_key) {
                                state.add_display(id, Some(config));
                                // Restore might have Unknown if it was saved that way, so update if needed
                                if state.display_category(&id) == DisplayCategory::Unknown {
                                    state.set_display_category(&id, category, ppi);
                                }
                            } else {
                                // New display
                                state.add_display_with_pos_and_profile(id, pos_key, category, ppi);
                            }
                            
                            let alpha = state.effective_alpha_u8(&id);
                            let _ = overlay::add_display(
                                &mut overlays,
                                event_loop_target,
                                &monitor,
                                &state,
                                state.is_visible(&id),
                                alpha,
                                &gpu
                            );

                            if let Some(overlay) = overlays.iter().find(|o| o.monitor_id == id) {
                                if state.panel_type(&id) == crate::display_config::PanelType::Unknown {
                                    state.set_panel_type(&id, overlay.panel_type);
                                }
                            }
                        }
                    }
                    
                    if let Some(ref t) = tray_handle {
                        t.rebuild_menu(&state, &overlays);
                    }
                    state.check_stealth_transition();
                    state.save();
                    needs_animation = calc_needs_animation(&overlays, &state);
                } else {
                    // IDs match: テーマ変更（ダークモード切替）の可能性があるため再描画する
                    overlay::sync_all(&mut overlays, &state, &gpu);
                    needs_animation = calc_needs_animation(&overlays, &state);
                    
                    // Don't rebuild_menu during benchmark as theme changes (Stealth modes) 
                    // will constantly close the menu.
                    if state.benchmark_progress.is_none() {
                        if let Some(ref t) = tray_handle {
                            t.rebuild_menu(&state, &overlays);
                        }
                    }
                }
            }
            _ => (),
        }

        // Handle Menu events (these usually trigger events themselves)
        if let Ok(menu_event) = menu_channel.try_recv() {
            let id = menu_event.id.0;
            if id == MENU_ID_GLOBAL_TOGGLE {
                println!("Menu: Toggle Global");
                state.toggle_global();
                state.save();
                overlay::sync_all(&mut overlays, &state, &gpu);
                needs_animation = calc_needs_animation(&overlays, &state);
                if let Some(ref t) = tray_handle {
                    t.rebuild_menu(&state, &overlays);
                }
            } else if let Some(id_str) = id.strip_prefix(MENU_ID_DISPLAY_TOGGLE_PREFIX) {
                if let Some(id_hex) = id_str.strip_prefix("0x") {
                    if let Ok(val) = u64::from_str_radix(id_hex, 16) {
                        let monitor_id = MonitorId(val);
                        println!("Menu: Toggle Display {:?}", monitor_id);
                        state.toggle_display(&monitor_id);
                        state.save();
                        overlay::sync_all(&mut overlays, &state, &gpu);
                        needs_animation = calc_needs_animation(&overlays, &state);
                        if let Some(ref t) = tray_handle {
                            t.rebuild_menu(&state, &overlays);
                        }
                    }
                }
            } else if let Some(rest) = id.strip_prefix(MENU_ID_ALPHA_PREFIX) {
                let parts: Vec<&str> = rest.split(':').collect();
                if parts.len() == 2 {
                    let id_str = parts[0];
                    let alpha_str = parts[1];
                    if let Some(id_hex) = id_str.strip_prefix("0x") {
                        if let Ok(val) = u64::from_str_radix(id_hex, 16) {
                            let monitor_id = MonitorId(val);
                            if let Ok(pct) = alpha_str.parse::<u32>() {
                                println!("Menu: Set Display {:?} Alpha {}%", monitor_id, pct);
                                state.set_display_alpha(&monitor_id, pct as f32 / 100.0);
                                state.save();
                                overlay::sync_all(&mut overlays, &state, &gpu);
                                needs_animation = calc_needs_animation(&overlays, &state);
                                if let Some(ref t) = tray_handle {
                                    t.rebuild_menu(&state, &overlays);
                                }
                            }
                        }
                    }
                }
            } else if let Some(rest) = id.strip_prefix(MENU_ID_MODE_PREFIX) {
                let parts: Vec<&str> = rest.split(':').collect();
                if parts.len() == 2 {
                    let id_str = parts[0];
                    let mode_str = parts[1];
                    if let Some(id_hex) = id_str.strip_prefix("0x") {
                        if let Ok(val) = u64::from_str_radix(id_hex, 16) {
                            let monitor_id = MonitorId(val);
                                let mode = match mode_str {
                                    "BlackLayer" => Some(FilterMode::BlackLayer),
                                    "VerticalLouver" => Some(FilterMode::VerticalLouver),
                                    "AIOcrInterference" => Some(FilterMode::AIOcrInterference),
                                    "HighIntensitySPD" => Some(FilterMode::HighIntensitySPD),
                                    "StealthDark" => Some(FilterMode::StealthDark),
                                    "StealthLight" => Some(FilterMode::StealthLight),
                                    "StealthLightSubpixel" => Some(FilterMode::StealthLightSubpixel),
                                    _ => None,
                                };
                            if let Some(m) = mode {
                                println!("Menu: Set Display {:?} Filter Mode {:?}", monitor_id, m);

                                state.set_filter_mode(&monitor_id, m);
                                state.save();
                                overlay::sync_all(&mut overlays, &state, &gpu);
                                needs_animation = calc_needs_animation(&overlays, &state);
                                 if let Some(ref t) = tray_handle {
                                     t.rebuild_menu(&state, &overlays);
                                 }
                             }
                         }
                     }
                 }
            } else if let Some(rest) = id.strip_prefix(MENU_ID_CATEGORY_PREFIX) {

                let parts: Vec<&str> = rest.split(':').collect();
                if parts.len() == 2 {
                    let id_str = parts[0];
                    let cat_str = parts[1];
                    if let Some(id_hex) = id_str.strip_prefix("0x") {
                        if let Ok(val) = u64::from_str_radix(id_hex, 16) {
                            let monitor_id = MonitorId(val);
                            let category = match cat_str {
                                "NotebookFhd" => Some(DisplayCategory::NotebookFhd),
                                "NotebookHiDpi" => Some(DisplayCategory::NotebookHiDpi),
                                "ExternalLarge4K" => Some(DisplayCategory::ExternalLarge4K),
                                "ExternalGeneral" => Some(DisplayCategory::ExternalGeneral),
                                _ => None,
                            };
                            if let Some(cat) = category {
                                println!("Menu: Set Display {:?} Category {:?}", monitor_id, cat);
                                let panel_type = state.panel_type(&monitor_id);
                                let profile = crate::display_config::DisplayProfile::from_config(cat, panel_type);
                                let existing_ppi = state.displays.get(&monitor_id)
                                    .map(|c| c.ppi)
                                    .filter(|&p| p > 0.0)
                                    .unwrap_or(profile.ppi);
                                state.set_display_category(&monitor_id, cat, existing_ppi);
                                state.save();
                                overlay::sync_all(&mut overlays, &state, &gpu);
                                needs_animation = calc_needs_animation(&overlays, &state);
                                if let Some(ref t) = tray_handle {
                                    t.rebuild_menu(&state, &overlays);
                                }
                            }
                        }
                    }
                }
            } else if let Some(rest) = id.strip_prefix(MENU_ID_PANEL_PREFIX) {
                let parts: Vec<&str> = rest.split(':').collect();
                if parts.len() == 2 {
                    let id_str = parts[0];
                    let panel_str = parts[1];
                    if let Some(id_hex) = id_str.strip_prefix("0x") {
                        if let Ok(val) = u64::from_str_radix(id_hex, 16) {
                            let monitor_id = MonitorId(val);
                            let panel_type = match panel_str {
                                "Unknown" => Some(crate::display_config::PanelType::Unknown),
                                "Oled" => Some(crate::display_config::PanelType::Oled),
                                "LcdIps" => Some(crate::display_config::PanelType::LcdIps),
                                "LcdTn" => Some(crate::display_config::PanelType::LcdTn),
                                _ => None,
                            };
                            if let Some(p) = panel_type {
                                println!("Menu: Set Display {:?} Panel Type {:?}", monitor_id, p);
                                state.set_panel_type(&monitor_id, p);
                                state.save();
                                overlay::sync_all(&mut overlays, &state, &gpu);
                                needs_animation = calc_needs_animation(&overlays, &state);
                                if let Some(ref t) = tray_handle {
                                    t.rebuild_menu(&state, &overlays);
                                }
                            }
                        }
                    }
                }
            } else if let Some(rest) = id.strip_prefix(MENU_ID_INTENSITY_PREFIX) {
                let parts: Vec<&str> = rest.split(':').collect();
                if parts.len() == 2 {
                    let id_str = parts[0];
                    let intensity_str = parts[1];
                    if let Some(id_hex) = id_str.strip_prefix("0x") {
                        if let Ok(val) = u64::from_str_radix(id_hex, 16) {
                            let monitor_id = MonitorId(val);
                            if let Ok(intensity) = intensity_str.parse::<f32>() {
                                println!("Menu: Set Display {:?} Filter Intensity {}", monitor_id, intensity);
                                state.set_filter_intensity(&monitor_id, intensity);
                                state.save();
                                overlay::sync_all(&mut overlays, &state, &gpu);
                                needs_animation = calc_needs_animation(&overlays, &state);
                                if let Some(ref t) = tray_handle {
                                    t.rebuild_menu(&state, &overlays);
                                }
                            }
                        }
                    }
                }
            } else if let Some(rest) = id.strip_prefix(MENU_ID_OVERRIDE_PERIOD_PREFIX) {
                let parts: Vec<&str> = rest.split(':').collect();
                if parts.len() == 2 {
                    let id_str = parts[0];
                    let val_str = parts[1];
                    if let Some(id_hex) = id_str.strip_prefix("0x") {
                        if let Ok(id_val) = u64::from_str_radix(id_hex, 16) {
                            let monitor_id = MonitorId(id_val);
                            let val = if val_str == "None" { None } else { val_str.parse::<f32>().ok() };
                            println!("Menu: Set Display {:?} Override Period {:?}", monitor_id, val);
                            state.set_override_period(&monitor_id, val);
                            state.save();
                            overlay::sync_all(&mut overlays, &state, &gpu);
                            needs_animation = calc_needs_animation(&overlays, &state);
                            if let Some(ref t) = tray_handle {
                                t.rebuild_menu(&state, &overlays);
                            }
                        }
                    }
                }
            } else if let Some(rest) = id.strip_prefix(MENU_ID_OVERRIDE_COVER_PREFIX) {
                let parts: Vec<&str> = rest.split(':').collect();
                if parts.len() == 2 {
                    let id_str = parts[0];
                    let val_str = parts[1];
                    if let Some(id_hex) = id_str.strip_prefix("0x") {
                        if let Ok(id_val) = u64::from_str_radix(id_hex, 16) {
                            let monitor_id = MonitorId(id_val);
                            let val = if val_str == "None" { None } else { val_str.parse::<f32>().ok() };
                            println!("Menu: Set Display {:?} Override Cover Ratio {:?}", monitor_id, val);
                            state.set_override_cover_ratio(&monitor_id, val);
                            state.save();
                            overlay::sync_all(&mut overlays, &state, &gpu);
                            needs_animation = calc_needs_animation(&overlays, &state);
                            if let Some(ref t) = tray_handle {
                                t.rebuild_menu(&state, &overlays);
                            }
                        }
                    }
                }
            } else if let Some(rest) = id.strip_prefix(MENU_ID_OVERRIDE_SPEED_PREFIX) {
                let parts: Vec<&str> = rest.split(':').collect();
                if parts.len() == 2 {
                    let id_str = parts[0];
                    let val_str = parts[1];
                    if let Some(id_hex) = id_str.strip_prefix("0x") {
                        if let Ok(id_val) = u64::from_str_radix(id_hex, 16) {
                            let monitor_id = MonitorId(id_val);
                            let val = if val_str == "None" { None } else { val_str.parse::<f32>().ok() };
                            println!("Menu: Set Display {:?} Override Scroll Speed {:?}", monitor_id, val);
                            state.set_override_scroll_speed(&monitor_id, val);
                            state.save();
                            overlay::sync_all(&mut overlays, &state, &gpu);
                            needs_animation = calc_needs_animation(&overlays, &state);
                            if let Some(ref t) = tray_handle {
                                t.rebuild_menu(&state, &overlays);
                            }
                        }
                    }
                }
            } else if let Some(rest) = id.strip_prefix(MENU_ID_RESET_RECOMMENDED) {
                if let Some(id_hex) = rest.strip_prefix("0x") {
                    if let Ok(val) = u64::from_str_radix(id_hex, 16) {
                        let monitor_id = MonitorId(val);
                        println!("Menu: Reset Display {:?} to Recommended Settings", monitor_id);
                        state.reset_to_recommended(&monitor_id);
                        state.save();
                        overlay::sync_all(&mut overlays, &state, &gpu);
                        needs_animation = calc_needs_animation(&overlays, &state);
                        if let Some(ref t) = tray_handle {
                            t.rebuild_menu(&state, &overlays);
                        }
                    }
                }
            } else if id == MENU_ID_AI_DETECTION {
                println!("Menu: Toggle AI Detection");
                let enabled = state.toggle_ai_detection();
                if enabled {
                    let _ = ai_cmd_tx.send(AIDetectionCommand::Start);
                } else {
                    let _ = ai_cmd_tx.send(AIDetectionCommand::Stop);
                }
                state.save();
                if let Some(ref t) = tray_handle {
                    t.rebuild_menu(&state, &overlays);
                }
            } else if let Some(name) = id.strip_prefix("preset_all:") {
                println!("Menu: Apply Preset {} to all displays", name);
                state.apply_preset_to_all(name);
                overlay::sync_all(&mut overlays, &state, &gpu);
                needs_animation = calc_needs_animation(&overlays, &state);
                if let Some(ref t) = tray_handle {
                    t.rebuild_menu(&state, &overlays);
                }
            } else if let Some(rest) = id.strip_prefix(MENU_ID_PRESET_APPLY_PREFIX) {
                let parts: Vec<&str> = rest.split(':').collect();
                if parts.len() == 2 {
                    let id_str = parts[0];
                    let name = parts[1];
                    if let Some(id_hex) = id_str.strip_prefix("0x") {
                        if let Ok(val) = u64::from_str_radix(id_hex, 16) {
                            let monitor_id = MonitorId(val);
                            println!("Menu: Apply Preset {} to Display {:?}", name, monitor_id);
                            state.apply_preset(name, &monitor_id);
                            overlay::sync_all(&mut overlays, &state, &gpu);
                            needs_animation = calc_needs_animation(&overlays, &state);
                            if let Some(ref t) = tray_handle {
                                t.rebuild_menu(&state, &overlays);
                            }
                        }
                    }
                }
            } else if let Some(name) = id.strip_prefix(MENU_ID_PRESET_DELETE_PREFIX) {
                println!("Menu: Delete Preset {}", name);
                state.delete_preset(name);
                if let Some(ref t) = tray_handle {
                    t.rebuild_menu(&state, &overlays);
                }
            } else if id == MENU_ID_PRESET_SAVE_CURRENT {
                let name = format!("Preset {}", state.presets.len() + 1);
                println!("Menu: Save Current as {}", name);
                // Use the first display's settings as the preset
                if let Some(first_config) = state.displays.values().next() {
                    let settings = first_config.get_settings();
                    state.save_preset(name, settings);
                }
                if let Some(ref t) = tray_handle {
                    t.rebuild_menu(&state, &overlays);
                }
            } else if id == MENU_ID_PRESET_CLEAR_ALL {
                println!("Menu: Clear All Presets");
                state.clear_presets();
                if let Some(ref t) = tray_handle {
                    t.rebuild_menu(&state, &overlays);
                }
            } else if id == MENU_ID_RUN_BENCHMARK {
                let _ = proxy.send_event(UserEvent::RunBenchmark);
            } else if id == MENU_ID_AUTO_START {
                println!("Menu: Toggle Auto Start");
                let enabled = state.toggle_auto_start();
                if let Some(ref al) = auto_launcher {
                    if enabled {
                        let _ = al.enable();
                    } else {
                        let _ = al.disable();
                    }
                }
                state.save();
                if let Some(ref t) = tray_handle {
                    t.rebuild_menu(&state, &overlays);
                }
            } else if id == MENU_ID_QUIT {
                println!("Menu: Quit");
                state.restore_os_settings();
                *control_flow = ControlFlow::Exit;
            }
        }

        if let Event::WindowEvent {
                event: tao::event::WindowEvent::CloseRequested,
                ..
            } = event {
            state.restore_os_settings();
            *control_flow = ControlFlow::Exit;
        }
    });
}

fn calc_needs_animation(overlays: &[overlay::OverlayWindow], state: &AppState) -> bool {
    overlays.iter().any(|o| {
        if !state.is_visible(&o.monitor_id) {
            return false;
        }
        let mode = state.filter_mode(&o.monitor_id);
        if matches!(mode, FilterMode::BlackLayer) {
            return false;
        }

        // DisplayConfig から実効プロファイルを取得してアニメーションが必要か判定
        if let Some(config) = state.displays.get(&o.monitor_id) {
            let profile = config.get_effective_profile();
            // スクロール速度があるか、位相反転（FastVibration）が有効ならアニメーションが必要
            profile.scroll_speed_mm_per_sec.abs() > 0.001 || profile.phase_flip_hz > 0.001
        } else {
            false
        }
    })
}
