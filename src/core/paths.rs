use std::path::PathBuf;

pub fn socket_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("DESKCTL_SOCKET_DIR") {
        return PathBuf::from(dir);
    }
    if let Ok(runtime) = std::env::var("XDG_RUNTIME_DIR") {
        return PathBuf::from(runtime).join("deskctl");
    }
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join(".deskctl")
}

pub fn socket_path_for_session(session: &str) -> PathBuf {
    socket_dir().join(format!("{session}.sock"))
}

pub fn pid_path_for_session(session: &str) -> PathBuf {
    socket_dir().join(format!("{session}.pid"))
}

pub fn socket_path_from_env() -> Option<PathBuf> {
    std::env::var("DESKCTL_SOCKET_PATH").ok().map(PathBuf::from)
}

pub fn pid_path_from_env() -> Option<PathBuf> {
    std::env::var("DESKCTL_PID_PATH").ok().map(PathBuf::from)
}
