use anyhow::{Context, Result};
use enigo::{Axis, Button, Coordinate, Direction, Enigo, Key, Keyboard, Mouse, Settings};
use image::RgbaImage;
use x11rb::connection::Connection;
use x11rb::protocol::randr::ConnectionExt as RandrConnectionExt;
use x11rb::protocol::xproto::{
    Atom, AtomEnum, ClientMessageData, ClientMessageEvent, ConfigureWindowAux,
    ConnectionExt as XprotoConnectionExt, EventMask, GetPropertyReply, ImageFormat, ImageOrder,
    Window,
};
use x11rb::rust_connection::RustConnection;

use crate::backend::{BackendMonitor, BackendWindow};

struct Atoms {
    client_list_stacking: Atom,
    active_window: Atom,
    net_wm_name: Atom,
    utf8_string: Atom,
    wm_name: Atom,
    wm_class: Atom,
    net_wm_state: Atom,
    net_wm_state_hidden: Atom,
}

pub struct X11Backend {
    enigo: Enigo,
    conn: RustConnection,
    root: Window,
    atoms: Atoms,
}

impl X11Backend {
    pub fn new() -> Result<Self> {
        let enigo = Enigo::new(&Settings::default())
            .map_err(|e| anyhow::anyhow!("Failed to initialize enigo: {e}"))?;
        let (conn, screen_num) = x11rb::connect(None).context("Failed to connect to X11 server")?;
        let root = conn.setup().roots[screen_num].root;
        let atoms = Atoms::new(&conn)?;
        Ok(Self {
            enigo,
            conn,
            root,
            atoms,
        })
    }

    fn stacked_windows(&self) -> Result<Vec<Window>> {
        let mut windows = self
            .get_property_u32(
                self.root,
                self.atoms.client_list_stacking,
                AtomEnum::WINDOW.into(),
                1024,
            )?
            .into_iter()
            .map(|id| id as Window)
            .collect::<Vec<_>>();

        if windows.is_empty() {
            windows = self
                .conn
                .query_tree(self.root)?
                .reply()
                .context("Failed to query root window tree")?
                .children;
        }

        // EWMH exposes bottom-to-top stacking order. Reverse it so @w1 is the topmost window.
        windows.reverse();
        Ok(windows)
    }

    fn collect_window_infos(&self) -> Result<Vec<BackendWindow>> {
        let active_window = self.active_window()?;
        let mut window_infos = Vec::new();

        for window in self.stacked_windows()? {
            let title = self.window_title(window).unwrap_or_default();
            let app_name = self.window_app_name(window).unwrap_or_default();
            if title.is_empty() && app_name.is_empty() {
                continue;
            }

            let (x, y, width, height) = match self.window_geometry(window) {
                Ok(geometry) => geometry,
                Err(_) => continue,
            };

            let minimized = self.window_is_minimized(window).unwrap_or(false);
            window_infos.push(BackendWindow {
                native_id: window,
                title,
                app_name,
                x,
                y,
                width,
                height,
                focused: active_window == Some(window),
                minimized,
            });
        }

        Ok(window_infos)
    }

    fn active_window_info(&self) -> Result<Option<BackendWindow>> {
        let Some(active_window) = self.active_window()? else {
            return Ok(None);
        };

        let title = self.window_title(active_window).unwrap_or_default();
        let app_name = self.window_app_name(active_window).unwrap_or_default();
        if title.is_empty() && app_name.is_empty() {
            return Ok(None);
        }

        let (x, y, width, height) = self.window_geometry(active_window)?;
        let minimized = self.window_is_minimized(active_window).unwrap_or(false);
        Ok(Some(BackendWindow {
            native_id: active_window,
            title,
            app_name,
            x,
            y,
            width,
            height,
            focused: true,
            minimized,
        }))
    }

