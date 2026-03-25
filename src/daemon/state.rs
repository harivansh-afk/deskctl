use std::path::PathBuf;
use crate::core::refs::RefMap;

#[allow(dead_code)]
pub struct DaemonState {
    pub session: String,
    pub socket_path: PathBuf,
    pub ref_map: RefMap,
}

impl DaemonState {
    pub fn new(session: String, socket_path: PathBuf) -> Self {
        Self {
            session,
            socket_path,
            ref_map: RefMap::new(),
        }
    }
}
