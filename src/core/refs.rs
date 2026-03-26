use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use crate::backend::BackendWindow;
use crate::core::types::WindowInfo;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct RefEntry {
    pub ref_id: String,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelectorQuery {
    Ref(String),
    WindowId(String),
    Title(String),
    Class(String),
    Focused,
    Fuzzy(String),
}

#[derive(Debug, Clone)]
pub enum ResolveResult {
    Match(RefEntry),
    NotFound {
        selector: String,
        mode: &'static str,
    },
    Ambiguous {
        selector: String,
        mode: &'static str,
        candidates: Vec<WindowInfo>,
    },
    Invalid {
        selector: String,
        mode: &'static str,
        message: String,
    },
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
                ref_id: ref_id.clone(),
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

    pub fn resolve(&self, selector: &str) -> ResolveResult {
        self.resolve_query(SelectorQuery::parse(selector), selector)
    }

    /// Resolve a selector to the center coordinates of the window.
    pub fn resolve_to_center(&self, selector: &str) -> ResolveResult {
        self.resolve(selector)
    }

    pub fn entries(&self) -> impl Iterator<Item = (&String, &RefEntry)> {
        self.refs.iter()
    }

    fn resolve_query(&self, query: SelectorQuery, selector: &str) -> ResolveResult {
        match query {
            SelectorQuery::Ref(ref_id) => self
                .refs
                .get(&ref_id)
                .cloned()
                .map(ResolveResult::Match)
                .unwrap_or_else(|| ResolveResult::NotFound {
                    selector: selector.to_string(),
                    mode: "ref",
                }),
            SelectorQuery::WindowId(window_id) => self
                .window_id_to_ref
                .get(&window_id)
                .and_then(|ref_id| self.refs.get(ref_id))
                .cloned()
                .map(ResolveResult::Match)
                .unwrap_or_else(|| ResolveResult::NotFound {
                    selector: selector.to_string(),
                    mode: "id",
                }),
            SelectorQuery::Focused => self.resolve_candidates(
                selector,
                "focused",
                self.refs
                    .values()
                    .filter(|entry| entry.focused)
                    .cloned()
                    .collect(),
            ),
            SelectorQuery::Title(title) => {
                if title.is_empty() {
                    return ResolveResult::Invalid {
                        selector: selector.to_string(),
                        mode: "title",
                        message: "title selectors must not be empty".to_string(),
                    };
                }
                self.resolve_candidates(
                    selector,
                    "title",
                    self.refs
                        .values()
                        .filter(|entry| entry.title.eq_ignore_ascii_case(&title))
                        .cloned()
                        .collect(),
                )
            }
            SelectorQuery::Class(app_class) => {
                if app_class.is_empty() {
                    return ResolveResult::Invalid {
                        selector: selector.to_string(),
                        mode: "class",
                        message: "class selectors must not be empty".to_string(),
                    };
                }
                self.resolve_candidates(
                    selector,
                    "class",
                    self.refs
                        .values()
                        .filter(|entry| entry.app_class.eq_ignore_ascii_case(&app_class))
                        .cloned()
                        .collect(),
                )
            }
            SelectorQuery::Fuzzy(value) => {
                if let Some(entry) = self.refs.get(&value).cloned() {
                    return ResolveResult::Match(entry);
                }

                if let Some(entry) = self
                    .window_id_to_ref
                    .get(&value)
                    .and_then(|ref_id| self.refs.get(ref_id))
                    .cloned()
                {
                    return ResolveResult::Match(entry);
                }

                let lower = value.to_lowercase();
                self.resolve_candidates(
                    selector,
                    "fuzzy",
                    self.refs
                        .values()
                        .filter(|entry| {
                            entry.app_class.to_lowercase().contains(&lower)
                                || entry.title.to_lowercase().contains(&lower)
                        })
                        .cloned()
                        .collect(),
                )
            }
        }
    }

    fn resolve_candidates(
        &self,
        selector: &str,
        mode: &'static str,
        mut candidates: Vec<RefEntry>,
    ) -> ResolveResult {
        candidates.sort_by(|left, right| left.ref_id.cmp(&right.ref_id));
        match candidates.len() {
            0 => ResolveResult::NotFound {
                selector: selector.to_string(),
                mode,
            },
            1 => ResolveResult::Match(candidates.remove(0)),
            _ => ResolveResult::Ambiguous {
                selector: selector.to_string(),
                mode,
                candidates: candidates
                    .into_iter()
                    .map(|entry| entry.to_window_info())
                    .collect(),
            },
        }
    }
}

impl SelectorQuery {
    pub fn parse(selector: &str) -> Self {
        if let Some(value) = selector.strip_prefix('@') {
            return Self::Ref(value.to_string());
        }
        if let Some(value) = selector.strip_prefix("ref=") {
            return Self::Ref(value.to_string());
        }
        if let Some(value) = selector.strip_prefix("id=") {
            return Self::WindowId(value.to_string());
        }
        if let Some(value) = selector.strip_prefix("title=") {
            return Self::Title(value.to_string());
        }
        if let Some(value) = selector.strip_prefix("class=") {
            return Self::Class(value.to_string());
        }
        if selector == "focused" {
            return Self::Focused;
        }
        Self::Fuzzy(selector.to_string())
    }

    pub fn needs_live_refresh(&self) -> bool {
        !matches!(self, Self::Ref(_))
    }
}

impl RefEntry {
    pub fn center(&self) -> (i32, i32) {
        (
            self.x + self.width as i32 / 2,
            self.y + self.height as i32 / 2,
        )
    }

    pub fn to_window_info(&self) -> WindowInfo {
        WindowInfo {
            ref_id: self.ref_id.clone(),
            window_id: self.window_id.clone(),
            title: self.title.clone(),
            app_name: self.app_class.clone(),
            x: self.x,
            y: self.y,
            width: self.width,
            height: self.height,
            focused: self.focused,
            minimized: self.minimized,
        }
    }
}

impl ResolveResult {
    pub fn matched_entry(&self) -> Option<&RefEntry> {
        match self {
            Self::Match(entry) => Some(entry),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{RefMap, ResolveResult, SelectorQuery};
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

        match refs.resolve("@w1") {
            ResolveResult::Match(entry) => assert_eq!(entry.window_id, window_id),
            other => panic!("unexpected resolve result: {other:?}"),
        }
        match refs.resolve(&window_id) {
            ResolveResult::Match(entry) => assert_eq!(entry.backend_window_id, 42),
            other => panic!("unexpected resolve result: {other:?}"),
        }
        match refs.resolve(&format!("id={window_id}")) {
            ResolveResult::Match(entry) => assert_eq!(entry.title, "Editor"),
            other => panic!("unexpected resolve result: {other:?}"),
        }
    }

    #[test]
    fn resolve_to_center_uses_window_geometry() {
        let mut refs = RefMap::new();
        refs.rebuild(&[sample_window(7, "Browser")]);

        match refs.resolve_to_center("w1") {
            ResolveResult::Match(entry) => assert_eq!(entry.center(), (160, 120)),
            other => panic!("unexpected resolve result: {other:?}"),
        }
    }

    #[test]
    fn selector_query_parses_explicit_modes() {
        assert_eq!(
            SelectorQuery::parse("@w1"),
            SelectorQuery::Ref("w1".to_string())
        );
        assert_eq!(
            SelectorQuery::parse("ref=w2"),
            SelectorQuery::Ref("w2".to_string())
        );
        assert_eq!(
            SelectorQuery::parse("id=win4"),
            SelectorQuery::WindowId("win4".to_string())
        );
        assert_eq!(
            SelectorQuery::parse("title=Chromium"),
            SelectorQuery::Title("Chromium".to_string())
        );
        assert_eq!(
            SelectorQuery::parse("class=Navigator"),
            SelectorQuery::Class("Navigator".to_string())
        );
        assert_eq!(SelectorQuery::parse("focused"), SelectorQuery::Focused);
    }

    #[test]
    fn resolve_supports_exact_title_class_and_focused_modes() {
        let mut refs = RefMap::new();
        refs.rebuild(&[
            sample_window(1, "Browser"),
            BackendWindow {
                native_id: 2,
                title: "Editor".to_string(),
                app_name: "Code".to_string(),
                x: 0,
                y: 0,
                width: 10,
                height: 10,
                focused: false,
                minimized: false,
            },
        ]);

        match refs.resolve("focused") {
            ResolveResult::Match(entry) => assert_eq!(entry.title, "Browser"),
            other => panic!("unexpected resolve result: {other:?}"),
        }
        match refs.resolve("title=Editor") {
            ResolveResult::Match(entry) => assert_eq!(entry.app_class, "Code"),
            other => panic!("unexpected resolve result: {other:?}"),
        }
        match refs.resolve("class=code") {
            ResolveResult::Match(entry) => assert_eq!(entry.title, "Editor"),
            other => panic!("unexpected resolve result: {other:?}"),
        }
    }

    #[test]
    fn fuzzy_resolution_fails_with_candidates_when_ambiguous() {
        let mut refs = RefMap::new();
        refs.rebuild(&[
            sample_window(1, "Chromium"),
            BackendWindow {
                native_id: 2,
                title: "Chromium Settings".to_string(),
                app_name: "Chromium".to_string(),
                x: 0,
                y: 0,
                width: 10,
                height: 10,
                focused: false,
                minimized: false,
            },
        ]);

        match refs.resolve("chromium") {
            ResolveResult::Ambiguous {
                mode, candidates, ..
            } => {
                assert_eq!(mode, "fuzzy");
                assert_eq!(candidates.len(), 2);
            }
            other => panic!("unexpected resolve result: {other:?}"),
        }
    }
}