    fn collect_monitors(&self) -> Result<Vec<BackendMonitor>> {
        let reply = self
            .conn
            .randr_get_monitors(self.root, true)?
            .reply()
            .context("Failed to query RANDR monitors")?;

        let mut monitors = Vec::with_capacity(reply.monitors.len());
        for (index, monitor) in reply.monitors.into_iter().enumerate() {
            monitors.push(BackendMonitor {
                name: self
                    .atom_name(monitor.name)
                    .unwrap_or_else(|_| format!("monitor{}", index + 1)),
                x: i32::from(monitor.x),
                y: i32::from(monitor.y),
                width: u32::from(monitor.width),
                height: u32::from(monitor.height),
                width_mm: monitor.width_in_millimeters,
                height_mm: monitor.height_in_millimeters,
                primary: monitor.primary,
                automatic: monitor.automatic,
            });
        }

        if monitors.is_empty() {
            let (width, height) = self.root_geometry()?;
            monitors.push(BackendMonitor {
                name: "screen".to_string(),
                x: 0,
                y: 0,
                width,
                height,
                width_mm: 0,
                height_mm: 0,
                primary: true,
                automatic: true,
            });
        }

        Ok(monitors)
    }

    fn capture_root_image(&self) -> Result<RgbaImage> {
        let (width, height) = self.root_geometry()?;
        let reply = self
            .conn
            .get_image(
                ImageFormat::Z_PIXMAP,
                self.root,
                0,
                0,
                width as u16,
                height as u16,
                u32::MAX,
            )?
            .reply()
            .context("Failed to capture root window image")?;

        rgba_from_image_reply(self.conn.setup(), width, height, &reply)
    }

    fn root_geometry(&self) -> Result<(u32, u32)> {
        let geometry = self
            .conn
            .get_geometry(self.root)?
            .reply()
            .context("Failed to get root geometry")?;
        Ok((geometry.width.into(), geometry.height.into()))
    }

    fn window_geometry(&self, window: Window) -> Result<(i32, i32, u32, u32)> {
        let geometry = self
            .conn
            .get_geometry(window)?
            .reply()
            .context("Failed to get window geometry")?;
        let translated = self
            .conn
            .translate_coordinates(window, self.root, 0, 0)?
            .reply()
            .context("Failed to translate window coordinates to root")?;

        Ok((
            i32::from(translated.dst_x),
            i32::from(translated.dst_y),
            geometry.width.into(),
            geometry.height.into(),
        ))
    }

    fn active_window(&self) -> Result<Option<Window>> {
        Ok(self
            .get_property_u32(
                self.root,
                self.atoms.active_window,
                AtomEnum::WINDOW.into(),
                1,
            )?
            .into_iter()
            .next()
            .map(|id| id as Window))
    }

    fn window_title(&self, window: Window) -> Result<String> {
        let title =
            self.read_text_property(window, self.atoms.net_wm_name, self.atoms.utf8_string)?;
        if !title.is_empty() {
            return Ok(title);
        }

        self.read_text_property(window, self.atoms.wm_name, AtomEnum::ANY.into())
    }

    fn window_app_name(&self, window: Window) -> Result<String> {
        let wm_class =
            self.read_text_property(window, self.atoms.wm_class, AtomEnum::STRING.into())?;
        let mut parts = wm_class.split('\0').filter(|part| !part.is_empty());
        Ok(parts
            .nth(1)
            .or_else(|| parts.next())
            .unwrap_or("")
            .to_string())
    }

    fn window_is_minimized(&self, window: Window) -> Result<bool> {
        let states =
            self.get_property_u32(window, self.atoms.net_wm_state, AtomEnum::ATOM.into(), 32)?;
        Ok(states.contains(&self.atoms.net_wm_state_hidden))
    }

    fn read_text_property(&self, window: Window, property: Atom, type_: Atom) -> Result<String> {
        let reply = self.get_property(window, property, type_, 1024)?;
        Ok(String::from_utf8_lossy(&reply.value)
            .trim_end_matches('\0')
            .to_string())
    }

