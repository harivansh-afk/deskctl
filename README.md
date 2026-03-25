# desktop-ctl

Desktop control CLI for AI agents on Linux X11. A single installable binary that gives agents full desktop access: screenshots with window refs, mouse/keyboard input, and window management.

Inspired by [agent-browser](https://github.com/vercel-labs/agent-browser) - but for the full desktop.

## Install

```bash
cargo install desktop-ctl
```

System dependencies (Debian/Ubuntu):
```bash
sudo apt install libxcb-dev libxrandr-dev libclang-dev
```

## Quick Start

```bash
# See the desktop
desktop-ctl snapshot

# Click a window
desktop-ctl click @w1

# Type text
desktop-ctl type "hello world"

# Focus by name
desktop-ctl focus "firefox"
```

## Architecture

Client-daemon architecture over Unix sockets (NDJSON wire protocol). The daemon starts automatically on first command and keeps the X11 connection alive for fast repeated calls.

```
Agent -> desktop-ctl CLI (thin client) -> Unix socket -> desktop-ctl daemon -> X11
```

## Requirements

- Linux with X11 session
- Rust 1.75+ (for building)

## Wayland Support

Coming in v0.2. The trait-based backend design means adding Hyprland/Wayland support is a single trait implementation with zero refactoring of the core.

## License

MIT OR Apache-2.0
