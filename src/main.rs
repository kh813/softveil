#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod display_config;
mod overlay;
mod platform;
mod single_instance;
mod tray;
mod hotkey;
mod ai_detection;

use app::AppState;
use display_config::{MonitorId, DisplayConfig, DisconnectedCache, FilterMode, DisplayCategory};
use tray::{TrayHandle, MENU_ID_GLOBAL_TOGGLE, MENU_ID_DISPLAY_TOGGLE_PREFIX, MENU_ID_ALPHA_PREFIX, MENU_ID_MODE_PREFIX, MENU_ID_PANEL_PREFIX, MENU_ID_CATEGORY_PREFIX, MENU_ID_INTENSITY_PREFIX, MENU_ID_AUTO_START, MENU_ID_AI_DETECTION, MENU_ID_QUIT};
use ai_detection::{AIDetectionCommand, start_detection_thread};
use auto_launch::AutoLaunchBuilder;
use hotkey::HotkeyEvent;
use tao::event_loop::ControlFlow;
use tao::event::Event;
use muda::MenuEvent;
use std::sync::mpsc;

#[derive(Debug)]
enum UserEvent {
    Hotkey(HotkeyEvent),
    AIDetected(bool),
    DisplayChange,
}

fn main() {
    println!("Starting Softveil...");
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

    // Phase 5: Check Screen Capture access (without automatic prompt)
    #[cfg(target_os = "macos")]
    {
        if !platform::has_screen_capture_access() {
            println!("Screen capture access is not granted. Advanced Phase 5 features will be limited.");
            // We could show the dialog here, but to be even less intrusive, 
            // maybe we only show it when a feature is selected.
            // For now, let's keep it in console to satisfy "not every time" 
            // and prepare for feature-gated prompts.
        }
    }

    let gpu = std::sync::Arc::new(pollster::block_on(overlay::GpuContext::new()).expect("Failed to initialize GPU"));
    
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
    overlay::sync_all(&mut overlays, &state, &gpu);
    
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
    let _hotkey_guard = hotkey::register(hotkey_tx).expect("Failed to register hotkey");

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

    let app_path = std::env::current_exe().expect("Failed to get current exe path");
    let auto_launcher = AutoLaunchBuilder::new()
        .set_app_name("Softveil")
        .set_app_path(&app_path.to_string_lossy())
        .set_use_launch_agent(true) // For macOS
        .build()
        .expect("Failed to create auto-launcher");

    // Sync auto-launch state with config on startup
    if state.auto_start {
        let _ = auto_launcher.enable();
    } else {
        let _ = auto_launcher.disable();
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

    println!("Softveil Phase 1 running. Waiting for events...");

    let mut needs_animation = calc_needs_animation(&overlays, &state);

    event_loop.run(move |event, event_loop_target, control_flow| {
        *control_flow = if needs_animation {
            ControlFlow::Poll
        } else {
            ControlFlow::Wait
        };

        match event {
            Event::MainEventsCleared
                if needs_animation => {
                    for overlay in overlays.iter_mut() {
                        if state.is_visible(&overlay.monitor_id) {
                            let alpha = state.effective_alpha_u8(&overlay.monitor_id);
                            let _ = overlay.draw(&gpu, &state, alpha);
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
                    state.save();
                    needs_animation = calc_needs_animation(&overlays, &state);
                } else {
                    // IDs match, but we might need to refresh names after prefetch
                    if let Some(ref t) = tray_handle {
                        t.rebuild_menu(&state, &overlays);
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
            } else if id == MENU_ID_QUIT {
                println!("Menu: Quit");
                *control_flow = ControlFlow::Exit;
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
                                _ => None,
                            };
                            if let Some(m) = mode {
                                println!("Menu: Set Display {:?} Filter Mode {:?}", monitor_id, m);

                                #[cfg(target_os = "macos")]
                                {
                                    // Check if we need screen capture permission for the selected mode.
                                    // Even though AI OCR Interference is currently procedural, it's marked as a Phase 5 feature 
                                    // that will eventually include Semantic Privacy analysis.
                                    if m == FilterMode::AIOcrInterference && !platform::has_screen_capture_access() {
                                        platform::macos::show_permission_alert(
                                            "「画面収録」の許可が必要です",
                                            "AI OCR 妨害機能の一部（高度な解析）を利用するには、システム設定の「プライバシーとセキュリティ > 画面収録」で Softveil を許可してください。"
                                        );
                                    }
                                }

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
                                let profile = crate::display_config::DisplayProfile::from_category(cat);
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
            } else if id == MENU_ID_AUTO_START {
                println!("Menu: Toggle Auto Start");
                let enabled = state.toggle_auto_start();
                if enabled {
                    let _ = auto_launcher.enable();
                } else {
                    let _ = auto_launcher.disable();
                }
                state.save();
                if let Some(ref t) = tray_handle {
                    t.rebuild_menu(&state, &overlays);
                }
            }
        }

        if let Event::WindowEvent {
                event: tao::event::WindowEvent::CloseRequested,
                ..
            } = event {
            *control_flow = ControlFlow::Exit;
        }
    });
}

fn calc_needs_animation(overlays: &[overlay::OverlayWindow], state: &AppState) -> bool {
    overlays.iter().any(|o| {
        let mode = state.filter_mode(&o.monitor_id);
        matches!(mode,
            FilterMode::VerticalLouver |
            FilterMode::HighIntensitySPD |
            FilterMode::AIOcrInterference
        ) && state.is_visible(&o.monitor_id)
    })
}
