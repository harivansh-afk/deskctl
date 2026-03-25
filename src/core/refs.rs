use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use crate::backend::BackendWindow;
use crate::core::types::WindowInfo;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct RefEntry {
    pub window_id: String,
    pub backend_window_id: u32,
    pub app_class: String,
    pub title: String,
    pub pid: u32,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub focused: bool,
    pub minimized: bool,
}

#[derive(Debug, Default)]
#[allow(dead_code)]
pub struct RefMap {
    refs: HashMap<String, RefEntry>,
    window_id_to_ref: HashMap<String, String>,
    backend_id_to_window_id: HashMap<u32, String>,
    next_ref: usize,
    next_window: usize,
}

#[allow(dead_code)]
impl RefMap {
    pub fn new() -> Self {
        Self {
            refs: HashMap::new(),
            window_id_to_ref: HashMap::new(),
            backend_id_to_window_id: HashMap::new(),
            next_ref: 1,
            next_window: 1,
        }
    }

    pub fn clear(&mut self) {
        self.refs.clear();
        self.window_id_to_ref.clear();
        self.next_ref = 1;
    }

    pub fn rebuild(&mut self, windows: &[BackendWindow]) -> Vec<WindowInfo> {
        self.clear();

        let active_backend_ids = windows
            .iter()
            .map(|window| window.native_id)
            .collect::<HashSet<_>>();
        self.backend_id_to_window_id
            .retain(|backend_id, _| active_backend_ids.contains(backend_id));

        let mut public_windows = Vec::with_capacity(windows.len());
        for window in windows {
            let ref_id = format!("w{}", self.next_ref);
            self.next_ref += 1;

            let window_id = self.window_id_for_backend(window.native_id);
            let entry = RefEntry {
                window_id: window_id.clone(),
                backend_window_id: window.native_id,
                app_class: window.app_name.clone(),
                title: window.title.clone(),
                pid: 0,
                x: window.x,
                y: window.y,
                width: window.width,
                height: window.height,
                focused: window.focused,
                minimized: window.minimized,
            };

            self.window_id_to_ref
                .insert(window_id.clone(), ref_id.clone());
            self.refs.insert(ref_id.clone(), entry);
            public_windows.push(WindowInfo {
                ref_id,
                window_id,
                title: window.title.clone(),
                app_name: window.app_name.clone(),
                x: window.x,
                y: window.y,
                width: window.width,
                height: window.height,
                focused: window.focused,
                minimized: window.minimized,
            });
        }

        public_windows
    }

    fn window_id_for_backend(&mut self, backend_window_id: u32) -> String {
        if let Some(existing) = self.backend_id_to_window_id.get(&backend_window_id) {
            return existing.clone();
        }

        let window_id = format!("win{}", self.next_window);
        self.next_window += 1;
        self.backend_id_to_window_id
            .insert(backend_window_id, window_id.clone());
        window_id
    }

    /// Resolve a selector to a RefEntry.
    /// Accepts: "@w1", "w1", "ref=w1", "win1", "id=win1", or a substring match on app_class/title.
    pub fn resolve(&self, selector: &str) -> Option<&RefEntry> {
        let normalized = selector
            .strip_prefix('@')
            .or_else(|| selector.strip_prefix("ref="))
            .unwrap_or(selector);

        if let Some(entry) = self.refs.get(normalized) {
            return Some(entry);
        }

        let window_id = selector.strip_prefix("id=").unwrap_or(normalized);
        if let Some(ref_id) = self.window_id_to_ref.get(window_id) {
            return self.refs.get(ref_id);
        }

        let lower = selector.to_lowercase();
        self.refs.values().find(|entry| {
            entry.app_class.to_lowercase().contains(&lower)
                || entry.title.to_lowercase().contains(&lower)
        })
    }

    /// Resolve a selector to the center coordinates of the window.
    pub fn resolve_to_center(&self, selector: &str) -> Option<(i32, i32)> {
        self.resolve(selector)
            .map(|entry| (entry.x + entry.width as i32 / 2, entry.y + entry.height as i32 / 2))
    }

    pub fn entries(&self) -> impl Iterator<Item = (&String, &RefEntry)> {
        self.refs.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::RefMap;
    use crate::backend::BackendWindow;

    fn sample_window(native_id: u32, title: &str) -> BackendWindow {
        BackendWindow {
            native_id,
            title: title.to_string(),
            app_name: "TestApp".to_string(),
            x: 10,
            y: 20,
            width: 300,
            height: 200,
            focused: native_id == 1,
            minimized: false,
        }
    }

    #[test]
    fn rebuild_assigns_stable_window_ids_for_same_native_window() {
        let mut refs = RefMap::new();
        let first = refs.rebuild(&[sample_window(1, "First")]);
        let second = refs.rebuild(&[sample_window(1, "First Updated")]);

        assert_eq!(first[0].window_id, second[0].window_id);
        assert_eq!(second[0].ref_id, "w1");
    }

    #[test]
    fn resolve_accepts_ref_and_window_id() {
        let mut refs = RefMap::new();
        let public = refs.rebuild(&[sample_window(42, "Editor")]);
        let window_id = public[0].window_id.clone();

        assert_eq!(refs.resolve("@w1").unwrap().window_id, window_id);
        assert_eq!(refs.resolve(&window_id).unwrap().backend_window_id, 42);
        assert_eq!(refs.resolve(&format!("id={window_id}")).unwrap().title, "Editor");
    }

    #[test]
    fn resolve_to_center_uses_window_geometry() {
        let mut refs = RefMap::new();
        refs.rebuild(&[sample_window(7, "Browser")]);

        assert_eq!(refs.resolve_to_center("w1"), Some((160, 120)));
    }
}
