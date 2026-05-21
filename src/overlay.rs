use crate::display_config::{MonitorId, FilterMode};
use crate::app::AppState;
use crate::platform;
use tao::event_loop::EventLoopWindowTarget;
use tao::monitor::MonitorHandle;
use tao::window::{Window, WindowBuilder};
use std::sync::Arc;

pub struct GpuContext {
    pub instance: wgpu::Instance,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}

impl GpuContext {
    pub async fn new() -> Option<Self> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            // Windows では DX12/DX11 を優先し、Vulkan を避けることで安定性と負荷を改善する
            #[cfg(target_os = "windows")]
            backends: wgpu::Backends::DX12,
            #[cfg(not(target_os = "windows"))]
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::LowPower,
            compatible_surface: None,
            force_fallback_adapter: false,
        }).await?;

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
            },
            None,
        ).await.ok()?;

        Some(Self {
            instance,
            adapter,
            device,
            queue,
        })
    }
}

#[repr(C, align(16))]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    v0: [f32; 4], // time, mode, alpha, width
    v1: [f32; 4], // height, panel_type, refresh_rate, intensity
    v2: [f32; 4], // bidirectional, period_px, scroll_speed_px, cover_ratio
    v3: [f32; 4], // phase_flip_hz, grid_period_px, luminance_compress, hatch_angle
    v4: [f32; 4], // is_light_mode, cos_hatch, sin_hatch, padding
}


// Ensure Uniforms is exactly 80 bytes and 16-byte aligned for WGSL
const _: () = assert!(std::mem::size_of::<Uniforms>() == 80);
const _: () = assert!(std::mem::align_of::<Uniforms>() >= 16);


pub struct OverlayWindow {
    pub monitor_id: MonitorId,
    pub monitor_name: String,
    pub window: Arc<Window>,
    pub surface: wgpu::Surface<'static>,
    pub config: wgpu::SurfaceConfiguration,
    pub pipeline: wgpu::RenderPipeline,
    pub uniform_buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
    pub start_time: std::time::Instant,
    pub panel_type: crate::display_config::PanelType,
    pub refresh_rate: u32,
}

#[derive(Debug)]
pub enum OverlayError {
    WindowCreationError(#[allow(dead_code)] tao::error::OsError),
    GpuError(#[allow(dead_code)] String),
}

impl OverlayWindow {
    pub fn new<T>(
        event_loop: &EventLoopWindowTarget<T>,
        monitor: &MonitorHandle,
        alpha: u8,
        gpu: &Arc<GpuContext>,
    ) -> Result<Self, OverlayError> {
        let monitor_id = MonitorId::from_monitor(monitor);
        let monitor_name = platform::get_monitor_name(monitor);

        let builder = WindowBuilder::new()
            .with_decorations(false)
            .with_transparent(true)
            .with_always_on_top(true)
            .with_position(monitor.position())
            .with_inner_size(monitor.size());

        #[cfg(target_os = "windows")]
        let builder = {
            use tao::platform::windows::WindowBuilderExtWindows;
            builder.with_skip_taskbar(true)
        };

        let window = Arc::new(builder
            .build(event_loop)
            .map_err(OverlayError::WindowCreationError)?);

        platform::apply_overlay_settings(&window, alpha);

        let size = window.inner_size();
        let surface = gpu.instance.create_surface(window.clone()).map_err(|e| OverlayError::GpuError(e.to_string()))?;

        let surface_caps = surface.get_capabilities(&gpu.adapter);
        let surface_format = surface_caps.formats.iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&gpu.device, &config);

        let shader = gpu.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let uniform_buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Uniform Buffer"),
            size: std::mem::size_of::<Uniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout = gpu.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Bind Group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let pipeline_layout = gpu.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = gpu.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        let refresh_rate = monitor.video_modes().next().map(|m| m.refresh_rate() as u32).unwrap_or(60);

        Ok(Self {
            monitor_id,
            monitor_name,
            window,
            surface,
            config,
            pipeline,
            uniform_buffer,
            bind_group,
            start_time: std::time::Instant::now(),
            panel_type: platform::detect_panel_type(monitor),
            refresh_rate,
        })
    }

    pub fn draw(&mut self, gpu: &GpuContext, state: &AppState, alpha: u8) -> Result<(), OverlayError> {
        let size = self.window.inner_size();

        // DisplayProfile を取得
        let config_ref = state.displays.get(&self.monitor_id).cloned().unwrap_or_default();
        let profile = config_ref.get_effective_profile();
        let ppi = state.displays.get(&self.monitor_id).map(|c| c.ppi).unwrap_or(110.0);
        let luminance_compress = (0.20f32 / profile.intensity_scale()).clamp(0.10, 0.35);

        let mode_val = match state.filter_mode(&self.monitor_id) {
            FilterMode::BlackLayer => 0.0,
            FilterMode::VerticalLouver => 1.0,
            FilterMode::AIOcrInterference => 2.0,
            FilterMode::HighIntensitySPD => 3.0,
            FilterMode::StealthDark => 4.0,
            FilterMode::StealthLight => 5.0,
        };

        let effective_mode = mode_val;

        let hatch_angle = std::f32::consts::FRAC_PI_4;

        let uniforms = Uniforms {
            v0: [
                self.start_time.elapsed().as_secs_f32(),
                effective_mode,
                alpha as f32 / 255.0,
                size.width as f32,
            ],
            v1: [
                size.height as f32,
                match state.panel_type(&self.monitor_id) {
                    crate::display_config::PanelType::Unknown => 0.0,
                    crate::display_config::PanelType::Oled => 1.0,
                    crate::display_config::PanelType::LcdIps => 2.0,
                    crate::display_config::PanelType::LcdTn => 3.0,
                },
                self.refresh_rate as f32,
                state.filter_intensity(&self.monitor_id),
            ],
            v2: [
                if profile.bidirectional { 1.0 } else { 0.0 },
                profile.period_px(ppi),
                profile.scroll_speed_px(ppi),
                profile.cover_ratio,
            ],
            v3: [
                profile.phase_flip_hz,
                profile.period_px(ppi),
                luminance_compress,
                hatch_angle,
            ],
            v4: [
                if platform::is_dark_mode() { 0.0 } else { 1.0 }, // Light Mode flag
                hatch_angle.cos(),
                hatch_angle.sin(),
                0.0
            ],
        };
        gpu.queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));

