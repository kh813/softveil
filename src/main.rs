#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod display_config;
mod overlay;
mod platform;
mod single_instance;
mod tray;
mod hotkey;
mod ai_detection;

use app::{AppState, FilterMode};
use display_config::{MonitorId, DisplayConfig, DisconnectedCache};
use tray::{TrayHandle, MENU_ID_GLOBAL_TOGGLE, MENU_ID_DISPLAY_TOGGLE_PREFIX, MENU_ID_ALPHA_PREFIX, MENU_ID_MODE_PREFIX, MENU_ID_AUTO_START, MENU_ID_AI_DETECTION, MENU_ID_QUIT};
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
    
    // Initial display registration
    for monitor in &monitors {
        let id = MonitorId::from_monitor(monitor);
        let pos_key = DisplayConfig::make_position_key(monitor.position(), monitor.size());
        println!("Registering monitor: {:?} with pos_key: {}", id, pos_key);
        state.add_display_with_pos(id, pos_key);
    }
    
    let mut overlays = overlay::create_all(&event_loop, monitors, &state);
    println!("Created {} overlay windows.", overlays.len());
    
    // Sync initial overlays with loaded state
    overlay::sync_all(&mut overlays, &state);
    
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

    event_loop.run(move |event, event_loop_target, control_flow| {
        *control_flow = ControlFlow::Wait;

        // Handle Hotkey events via UserEvent
        match event {
            Event::UserEvent(UserEvent::Hotkey(HotkeyEvent::ToggleGlobal)) => {
                println!("Event: Toggle Global (Hotkey)");
                state.toggle_global();
                state.save();
                overlay::sync_all(&mut overlays, &state);
                if let Some(ref t) = tray_handle {
                    t.rebuild_menu(&state, &overlays);
                }
            }
            Event::UserEvent(UserEvent::AIDetected(detected)) => {
                if state.ai_peeper_detected != detected {
                    println!("Event: AI Peeper Detected = {}", detected);
                    state.set_peeper_detected(detected);
                    overlay::sync_all(&mut overlays, &state);
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
                overlay::sync_all(&mut overlays, &state);
                if let Some(ref t) = tray_handle {
                    t.rebuild_menu(&state, &overlays);
                }
            } else if id == MENU_ID_QUIT {
                println!("Menu: Quit");
                *control_flow = ControlFlow::Exit;
            } else if id.starts_with(MENU_ID_DISPLAY_TOGGLE_PREFIX) {
                let id_str = &id[MENU_ID_DISPLAY_TOGGLE_PREFIX.len()..];
                if id_str.starts_with("0x") {
                    if let Ok(val) = u64::from_str_radix(&id_str[2..], 16) {
                        let monitor_id = MonitorId(val);
                        println!("Menu: Toggle Display {:?}", monitor_id);
                        state.toggle_display(&monitor_id);
                        state.save();
                        overlay::sync_all(&mut overlays, &state);
                        if let Some(ref t) = tray_handle {
                            t.rebuild_menu(&state, &overlays);
                        }
                    }
                }
            } else if id.starts_with(MENU_ID_ALPHA_PREFIX) {
                let alpha_str = &id[MENU_ID_ALPHA_PREFIX.len()..];
                if let Ok(pct) = alpha_str.parse::<u32>() {
                    println!("Menu: Set Alpha {}%", pct);
                    state.set_global_alpha(pct as f32 / 100.0);
                    state.save();
                    overlay::sync_all(&mut overlays, &state);
                    if let Some(ref t) = tray_handle {
                        t.rebuild_menu(&state, &overlays);
                    }
                }
            } else if id.starts_with(MENU_ID_MODE_PREFIX) {
                let mode_str = &id[MENU_ID_MODE_PREFIX.len()..];
                let mode = match mode_str {
                    "BlackLayer" => Some(FilterMode::BlackLayer),
                    "Louver" => Some(FilterMode::Louver),
                    _ => None,
                };
                if let Some(m) = mode {
                    println!("Menu: Set Filter Mode {:?}", m);
                    state.set_filter_mode(m);
                    state.save();
                    overlay::sync_all(&mut overlays, &state);
                    if let Some(ref t) = tray_handle {
                        t.rebuild_menu(&state, &overlays);
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

        // Handle Hotplug events
        if let Ok(_) = display_change_rx.try_recv() {
            let current_monitors: Vec<_> = event_loop_target.available_monitors().collect();
            let current_ids: Vec<MonitorId> = current_monitors.iter().map(MonitorId::from_monitor).collect();
            
            let mut existing_ids: Vec<MonitorId> = overlays.iter().map(|o| o.monitor_id).collect();
            existing_ids.sort();
            let mut new_ids = current_ids.clone();
            new_ids.sort();

            if existing_ids != new_ids {
                println!("Display configuration changed. Recalculating...");
                
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
                        println!("Adding display: {:?}", id);
                        let pos_key = DisplayConfig::make_position_key(monitor.position(), monitor.size());
                        let config = cache.restore(&pos_key).unwrap_or_else(|| {
                            let mut c = DisplayConfig::default();
                            c.position_key = pos_key;
                            c
                        });
                        
                        state.add_display(id, Some(config.clone()));
                        let _ = overlay::add_display(
                            &mut overlays,
                            event_loop_target,
                            &monitor,
                            &state,
                            state.is_visible(&id),
                            config.alpha_u8()
                        );
                    }
                }
                
                if let Some(ref t) = tray_handle {
                    t.rebuild_menu(&state, &overlays);
                }
                state.save();
            }
        }

        match event {
            Event::WindowEvent {
                event: tao::event::WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            _ => (),
        }
    });
}
