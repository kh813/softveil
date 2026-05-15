use crate::display_config::MonitorId;
use crate::app::AppState;
use crate::platform;
use tao::event_loop::EventLoopWindowTarget;
use tao::monitor::MonitorHandle;
use tao::window::{Window, WindowBuilder};
use softbuffer::{Context, Surface};
use std::num::NonZeroU32;

pub struct OverlayWindow {
    pub monitor_id: MonitorId,
    pub monitor_name: String,
    pub window: Window,
}

#[derive(Debug)]
pub enum OverlayError {
    WindowCreationError(tao::error::OsError),
    SurfaceError(softbuffer::SoftBufferError),
}

impl OverlayWindow {
    pub fn new<T>(
        event_loop: &EventLoopWindowTarget<T>,
        monitor: &MonitorHandle,
        alpha: u8,
    ) -> Result<Self, OverlayError> {
        let monitor_id = MonitorId::from_monitor(monitor);
        let monitor_name = monitor.name().unwrap_or_else(|| format!("Display {}", monitor_id.0));

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

        let window = builder
            .build(event_loop)
            .map_err(OverlayError::WindowCreationError)?;

        platform::apply_overlay_settings(&window, alpha);

        Ok(Self {
            monitor_id,
            monitor_name,
            window,
        })
    }

    pub fn draw(&mut self, _alpha: u8) -> Result<(), OverlayError> {
        let size = self.window.inner_size();
        let width = NonZeroU32::new(size.width.max(1)).unwrap();
        let height = NonZeroU32::new(size.height.max(1)).unwrap();

        let context = Context::new(&self.window).map_err(OverlayError::SurfaceError)?;
        let mut surface = Surface::new(&context, &self.window).map_err(OverlayError::SurfaceError)?;
        
        surface
            .resize(width, height)
            .map_err(OverlayError::SurfaceError)?;

        let mut buffer = surface.buffer_mut().map_err(OverlayError::SurfaceError)?;
        
        // Fill with black. Alpha is handled by NSWindow::setAlphaValue on macOS
        // or SetLayeredWindowAttributes on Windows.
        buffer.fill(0x00000000);

        buffer.present().map_err(OverlayError::SurfaceError)?;
        Ok(())
    }

    pub fn set_visible(&self, visible: bool) {
        self.window.set_visible(visible);
    }

    pub fn update_alpha(&mut self, alpha: u8) -> Result<(), OverlayError> {
        #[cfg(target_os = "windows")]
        platform::apply_overlay_settings(&self.window, alpha);
        
        self.draw(alpha)
    }
}

pub fn create_all<T>(
    event_loop: &EventLoopWindowTarget<T>,
    monitors: Vec<MonitorHandle>,
) -> Vec<OverlayWindow> {
    let mut overlays = Vec::new();
    for monitor in monitors {
        match OverlayWindow::new(event_loop, &monitor, 77) {
            Ok(mut overlay) => {
                let _ = overlay.draw(77);
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
    visible: bool,
    alpha: u8,
) -> Result<MonitorId, OverlayError> {
    let mut overlay = OverlayWindow::new(event_loop, monitor, alpha)?;
    overlay.set_visible(visible);
    if visible {
        let _ = overlay.draw(alpha);
    }
    let id = overlay.monitor_id;
    overlays.push(overlay);
    Ok(id)
}

pub fn remove_display(overlays: &mut Vec<OverlayWindow>, id: &MonitorId) {
    overlays.retain(|o| o.monitor_id != *id);
}

pub fn sync_all(overlays: &mut Vec<OverlayWindow>, state: &AppState) {
    for overlay in overlays {
        let visible = state.is_visible(&overlay.monitor_id);
        overlay.set_visible(visible);
        if visible {
            if let Some(config) = state.displays.get(&overlay.monitor_id) {
                let _ = overlay.update_alpha(config.alpha_u8());
            }
        }
    }
}
