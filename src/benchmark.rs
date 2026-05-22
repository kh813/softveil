use crate::app::AppState;
use crate::display_config::{FilterMode, MonitorId};
use crate::overlay::{GpuContext, OverlayWindow};
use std::sync::Arc;
use std::time::Duration;
use image::{DynamicImage, GenericImageView, RgbaImage};
use std::fs;
use std::io::{self, Write};

pub fn run_benchmark(gpu: Arc<GpuContext>, mut state: AppState, overlays: &mut Vec<OverlayWindow>) {
    println!("Starting Lightweight Mechanical Benchmark...");
    
    let results_dir = "benchmark_results";
    if let Err(e) = fs::create_dir_all(results_dir) {
        eprintln!("Failed to create results directory: {:?}", e);
        return;
    }

    let monitor_ids: Vec<MonitorId> = overlays.iter().map(|o| o.monitor_id).collect();
    if monitor_ids.is_empty() {
        println!("No monitors found for benchmark.");
        return;
    }
    
    let filter_modes = [
        FilterMode::BlackLayer,
        FilterMode::VerticalLouver,
        FilterMode::AIOcrInterference,
        FilterMode::HighIntensitySPD,
        FilterMode::StealthDark,
        FilterMode::StealthLight,
        FilterMode::StealthLightSubpixel,
    ];
    
    // More realistic Alpha range for evaluation
    let alphas = [0.1, 0.3, 0.5];
    
    let mut report = String::from("# Softveil Mechanical Benchmark Report (Optimized)\n\n");
    report.push_str("Note: Analysis performed on 1/2 resolution for speed. Images saved as JPEG.\n\n");

    let overlay_count = overlays.len();
    for i in 0..overlay_count {
        let (test_monitor_id, monitor_name) = {
            let overlay = &overlays[i];
            (overlay.monitor_id, overlay.monitor_name.clone())
        };
        
        println!("\n--- Testing Monitor: {} ({:?}) ---", monitor_name, test_monitor_id);
        report.push_str(&format!("## Monitor: {}\n\n", monitor_name));
        report.push_str("| Mode | Alpha | Contrast Reduction | Obfuscation Score | Image |\n");
        report.push_str("| :--- | :--- | :--- | :--- | :--- |\n");

        for mode in &filter_modes {
            for alpha in &alphas {
                print!("Testing Mode: {:?}, Alpha: {:.1}...", mode, alpha);
                io::stdout().flush().unwrap();
                
                state.set_filter_mode(&test_monitor_id, *mode);
                state.set_display_alpha(&test_monitor_id, *alpha);
                
                crate::overlay::sync_all(overlays, &state, &gpu);
                
                #[cfg(target_os = "windows")]
                pump_messages();

                let wait_ms = if matches!(mode, FilterMode::StealthDark | FilterMode::StealthLight) { 2000 } else { 800 };
                std::thread::sleep(Duration::from_millis(wait_ms));
                
                #[cfg(target_os = "windows")]
                pump_messages();

                // Capture screenshot for SPECIFIC monitor
                print!(" Capturing...");
                io::stdout().flush().unwrap();
                let mut img = match crate::platform::capture_display(&test_monitor_id) {
                    Ok(i) => i,
                    Err(_e) => {
                        println!(" Error!");
                        continue;
                    }
                };
                
                // SCALE DOWN for fast processing in debug builds
                let (w, h) = img.dimensions();
                img = img.thumbnail(w / 2, h / 2);
                
                // Analyze image in memory (Fast)
                print!(" Analyzing...");
                io::stdout().flush().unwrap();
                let (crr, obfuscation) = analyze_privacy_effect(&img);
                
                // Save simulation as JPEG (much faster than PNG)
                print!(" Simulating Oblique...");
                io::stdout().flush().unwrap();
                let simulated_path = format!("{}/simulated_{:x}_{:?}_{:.1}.jpg", results_dir, test_monitor_id.0, mode, alpha);
                simulate_oblique_view_to_jpg(&img, &simulated_path);

                println!(" Done. (Score: {:.2})", obfuscation);

                report.push_str(&format!(
                    "| {:?} | {:.1} | {:.2}% | {:.2} | [Link]({}) |\n",
                    mode, alpha, crr * 100.0, obfuscation, simulated_path
                ));
                
                #[cfg(target_os = "windows")]
                pump_messages();
            }
        }
        
        report.push_str("\n");

        // Phase 10: Auto-Optimization per monitor
        println!("Running Auto-Optimization search for {}...", monitor_name);
        if let Some((opt_period, opt_cover)) = find_optimal_params(gpu.clone(), state.clone(), overlays, test_monitor_id) {
            println!("\n[Optimization Result - {}]", monitor_name);
            println!("Optimal Period: {:.2}mm", opt_period);
            println!("Optimal Cover Ratio: {:.0}%", opt_cover * 100.0);
            
            // Phase 11: Propose recommended presets with monitor name
            propose_recommended_presets_for_monitor(&mut state, opt_period, opt_cover, &monitor_name);
        }
    }

    fs::write("BENCHMARK_REPORT.md", report).expect("Failed to write report");
}