        let output = self.surface.get_current_texture().map_err(|e| OverlayError::GpuError(e.to_string()))?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = gpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 0.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0, &self.bind_group, &[]);
            render_pass.draw(0..3, 0..1);
        }

        gpu.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    pub fn resize(&mut self, gpu: &GpuContext, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&gpu.device, &self.config);
        }
    }

    pub fn set_visible(&self, visible: bool) {
        self.window.set_visible(visible);
    }

    pub fn update_alpha(&mut self, gpu: &GpuContext, state: &AppState, alpha: u8) -> Result<(), OverlayError> {
        #[cfg(target_os = "windows")]
        platform::apply_overlay_settings(&self.window, alpha);
        
        self.draw(gpu, state, alpha)
    }
}

pub fn create_all<T>(
    event_loop: &EventLoopWindowTarget<T>,
    monitors: Vec<MonitorHandle>,
    state: &AppState,
    gpu: &Arc<GpuContext>,
) -> Vec<OverlayWindow> {
    let mut overlays = Vec::new();
    for monitor in monitors {
        match OverlayWindow::new(event_loop, &monitor, 77, gpu) {
            Ok(mut overlay) => {
                let _ = overlay.draw(gpu, state, 77);
                overlays.push(overlay);
            }
            Err(e) => eprintln!("Failed to create overlay window: {:?}", e),
        }
    }
    overlays
}

pub fn add_display<T>(
    overlays: &mut Vec<OverlayWindow>,
    event_loop: &EventLoopWindowTarget<T>,
    monitor: &MonitorHandle,
    state: &AppState,
    visible: bool,
    alpha: u8,
    gpu: &Arc<GpuContext>,
) -> Result<MonitorId, OverlayError> {
    let mut overlay = OverlayWindow::new(event_loop, monitor, alpha, gpu)?;
    overlay.set_visible(visible);
    if visible {
        let _ = overlay.draw(gpu, state, alpha);
    }
    let id = overlay.monitor_id;
    overlays.push(overlay);
    Ok(id)
}

pub fn remove_display(overlays: &mut Vec<OverlayWindow>, id: &MonitorId) {
    overlays.retain(|o| o.monitor_id != *id);
}

pub fn sync_all(overlays: &mut Vec<OverlayWindow>, state: &AppState, gpu: &GpuContext) {
    for overlay in overlays {
        let visible = state.is_visible(&overlay.monitor_id);
        overlay.set_visible(visible);
        if visible {
            let alpha = state.effective_alpha_u8(&overlay.monitor_id);
            let _ = overlay.update_alpha(gpu, state, alpha);
        }
    }
}