    fn get_property_u32(
        &self,
        window: Window,
        property: Atom,
        type_: Atom,
        long_length: u32,
    ) -> Result<Vec<u32>> {
        let reply = self.get_property(window, property, type_, long_length)?;
        Ok(reply
            .value32()
            .map(|iter| iter.collect::<Vec<_>>())
            .unwrap_or_default())
    }

    fn get_property(
        &self,
        window: Window,
        property: Atom,
        type_: Atom,
        long_length: u32,
    ) -> Result<GetPropertyReply> {
        self.conn
            .get_property(false, window, property, type_, 0, long_length)?
            .reply()
            .with_context(|| format!("Failed to read property {property} from window {window}"))
    }

    fn atom_name(&self, atom: Atom) -> Result<String> {
        self.conn
            .get_atom_name(atom)?
            .reply()
            .map(|reply| String::from_utf8_lossy(&reply.name).to_string())
            .with_context(|| format!("Failed to read atom name for {atom}"))
    }
}

impl super::DesktopBackend for X11Backend {
    fn list_windows(&mut self) -> Result<Vec<BackendWindow>> {
        self.collect_window_infos()
    }

    fn active_window(&mut self) -> Result<Option<BackendWindow>> {
        self.active_window_info()
    }

    fn list_monitors(&self) -> Result<Vec<BackendMonitor>> {
        match self.collect_monitors() {
            Ok(monitors) => Ok(monitors),
            Err(_) => {
                let (width, height) = self.root_geometry()?;
                Ok(vec![BackendMonitor {
                    name: "screen".to_string(),
                    x: 0,
                    y: 0,
                    width,
                    height,
                    width_mm: 0,
                    height_mm: 0,
                    primary: true,
                    automatic: true,
                }])
            }
        }
    }

    fn capture_screenshot(&mut self) -> Result<RgbaImage> {
        self.capture_root_image()
    }

    fn focus_window(&mut self, native_id: u32) -> Result<()> {
        // Use _NET_ACTIVE_WINDOW client message (avoids focus-stealing prevention)
        let net_active = self
            .conn
            .intern_atom(false, b"_NET_ACTIVE_WINDOW")?
            .reply()
            .context("Failed to intern _NET_ACTIVE_WINDOW atom")?
            .atom;

        let event = ClientMessageEvent {
            response_type: x11rb::protocol::xproto::CLIENT_MESSAGE_EVENT,
            format: 32,
            sequence: 0,
            window: native_id,
            type_: net_active,
            data: ClientMessageData::from([
                2u32, 0, 0, 0, 0, // source=2 (pager), timestamp=0, currently_active=0
            ]),
        };

        self.conn.send_event(
            false,
            self.root,
            EventMask::SUBSTRUCTURE_REDIRECT | EventMask::SUBSTRUCTURE_NOTIFY,
            event,
        )?;
        self.conn
            .flush()
            .context("Failed to flush X11 connection")?;
        Ok(())
    }

    fn move_window(&mut self, native_id: u32, x: i32, y: i32) -> Result<()> {
        self.conn
            .configure_window(native_id, &ConfigureWindowAux::new().x(x).y(y))?;
        self.conn
            .flush()
            .context("Failed to flush X11 connection")?;
        Ok(())
    }

    fn resize_window(&mut self, native_id: u32, w: u32, h: u32) -> Result<()> {
        self.conn
            .configure_window(native_id, &ConfigureWindowAux::new().width(w).height(h))?;
        self.conn
            .flush()
            .context("Failed to flush X11 connection")?;
        Ok(())
    }

    fn close_window(&mut self, native_id: u32) -> Result<()> {
        // Use _NET_CLOSE_WINDOW for graceful close (respects WM protocols)
        let net_close = self
            .conn
            .intern_atom(false, b"_NET_CLOSE_WINDOW")?
            .reply()
            .context("Failed to intern _NET_CLOSE_WINDOW atom")?
            .atom;

        let event = ClientMessageEvent {
            response_type: x11rb::protocol::xproto::CLIENT_MESSAGE_EVENT,
            format: 32,
            sequence: 0,
            window: native_id,
            type_: net_close,
            data: ClientMessageData::from([
                0u32, 2, 0, 0, 0, // timestamp=0, source=2 (pager)
            ]),
        };

        self.conn.send_event(
            false,
            self.root,
            EventMask::SUBSTRUCTURE_REDIRECT | EventMask::SUBSTRUCTURE_NOTIFY,
            event,
        )?;
        self.conn
            .flush()
            .context("Failed to flush X11 connection")?;
        Ok(())
    }