pub fn propose_recommended_presets_for_monitor(state: &mut AppState, opt_period: f32, opt_cover: f32, monitor_name: &str) {
    use crate::display_config::FilterSettings;
    
    println!("Adding recommended presets for {}...", monitor_name);
    
    let presets = [
        ("Office (Balanced)", FilterMode::StealthLight, 0.3, 1.0, Some(0.0)),
        ("Transit (Maximum)", FilterMode::HighIntensitySPD, 0.5, 0.8, Some(10.0)),
        ("Night (Stealth Dark)", FilterMode::StealthDark, 0.3, 1.0, Some(0.0)),
    ];

    for (base_name, mode, alpha, intensity, speed) in presets {
        let name = format!("{} - {}", base_name, monitor_name);
        let period = if base_name.contains("Transit") { opt_period * 0.75 } else { opt_period };
        let cover = if base_name.contains("Transit") { opt_cover.max(0.85) } else { opt_cover };

        state.save_preset(name, FilterSettings {
            alpha,
            filter_mode: mode,
            filter_intensity: intensity,
            override_period_mm: Some(period),
            override_cover_ratio: Some(cover),
            override_scroll_speed: speed,
        });
    }

    #[cfg(target_os = "windows")]
    crate::platform::show_error_dialog(
        "最適化完了 / Optimization Complete",
        &format!("モニター「{}」に最適な3つのプリセットを自動生成しました。\n設定プリセットメニューから適用可能です。", monitor_name)
    );
}

pub fn find_optimal_params(
    gpu: Arc<GpuContext>,
    mut state: AppState,
    overlays: &mut Vec<OverlayWindow>,
    monitor_id: MonitorId,
) -> Option<(f32, f32)> {
    let periods = [0.15, 0.20, 0.30, 0.40];
    let covers = [0.50, 0.70, 0.85];
    
    let mut best_score = -1.0;
    let mut best_params = None;

    state.set_filter_mode(&monitor_id, FilterMode::HighIntensitySPD);
    state.set_display_alpha(&monitor_id, 0.3);

    for &p in &periods {
        for &c in &covers {
            state.set_override_period(&monitor_id, Some(p));
            state.set_override_cover_ratio(&monitor_id, Some(c));
            
            crate::overlay::sync_all(overlays, &state, &gpu);
            #[cfg(target_os = "windows")]
            pump_messages();
            
            std::thread::sleep(Duration::from_millis(500));
            
            if let Ok(img) = crate::platform::capture_display(&monitor_id) {
                let (_, score) = analyze_privacy_effect(&img);
                if score > best_score {
                    best_score = score;
                    best_params = Some((p, c));
                }
            }
        }
    }
    
    best_params
}

