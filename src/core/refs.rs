use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct RefEntry {
    pub xcb_id: u32,
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
    map: HashMap<String, RefEntry>,
    next_ref: usize,
}

#[allow(dead_code)]
impl RefMap {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
            next_ref: 1,
        }
    }

    pub fn clear(&mut self) {
        self.map.clear();
        self.next_ref = 1;
    }

    pub fn insert(&mut self, entry: RefEntry) -> String {
        let ref_id = format!("w{}", self.next_ref);
        self.next_ref += 1;
        self.map.insert(ref_id.clone(), entry);
        ref_id
    }

    /// Resolve a selector to a RefEntry.
    /// Accepts: "@w1", "w1", "ref=w1", or a substring match on app_class/title.
    pub fn resolve(&self, selector: &str) -> Option<&RefEntry> {
        let normalized = selector
            .strip_prefix('@')
            .or_else(|| selector.strip_prefix("ref="))
            .unwrap_or(selector);

        // Try direct ref lookup
        if let Some(entry) = self.map.get(normalized) {
            return Some(entry);
        }

        // Try substring match on app_class or title (case-insensitive)
        let lower = selector.to_lowercase();
        self.map.values().find(|e| {
            e.app_class.to_lowercase().contains(&lower) || e.title.to_lowercase().contains(&lower)
        })
    }

    /// Resolve a selector to the center coordinates of the window.
    pub fn resolve_to_center(&self, selector: &str) -> Option<(i32, i32)> {
        self.resolve(selector)
            .map(|e| (e.x + e.width as i32 / 2, e.y + e.height as i32 / 2))
    }

    pub fn entries(&self) -> impl Iterator<Item = (&String, &RefEntry)> {
        self.map.iter()
    }
}
