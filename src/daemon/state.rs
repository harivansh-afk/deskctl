use std::path::PathBuf;

use crate::backend::{x11::X11Backend, DesktopBackend};
use crate::core::refs::RefMap;

#[allow(dead_code)]
pub struct DaemonState {
    pub session: String,
    pub socket_path: PathBuf,
    pub ref_map: RefMap,
    pub backend: Box<dyn DesktopBackend>,
}

impl DaemonState {
    pub fn new(session: String, socket_path: PathBuf) -> anyhow::Result<Self> {
        let backend: Box<dyn DesktopBackend> = Box::new(X11Backend::new()?);
        Ok(Self {
            session,
            socket_path,
            ref_map: RefMap::new(),
            backend,
        })
    }
}
