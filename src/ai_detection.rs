use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use nokhwa::pixel_format::RgbFormat;
use nokhwa::utils::{CameraIndex, RequestedFormat, RequestedFormatType};
use nokhwa::Camera;
use tract_onnx::prelude::*;

pub enum AIDetectionCommand {
    Start,
    Stop,
}

pub fn start_detection_thread(
    command_rx: mpsc::Receiver<AIDetectionCommand>,
    event_tx: mpsc::Sender<bool>,
) {
    thread::spawn(move || {
        let mut camera: Option<Camera> = None;
        let mut running = false;

        // Load AI model from a relative path to the executable or resources
        let exe_path = std::env::current_exe().unwrap_or_default();
        let exe_dir = exe_path.parent().unwrap_or(std::path::Path::new("."));
        
        #[cfg(target_os = "macos")]
        let model_path = exe_dir.parent().unwrap_or(exe_dir).join("Resources").join("face_detector.onnx");
        
        #[cfg(not(target_os = "macos"))]
        let model_path = exe_dir.join("face_detector.onnx");

        // Fallback to local assets during development
        let model_path = if model_path.exists() {
            model_path
        } else {
            std::path::PathBuf::from("assets/face_detector.onnx")
        };

        let model = match onnx()
            .model_for_path(&model_path)
            .and_then(|m| m.with_input_fact(0, f32::fact(&[1, 3, 240, 320]).into()))
            .and_then(|m| m.into_optimized())
            .and_then(|m| m.into_runnable()) 
        {
            Ok(m) => {
                println!("AI model loaded successfully from {}", model_path.display());
                Some(m)
            }
            Err(e) => {
                eprintln!("Failed to load AI model: {:?}", e);
                None
            }
        };

        loop {
            // Check for commands
            while let Ok(command) = command_rx.try_recv() {
                match command {
                    AIDetectionCommand::Start => {
                        if camera.is_none() {
                            let index = CameraIndex::Index(0);
                            let requested = RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestFrameRate);
                            camera = match Camera::new(index, requested) {
                                Ok(mut cam) => {
                                    if let Err(e) = cam.open_stream() {
                                        eprintln!("Failed to open camera stream: {:?}", e);
                                        None
                                    } else {
                                        println!("Camera stream opened.");
                                        Some(cam)
                                    }
                                }
                                Err(e) => {
                                    eprintln!("Failed to initialize camera: {:?}", e);
                                    None
                                }
                            };
                        }
                        running = true;
                    }
                    AIDetectionCommand::Stop => {
                        running = false;
                        if let Some(mut cam) = camera.take() {
                            let _ = cam.stop_stream();
                            println!("Camera stream stopped.");
                        }
                        let _ = event_tx.send(false);
                    }
                }
            }

            if running && camera.is_some() && model.is_some() {
                if let Some(ref mut cam) = camera {
                    if let Some(ref runnable) = model {
                        match cam.frame() {
                            Ok(frame) => {
                                if let Ok(img) = frame.decode_image::<RgbFormat>() {
                                    // Preprocess
                                    let resized = image::imageops::resize(&img, 320, 240, image::imageops::FilterType::Triangle);
                                    let tensor: Tensor = tract_ndarray::Array4::from_shape_fn((1, 3, 240, 320), |(_, c, y, x)| {
                                        let pixel = resized.get_pixel(x as u32, y as u32);
                                        let val = match c {
                                            0 => pixel[0], // R
                                            1 => pixel[1], // G
                                            2 => pixel[2], // B
                                            _ => 0,
                                        };
                                        // Normalize (Standard for this model: (x - 127) / 128)
                                        (val as f32 - 127.0) / 128.0
                                    }).into();

                                    // Run inference
                                    match runnable.run(tvec!(tensor.into())) {
                                        Ok(outputs) => {
                                            // Output 0: scores [1, N, 2]
                                            // Output 1: boxes [1, N, 4]
                                            let scores = outputs[0].to_array_view::<f32>().unwrap();
                                            
                                            let mut face_count = 0;
                                            // Simple heuristic: count detections with score > 0.7
                                            // The score for 'face' is usually at index 1 of the last dim.
                                            for i in 0..(scores.len() / 2) {
                                                let score = scores[[0, i, 1]];
                                                if score > 0.7 {
                                                    face_count += 1;
                                                }
                                            }

                                            // If more than 1 face detected, trigger alert
                                            let _ = event_tx.send(face_count > 1);
                                        }
                                        Err(e) => eprintln!("Inference error: {:?}", e),
                                    }
                                }
                            }
                            Err(e) => eprintln!("Failed to capture frame: {:?}", e),
                        }
                    }
                }
                thread::sleep(Duration::from_millis(500)); // Run every 0.5s
            } else {
                thread::sleep(Duration::from_millis(100));
            }
        }
    });
}