    // Phase 4: input simulation via enigo

    fn click(&mut self, x: i32, y: i32) -> Result<()> {
        self.enigo
            .move_mouse(x, y, Coordinate::Abs)
            .map_err(|e| anyhow::anyhow!("Mouse move failed: {e}"))?;
        std::thread::sleep(std::time::Duration::from_millis(10));
        self.enigo
            .button(Button::Left, Direction::Click)
            .map_err(|e| anyhow::anyhow!("Click failed: {e}"))?;
        Ok(())
    }

    fn dblclick(&mut self, x: i32, y: i32) -> Result<()> {
        self.enigo
            .move_mouse(x, y, Coordinate::Abs)
            .map_err(|e| anyhow::anyhow!("Mouse move failed: {e}"))?;
        std::thread::sleep(std::time::Duration::from_millis(10));
        self.enigo
            .button(Button::Left, Direction::Click)
            .map_err(|e| anyhow::anyhow!("First click failed: {e}"))?;
        std::thread::sleep(std::time::Duration::from_millis(50));
        self.enigo
            .button(Button::Left, Direction::Click)
            .map_err(|e| anyhow::anyhow!("Second click failed: {e}"))?;
        Ok(())
    }

    fn type_text(&mut self, text: &str) -> Result<()> {
        self.enigo
            .text(text)
            .map_err(|e| anyhow::anyhow!("Type failed: {e}"))?;
        Ok(())
    }

    fn press_key(&mut self, key: &str) -> Result<()> {
        let k = parse_key(key)?;
        self.enigo
            .key(k, Direction::Click)
            .map_err(|e| anyhow::anyhow!("Key press failed: {e}"))?;
        Ok(())
    }

    fn hotkey(&mut self, keys: &[String]) -> Result<()> {
        // Press all modifier keys, click the last key, release modifiers in reverse
        let parsed: Vec<Key> = keys
            .iter()
            .map(|k| parse_key(k))
            .collect::<Result<Vec<_>>>()?;

        if parsed.is_empty() {
            anyhow::bail!("No keys specified for hotkey");
        }

        let (modifiers, tail) = parsed.split_at(parsed.len() - 1);

        for m in modifiers {
            self.enigo
                .key(*m, Direction::Press)
                .map_err(|e| anyhow::anyhow!("Modifier press failed: {e}"))?;
        }

        self.enigo
            .key(tail[0], Direction::Click)
            .map_err(|e| anyhow::anyhow!("Key click failed: {e}"))?;

        for m in modifiers.iter().rev() {
            self.enigo
                .key(*m, Direction::Release)
                .map_err(|e| anyhow::anyhow!("Modifier release failed: {e}"))?;
        }

        Ok(())
    }

    fn mouse_move(&mut self, x: i32, y: i32) -> Result<()> {
        self.enigo
            .move_mouse(x, y, Coordinate::Abs)
            .map_err(|e| anyhow::anyhow!("Mouse move failed: {e}"))?;
        Ok(())
    }

    fn scroll(&mut self, amount: i32, axis: &str) -> Result<()> {
        let ax = match axis {
            "horizontal" | "h" => Axis::Horizontal,
            _ => Axis::Vertical,
        };
        self.enigo
            .scroll(amount, ax)
            .map_err(|e| anyhow::anyhow!("Scroll failed: {e}"))?;
        Ok(())
    }

