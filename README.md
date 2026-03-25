# deskctl

Desktop control CLI for AI agents on Linux X11. 

## Install

```bash
cargo install deskctl
```

System deps (Debian/Ubuntu):
```bash
sudo apt install libxcb-dev libxrandr-dev libclang-dev
```

## Quick Start

```bash
# See the desktop
deskctl snapshot

# Click a window
deskctl click @w1

# Type text
deskctl type "hello world"

# Focus by name
deskctl focus "firefox"
```

## Architecture

Client-daemon architecture over Unix sockets (NDJSON wire protocol). 
The daemon starts automatically on first command and keeps the X11 connection alive for fast repeated calls.

## Requirements

- Linux with X11 session
- Rust 1.75+ (for build)

## Wayland Support

Coming soon hopefully. The trait-based backend design means adding Hyprland/Wayland support is a single trait implementation with zero refactoring of the core which is good.
