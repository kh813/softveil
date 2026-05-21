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
    
    let test_monitor_id = monitor_ids[0];
    
    let filter_modes = [
        FilterMode::BlackLayer,
        FilterMode::VerticalLouver,
        FilterMode::AIOcrInterference,
        FilterMode::HighIntensitySPD,
        FilterMode::StealthDark,
        FilterMode::StealthLight,
    ];
    
    // More realistic Alpha range for evaluation
    let alphas = [0.1, 0.3, 0.5];
    
    let mut report = String::from("# Softveil Mechanical Benchmark Report (Optimized)\n\n");
    report.push_str("Note: Analysis performed on 1/2 resolution for speed. Images saved as JPEG.\n\n");
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

            // Capture screenshot into memory
            print!(" Capturing...");
            io::stdout().flush().unwrap();
            let mut img = match crate::platform::capture_primary_display() {
                Ok(i) => i,
                Err(e) => {
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
            let simulated_path = format!("{}/simulated_{:?}_{:.1}.jpg", results_dir, mode, alpha);
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

    fs::write("BENCHMARK_REPORT.md", report).expect("Failed to write report");
    println!("\nBenchmark complete. Report saved to BENCHMARK_REPORT.md");
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
