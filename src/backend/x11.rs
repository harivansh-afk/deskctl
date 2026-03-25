use anyhow::{Context, Result};

use crate::core::types::{Snapshot, WindowInfo};
use super::annotate::annotate_screenshot;

pub struct X11Backend {
    // enigo and x11rb connections added in later phases
}

impl X11Backend {
    pub fn new() -> Result<Self> {
        Ok(Self {})
    }
}

impl super::DesktopBackend for X11Backend {
    fn snapshot(&mut self, annotate: bool) -> Result<Snapshot> {
        // Get z-ordered window list via xcap (topmost first internally)
        let windows = xcap::Window::all()
            .context("Failed to enumerate windows")?;

        // Get primary monitor for screenshot
        let monitors = xcap::Monitor::all()
            .context("Failed to enumerate monitors")?;
        let monitor = monitors.into_iter().next()
            .context("No monitor found")?;

        let mut image = monitor.capture_image()
            .context("Failed to capture screenshot")?;

        // Build window info list
        let mut window_infos = Vec::new();
        let mut ref_counter = 1usize;

        for win in &windows {
            // Each xcap method returns XCapResult<T> - skip windows where metadata fails
            let title = win.title().unwrap_or_default();
            let app_name = win.app_name().unwrap_or_default();

            // Skip windows with empty titles and app names (desktop, panels, etc.)
            if title.is_empty() && app_name.is_empty() {
                continue;
            }

            let xcb_id = win.id().unwrap_or(0);
            let x = win.x().unwrap_or(0);
            let y = win.y().unwrap_or(0);
            let width = win.width().unwrap_or(0);
            let height = win.height().unwrap_or(0);
            let focused = win.is_focused().unwrap_or(false);
            let minimized = win.is_minimized().unwrap_or(false);

            let ref_id = format!("w{ref_counter}");
            ref_counter += 1;

            window_infos.push(WindowInfo {
                ref_id,
                xcb_id,
                title,
                app_name,
                x,
                y,
                width,
                height,
                focused,
                minimized,
            });
        }

        // Annotate if requested - draw bounding boxes and @wN labels
        if annotate {
            annotate_screenshot(&mut image, &window_infos);
        }

        // Save screenshot
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        let screenshot_path = format!("/tmp/desktop-ctl-{timestamp}.png");
        image.save(&screenshot_path)
            .context("Failed to save screenshot")?;

        Ok(Snapshot {
            screenshot: screenshot_path,
            windows: window_infos,
        })
    }

    // Stub implementations for methods added in later phases
    fn focus_window(&mut self, _xcb_id: u32) -> Result<()> {
        anyhow::bail!("Window management not yet implemented (Phase 5)")
    }

    fn move_window(&mut self, _xcb_id: u32, _x: i32, _y: i32) -> Result<()> {
        anyhow::bail!("Window management not yet implemented (Phase 5)")
    }

    fn resize_window(&mut self, _xcb_id: u32, _w: u32, _h: u32) -> Result<()> {
        anyhow::bail!("Window management not yet implemented (Phase 5)")
    }

    fn close_window(&mut self, _xcb_id: u32) -> Result<()> {
        anyhow::bail!("Window management not yet implemented (Phase 5)")
    }

    fn click(&mut self, _x: i32, _y: i32) -> Result<()> {
        anyhow::bail!("Input simulation not yet implemented (Phase 4)")
    }

    fn dblclick(&mut self, _x: i32, _y: i32) -> Result<()> {
        anyhow::bail!("Input simulation not yet implemented (Phase 4)")
    }

    fn type_text(&mut self, _text: &str) -> Result<()> {
        anyhow::bail!("Input simulation not yet implemented (Phase 4)")
    }

    fn press_key(&mut self, _key: &str) -> Result<()> {
        anyhow::bail!("Input simulation not yet implemented (Phase 4)")
    }

    fn hotkey(&mut self, _keys: &[String]) -> Result<()> {
        anyhow::bail!("Input simulation not yet implemented (Phase 4)")
    }

    fn mouse_move(&mut self, _x: i32, _y: i32) -> Result<()> {
        anyhow::bail!("Input simulation not yet implemented (Phase 4)")
    }

    fn scroll(&mut self, _amount: i32, _axis: &str) -> Result<()> {
        anyhow::bail!("Input simulation not yet implemented (Phase 4)")
    }

    fn drag(&mut self, _x1: i32, _y1: i32, _x2: i32, _y2: i32) -> Result<()> {
        anyhow::bail!("Input simulation not yet implemented (Phase 4)")
    }

    fn screen_size(&self) -> Result<(u32, u32)> {
        anyhow::bail!("Utility commands not yet implemented (Phase 6)")
    }

    fn mouse_position(&self) -> Result<(i32, i32)> {
        anyhow::bail!("Utility commands not yet implemented (Phase 6)")
    }

    fn screenshot(&mut self, _path: &str, _annotate: bool) -> Result<String> {
        anyhow::bail!("Standalone screenshot not yet implemented (Phase 6)")
    }

    fn launch(&self, _command: &str, _args: &[String]) -> Result<u32> {
        anyhow::bail!("Launch not yet implemented (Phase 6)")
    }
}
