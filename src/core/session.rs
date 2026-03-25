use anyhow::{bail, Result};

pub enum SessionType {
    X11,
}

pub fn detect_session() -> Result<SessionType> {
    let session_type = std::env::var("XDG_SESSION_TYPE").unwrap_or_default();

    match session_type.as_str() {
        "x11" => {}
        "" => {
            // No XDG_SESSION_TYPE set - check for DISPLAY as fallback
            if std::env::var("DISPLAY").is_err() {
                bail!(
                    "No X11 session detected.\n\
                     XDG_SESSION_TYPE is not set and DISPLAY is not set.\n\
                     deskctl requires an X11 session."
                );
            }
        }
        "wayland" => {
            bail!(
                "Wayland session detected (XDG_SESSION_TYPE=wayland).\n\
                 deskctl currently supports X11 only."
            );
        }
        other => {
            bail!(
                "Unsupported session type: {other}\n\
                 deskctl currently supports X11 only."
            );
        }
    }

    // Confirm DISPLAY is set for X11
    if std::env::var("DISPLAY").is_err() {
        bail!(
            "X11 session detected but DISPLAY is not set.\n\
             Ensure your X server is running and DISPLAY is exported."
        );
    }

    Ok(SessionType::X11)
}