#[cfg(target_os = "windows")]
fn pump_messages() {
    use windows_sys::Win32::UI::WindowsAndMessaging::*;
    unsafe {
        let mut msg: MSG = std::mem::zeroed();
        while PeekMessageW(&mut msg, std::ptr::null_mut(), 0, 0, PM_REMOVE) != 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}

fn analyze_privacy_effect(img: &DynamicImage) -> (f32, f32) {
    let luma = img.to_luma8();
    let (w, h) = luma.dimensions();
    let raw = luma.as_raw();
    
    let mut min_luma = 255u8;
    let mut max_luma = 0u8;
    
    for &val in raw {
        if val < min_luma { min_luma = val; }
        if val > max_luma { max_luma = val; }
    }
    
    let contrast = (max_luma as f32 - min_luma as f32) / 255.0;
    
    let mut edge_sum = 0.0;
    if w > 2 && h > 2 {
        let stride = w as usize;
        for y in 1..(h as usize - 1) {
            let row_offset = y * stride;
            for x in 1..(w as usize - 1) {
                let idx = row_offset + x;
                let center = raw[idx] as f32;
                let surrounding = (
                    raw[idx - 1] as f32 +
                    raw[idx + 1] as f32 +
                    raw[idx - stride] as f32 +
                    raw[idx + stride] as f32
                ) * 0.25;
                edge_sum += (center - surrounding).abs();
            }
        }
    }
    
    let edge_density = edge_sum / (w * h) as f32;
    let crr = 1.0 - contrast; 
    let obfuscation = edge_density;

    (crr, obfuscation)
}

pub fn run_benchmark_threaded(
    monitor_info: Vec<(MonitorId, String)>,
    cmd_tx: std::sync::mpsc::Sender<crate::BenchmarkCommand>,
    resp_rx: std::sync::mpsc::Receiver<()>,
    original_settings: std::collections::HashMap<MonitorId, crate::display_config::FilterSettings>,
    proxy: tao::event_loop::EventLoopProxy<crate::UserEvent>,
) {
    println!("Starting Optimized Threaded Mechanical Benchmark (Batch Mode)...");

    let results_dir = "benchmark_results";
    let _ = fs::create_dir_all(results_dir);

    // Helper to send command and wake up main loop
    let send_cmd = |cmd: crate::BenchmarkCommand| {
        let _ = cmd_tx.send(cmd);
        let _ = proxy.send_event(crate::UserEvent::ProcessBenchmarkCommand);
    };

    let filter_modes = [
        FilterMode::BlackLayer,
        FilterMode::VerticalLouver,
        FilterMode::AIOcrInterference,
        FilterMode::HighIntensitySPD,
        FilterMode::StealthDark,
        FilterMode::StealthLight,
        FilterMode::StealthLightSubpixel,
    ];
    let alphas = [0.1, 0.3, 0.5];
    
    let mut monitor_stats = std::collections::HashMap::new();
    for (id, name) in &monitor_info {
        monitor_stats.insert(*id, (name.clone(), -1.0, String::from("Unknown")));
    }

    let total_steps = filter_modes.len() * alphas.len() + (4 * 3); // Test matrix + opt search
    let mut current_step = 0;

    // --- Part 1: Test Matrix (Batched) ---
    for mode in &filter_modes {
        for alpha in &alphas {
            current_step += 1;
            let progress = current_step as f32 / total_steps as f32;
            send_cmd(crate::BenchmarkCommand::Progress(
                progress,
                format!("全モニターを測定中... ({:?} / Alpha {:.1})", mode, alpha)
            ));

            // Set settings for ALL monitors
            let mut batch = Vec::new();
            for (id, _) in &monitor_info {
                batch.push((*id, *mode, *alpha, None, None, None));
            }
            send_cmd(crate::BenchmarkCommand::SetBatchSettings(batch));
            let _ = resp_rx.recv();

            let wait_ms = if matches!(mode, FilterMode::StealthDark | FilterMode::StealthLight) { 2000 } else { 800 };
            std::thread::sleep(Duration::from_millis(wait_ms));

            // Capture ALL monitors
            let (tx, rx) = std::sync::mpsc::channel();
            let monitor_ids: Vec<MonitorId> = monitor_info.iter().map(|(id, _)| *id).collect();
            send_cmd(crate::BenchmarkCommand::CaptureBatch(monitor_ids, tx));
            
            if let Ok(results) = rx.recv() {
                // Process results in parallel (simple threading for each monitor)
                let mut handles = Vec::new();
                for (id, img_res) in results {
                    if let Ok(img) = img_res {
                        let mode = *mode;
                        let alpha = *alpha;
                        let results_dir = results_dir.to_string(); // move to thread
                        let handle = std::thread::spawn(move || {
                            let (w, h) = img.dimensions();
                            let small_img = img.thumbnail(w / 2, h / 2);
                            let (_, obfuscation) = analyze_privacy_effect(&small_img);
                            
                            let simulated_path = format!("{}/simulated_{:x}_{:?}_{:.1}.jpg", results_dir, id.0, mode, alpha);
                            simulate_oblique_view_to_jpg(&small_img, &simulated_path);
                            (id, obfuscation, format!("{:?} (Alpha {:.1})", mode, alpha))
                        });
                        handles.push(handle);
                    }
                }

                for h in handles {
                    if let Ok((id, score, label)) = h.join() {
                        if let Some(stats) = monitor_stats.get_mut(&id) {
                            if score > stats.1 {
                                stats.1 = score;
                                stats.2 = label;
                            }
                        }
                    }
                }
            }
        }
    }
    
    // --- Part 2: Auto-Optimization Search (Batched) ---
    println!("Running Batched Auto-Optimization search...");
    let periods = [0.15, 0.20, 0.30, 0.40];
    let covers = [0.50, 0.70, 0.85];
    
    let mut best_params = std::collections::HashMap::new();
    let mut best_params_scores = std::collections::HashMap::new();
    for (id, _) in &monitor_info {
        best_params_scores.insert(*id, -1.0f32);
    }

    for &p in &periods {
        for &c in &covers {
            current_step += 1;
            let progress = current_step as f32 / total_steps as f32;
            send_cmd(crate::BenchmarkCommand::Progress(
                progress,
                format!("全モニターの最適化パラメータを探索中... (P={:.2} C={:.0}%)", p, c * 100.0)
            ));

            let mut batch = Vec::new();
            for (id, _) in &monitor_info {
                batch.push((*id, FilterMode::HighIntensitySPD, 0.3, Some(p), Some(c), None));
            }
            send_cmd(crate::BenchmarkCommand::SetBatchSettings(batch));
            let _ = resp_rx.recv();
            std::thread::sleep(Duration::from_millis(500));
            
            let (tx, rx) = std::sync::mpsc::channel();
            let monitor_ids: Vec<MonitorId> = monitor_info.iter().map(|(id, _)| *id).collect();
            send_cmd(crate::BenchmarkCommand::CaptureBatch(monitor_ids, tx));
            
            if let Ok(results) = rx.recv() {
                for (id, img_res) in results {
                    if let Ok(img) = img_res {
                        let (_, score) = analyze_privacy_effect(&img);
                        if let Some(best_score) = best_params_scores.get_mut(&id) {
                            if score > *best_score {
                                *best_score = score;
                                best_params.insert(id, (p, c));
                            }
                        }
                    }
                }
            }
        }
    }

    let mut all_new_presets = Vec::new();
    let mut results_summary = String::new();

    for (monitor_id, monitor_name) in monitor_info {
        if let Some((opt_period, opt_cover)) = best_params.get(&monitor_id) {
            let presets = generate_recommended_presets_for_monitor(*opt_period, *opt_cover, &monitor_name);
            all_new_presets.extend(presets);
            
            if let Some(stats) = monitor_stats.get(&monitor_id) {
                results_summary.push_str(&format!("▼ {}\n", monitor_name));
                results_summary.push_str(&format!(
                    "  ・最高秘匿スコア: {:.2} ({})\n", stats.1, stats.2
                ));
                results_summary.push_str(&format!(
                    "  ・最適パラメータ: 周期 {:.2}mm / 遮蔽率 {:.0}%\n\n", opt_period, opt_cover * 100.0
                ));
            }
        }
    }

    // Restore original settings
    for (id, settings) in original_settings {
        send_cmd(crate::BenchmarkCommand::SetTestSettings(
            id, settings.filter_mode, settings.alpha, settings.override_period_mm, settings.override_cover_ratio, settings.override_scroll_speed
        ));
        let _ = resp_rx.recv();
    }

    send_cmd(crate::BenchmarkCommand::Finished(all_new_presets, results_summary));
}


fn generate_recommended_presets_for_monitor(opt_period: f32, opt_cover: f32, monitor_name: &str) -> Vec<crate::display_config::Preset> {
    use crate::display_config::{FilterSettings, Preset};
    
    let base_presets = [
        ("Office (Balanced)", FilterMode::StealthLight, 0.3, 1.0, Some(0.0)),
        ("Transit (Maximum)", FilterMode::HighIntensitySPD, 0.5, 0.8, Some(10.0)),
        ("Night (Stealth Dark)", FilterMode::StealthDark, 0.3, 1.0, Some(0.0)),
    ];

    let mut presets = Vec::new();
    for (base_name, mode, alpha, intensity, speed) in base_presets {
        let name = format!("{} - {}", base_name, monitor_name);
        let period = if base_name.contains("Transit") { opt_period * 0.75 } else { opt_period };
        let cover = if base_name.contains("Transit") { opt_cover.max(0.85) } else { opt_cover };

        presets.push(Preset {
            name,
            settings: FilterSettings {
                alpha,
                filter_mode: mode,
                filter_intensity: intensity,
                override_period_mm: Some(period),
                override_cover_ratio: Some(cover),
                override_scroll_speed: speed,
            },
        });
    }
    presets
}

fn simulate_oblique_view_to_jpg(img: &DynamicImage, output_path: &str) {
    let (w, h) = img.dimensions();
    let new_w = (w as f32 * 0.707) as u32;
    
    let rgba = img.to_rgba8();
    let input_raw = rgba.as_raw();
    let mut simulated = RgbaImage::new(new_w, h);
    
    let output_raw = simulated.as_mut();
    
    for y in 0..h {
        let in_row_offset = y * w * 4;
        let out_row_offset = y * new_w * 4;
        for x in 0..new_w {
            let orig_x = (x as f32 / 0.707) as usize;
            if (orig_x as u32) < w {
                let in_idx = in_row_offset as usize + orig_x * 4;
                let out_idx = (out_row_offset as usize + x as usize * 4) as usize;
                
                for i in 0..3 {
                    output_raw[out_idx + i] = ((input_raw[in_idx + i] as f32 * 0.8) + 40.0).clamp(0.0, 255.0) as u8;
                }
                output_raw[out_idx + 3] = 255;
            }
        }
    }
    
    // Save as JPEG with 80% quality for extreme speed in debug mode
    let rgb = DynamicImage::ImageRgba8(simulated).to_rgb8();
    rgb.save_with_format(output_path, image::ImageFormat::Jpeg).expect("Failed to save JPEG");
}

