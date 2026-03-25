# deskctl

Desktop control CLI for AI agents on Linux X11. 

## Install

```bash
cargo install deskctl
```

Build a Linux binary with Docker:

```bash
docker compose -f docker/docker-compose.yml run --rm build
```

This writes `dist/deskctl-linux-x86_64`.

Copy it to an SSH machine where `scp` is unavailable:

```bash
ssh -p 443 deskctl@ssh.agentcomputer.ai 'cat > ~/deskctl && chmod +x ~/deskctl' < dist/deskctl-linux-x86_64
```

Run it on an X11 session:

```bash
DISPLAY=:1 XDG_SESSION_TYPE=x11 ~/deskctl --json snapshot --annotate
```

Local source build requirements:
```bash
cargo build
```

At the moment there are no extra native build dependencies beyond a Rust toolchain.

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

## Runtime Requirements

- Linux with X11 session
- Rust 1.75+ (for build)

The binary itself only links the standard glibc runtime on Linux (`libc`, `libm`, `libgcc_s`).

For deskctl to be fully functional on a fresh VM you still need:

- an X11 server and an active `DISPLAY`
- `XDG_SESSION_TYPE=x11` or an equivalent X11 session environment
- a window manager or desktop environment that exposes standard EWMH properties such as `_NET_CLIENT_LIST_STACKING` and `_NET_ACTIVE_WINDOW`
- an X server with the extensions needed for input simulation and screen metadata, which is standard on normal desktop X11 setups

## Wayland Support

Coming soon. The trait-based backend design means adding Hyprland/Wayland support is a single trait implementation with zero refactoring of the core which is good.

## Acknowledgements

- [@barrettruth](github.com/barrettruth) - i stole the website from [vimdoc](https://github.com/barrettruth/vimdoc-language-server)