    fn drag(&mut self, x1: i32, y1: i32, x2: i32, y2: i32) -> Result<()> {
        self.enigo
            .move_mouse(x1, y1, Coordinate::Abs)
            .map_err(|e| anyhow::anyhow!("Mouse move failed: {e}"))?;
        std::thread::sleep(std::time::Duration::from_millis(10));
        self.enigo
            .button(Button::Left, Direction::Press)
            .map_err(|e| anyhow::anyhow!("Button press failed: {e}"))?;
        std::thread::sleep(std::time::Duration::from_millis(50));
        self.enigo
            .move_mouse(x2, y2, Coordinate::Abs)
            .map_err(|e| anyhow::anyhow!("Mouse move to target failed: {e}"))?;
        std::thread::sleep(std::time::Duration::from_millis(10));
        self.enigo
            .button(Button::Left, Direction::Release)
            .map_err(|e| anyhow::anyhow!("Button release failed: {e}"))?;
        Ok(())
    }

    fn screen_size(&self) -> Result<(u32, u32)> {
        self.root_geometry()
    }

    fn mouse_position(&self) -> Result<(i32, i32)> {
        let reply = self
            .conn
            .query_pointer(self.root)?
            .reply()
            .context("Failed to query pointer")?;
        Ok((reply.root_x as i32, reply.root_y as i32))
    }

    fn launch(&self, command: &str, args: &[String]) -> Result<u32> {
        let child = std::process::Command::new(command)
            .args(args)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .with_context(|| format!("Failed to launch: {command}"))?;
        Ok(child.id())
    }

    fn backend_name(&self) -> &'static str {
        "x11"
    }
}

fn parse_key(name: &str) -> Result<Key> {
    match name.to_lowercase().as_str() {
        // Modifiers
        "ctrl" | "control" => Ok(Key::Control),
        "alt" => Ok(Key::Alt),
        "shift" => Ok(Key::Shift),
        "super" | "meta" | "win" => Ok(Key::Meta),

        // Navigation / editing
        "enter" | "return" => Ok(Key::Return),
        "tab" => Ok(Key::Tab),
        "escape" | "esc" => Ok(Key::Escape),
        "backspace" => Ok(Key::Backspace),
        "delete" | "del" => Ok(Key::Delete),
        "space" => Ok(Key::Space),

        // Arrow keys
        "up" => Ok(Key::UpArrow),
        "down" => Ok(Key::DownArrow),
        "left" => Ok(Key::LeftArrow),
        "right" => Ok(Key::RightArrow),

        // Page navigation
        "home" => Ok(Key::Home),
        "end" => Ok(Key::End),
        "pageup" => Ok(Key::PageUp),
        "pagedown" => Ok(Key::PageDown),

        // Function keys
        "f1" => Ok(Key::F1),
        "f2" => Ok(Key::F2),
        "f3" => Ok(Key::F3),
        "f4" => Ok(Key::F4),
        "f5" => Ok(Key::F5),
        "f6" => Ok(Key::F6),
        "f7" => Ok(Key::F7),
        "f8" => Ok(Key::F8),
        "f9" => Ok(Key::F9),
        "f10" => Ok(Key::F10),
        "f11" => Ok(Key::F11),
        "f12" => Ok(Key::F12),

        // Single character - map to Unicode key
        s if s.len() == 1 => Ok(Key::Unicode(s.chars().next().unwrap())),

        other => anyhow::bail!("Unknown key: {other}"),
    }
}

impl Atoms {
    fn new(conn: &RustConnection) -> Result<Self> {
        Ok(Self {
            client_list_stacking: intern_atom(conn, b"_NET_CLIENT_LIST_STACKING")?,
            active_window: intern_atom(conn, b"_NET_ACTIVE_WINDOW")?,
            net_wm_name: intern_atom(conn, b"_NET_WM_NAME")?,
            utf8_string: intern_atom(conn, b"UTF8_STRING")?,
            wm_name: intern_atom(conn, b"WM_NAME")?,
            wm_class: intern_atom(conn, b"WM_CLASS")?,
            net_wm_state: intern_atom(conn, b"_NET_WM_STATE")?,
            net_wm_state_hidden: intern_atom(conn, b"_NET_WM_STATE_HIDDEN")?,
        })
    }
}

fn intern_atom(conn: &RustConnection, name: &[u8]) -> Result<Atom> {
    conn.intern_atom(false, name)?
        .reply()
        .with_context(|| format!("Failed to intern atom {}", String::from_utf8_lossy(name)))
        .map(|reply| reply.atom)
}

fn rgba_from_image_reply(
    setup: &x11rb::protocol::xproto::Setup,
    width: u32,
    height: u32,
    reply: &x11rb::protocol::xproto::GetImageReply,
) -> Result<RgbaImage> {
    let pixmap_format = setup
        .pixmap_formats
        .iter()
        .find(|format| format.depth == reply.depth)
        .context("Failed to find pixmap format for captured image depth")?;
    let bits_per_pixel = u32::from(pixmap_format.bits_per_pixel);
    let bit_order = setup.bitmap_format_bit_order;
    let bytes = reply.data.as_slice();

    let get_pixel_rgba = match reply.depth {
        8 => pixel8_rgba,
        16 => pixel16_rgba,
        24 | 32 => pixel24_32_rgba,
        depth => anyhow::bail!("Unsupported X11 image depth: {depth}"),
    };

    let mut rgba = vec![0u8; (width * height * 4) as usize];
    for y in 0..height {
        for x in 0..width {
            let index = ((y * width + x) * 4) as usize;
            let (r, g, b, a) = get_pixel_rgba(bytes, x, y, width, bits_per_pixel, bit_order);
            rgba[index] = r;
            rgba[index + 1] = g;
            rgba[index + 2] = b;
            rgba[index + 3] = a;
        }
    }

    RgbaImage::from_raw(width, height, rgba)
        .context("Failed to convert captured X11 image into RGBA buffer")
}

fn pixel8_rgba(
    bytes: &[u8],
    x: u32,
    y: u32,
    width: u32,
    bits_per_pixel: u32,
    bit_order: ImageOrder,
) -> (u8, u8, u8, u8) {
    let index = ((y * width + x) * bits_per_pixel / 8) as usize;
    let pixel = if bit_order == ImageOrder::LSB_FIRST {
        bytes[index]
    } else {
        bytes[index] & (7 << 4) | (bytes[index] >> 4)
    };

    let r = (pixel >> 6) as f32 / 3.0 * 255.0;
    let g = ((pixel >> 2) & 7) as f32 / 7.0 * 255.0;
    let b = (pixel & 3) as f32 / 3.0 * 255.0;
    (r as u8, g as u8, b as u8, 255)
}

fn pixel16_rgba(
    bytes: &[u8],
    x: u32,
    y: u32,
    width: u32,
    bits_per_pixel: u32,
    bit_order: ImageOrder,
) -> (u8, u8, u8, u8) {
    let index = ((y * width + x) * bits_per_pixel / 8) as usize;
    let pixel = if bit_order == ImageOrder::LSB_FIRST {
        u16::from(bytes[index]) | (u16::from(bytes[index + 1]) << 8)
    } else {
        (u16::from(bytes[index]) << 8) | u16::from(bytes[index + 1])
    };

    let r = (pixel >> 11) as f32 / 31.0 * 255.0;
    let g = ((pixel >> 5) & 63) as f32 / 63.0 * 255.0;
    let b = (pixel & 31) as f32 / 31.0 * 255.0;
    (r as u8, g as u8, b as u8, 255)
}

fn pixel24_32_rgba(
    bytes: &[u8],
    x: u32,
    y: u32,
    width: u32,
    bits_per_pixel: u32,
    bit_order: ImageOrder,
) -> (u8, u8, u8, u8) {
    let index = ((y * width + x) * bits_per_pixel / 8) as usize;
    if bit_order == ImageOrder::LSB_FIRST {
        (bytes[index + 2], bytes[index + 1], bytes[index], 255)
    } else {
        (bytes[index], bytes[index + 1], bytes[index + 2], 255)
    }
}
