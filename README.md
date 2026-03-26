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
# Diagnose the environment first
deskctl doctor

# See the desktop
deskctl snapshot

# Query focused runtime state
deskctl get active-window
deskctl get monitors

# Click a window
deskctl click @w1

# Type text
deskctl type "hello world"

# Wait for a window or focus transition
deskctl wait window --selector 'title=Firefox' --timeout 10
deskctl wait focus --selector 'class=firefox' --timeout 5

# Focus by explicit selector
deskctl focus 'title=Firefox'
```

## Architecture

Client-daemon architecture over Unix sockets (NDJSON wire protocol). 
The daemon starts automatically on first command and keeps the X11 connection alive for fast repeated calls.

Source layout:

- `src/lib.rs` exposes the shared library target
- `src/main.rs` is the thin CLI wrapper
- `src/` contains production code and unit tests
- `tests/` contains Linux/X11 integration tests
- `tests/support/` contains shared integration helpers

## Runtime Requirements

- Linux with X11 session
- Rust 1.75+ (for build)

The binary itself only links the standard glibc runtime on Linux (`libc`, `libm`, `libgcc_s`).

For deskctl to be fully functional on a fresh VM you still need:

- an X11 server and an active `DISPLAY`
- `XDG_SESSION_TYPE=x11` or an equivalent X11 session environment
- a window manager or desktop environment that exposes standard EWMH properties such as `_NET_CLIENT_LIST_STACKING` and `_NET_ACTIVE_WINDOW`
- an X server with the extensions needed for input simulation and screen metadata, which is standard on normal desktop X11 setups

If setup fails, run:

```bash
deskctl doctor
```

## Contract Notes

- `@wN` refs are short-lived handles assigned by `snapshot` and `list-windows`
- `--json` output includes a stable `window_id` for programmatic targeting within the current daemon session
- `list-windows` is a cheap read-only operation and does not capture or write a screenshot
- the stable runtime JSON/error contract is documented in [docs/runtime-output.md](docs/runtime-output.md)

## Read and Wait Surface

The grouped runtime reads are:

```bash
deskctl get active-window
deskctl get monitors
deskctl get version
deskctl get systeminfo
```

The grouped runtime waits are:

```bash
deskctl wait window --selector 'title=Firefox' --timeout 10
deskctl wait focus --selector 'id=win3' --timeout 5
```

Successful `get active-window`, `wait window`, and `wait focus` responses return a `window` payload with:
- `ref_id`
- `window_id`
- `title`
- `app_name`
- geometry (`x`, `y`, `width`, `height`)
- state flags (`focused`, `minimized`)

`get monitors` returns:
- `count`
- `monitors[]` with geometry and primary/automatic flags

`get version` returns:
- `version`
- `backend`

`get systeminfo` stays runtime-scoped and returns:
- `backend`
- `display`
- `session_type`
- `session`
- `socket_path`
- `screen`
- `monitor_count`
- `monitors`

Wait timeout and selector failures are structured in `--json` mode so agents can recover without string parsing.

## Output Policy

Text mode is compact and follow-up-oriented, but JSON is the parsing contract.

- use `--json` when an agent needs strict parsing
- rely on `window_id`, selector-related fields, grouped read payloads, and structured error `kind` values for stable automation
- treat monitor naming, incidental whitespace, and default screenshot file names as best-effort

See [docs/runtime-output.md](docs/runtime-output.md) for the exact stable-vs-best-effort breakdown.

## Selector Contract

Explicit selector modes:

```bash
ref=w1
id=win1
title=Firefox
class=firefox
focused
```

Legacy refs remain supported:

```bash
@w1
w1
win1
```

Bare selectors such as `firefox` are still supported as fuzzy substring matches, but they now fail on ambiguity and return candidate windows instead of silently picking the first match.

## Support Boundary

`deskctl` supports Linux X11 in this phase. Wayland and Hyprland are explicitly out of scope for the current runtime contract.

## Workflow

Local validation uses the root `Makefile`:

```bash
make fmt-check
make lint
make test-unit
make test-integration
make site-format-check
make validate
```

`make validate` is the full repo-quality check and requires Linux with `xvfb-run` plus `pnpm --dir site install`.

The repository standardizes on `pre-commit` for fast commit-time checks:

```bash
pre-commit install
pre-commit run --all-files
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for the full contributor guide.

## Acknowledgements

- [@barrettruth](github.com/barrettruth) - i stole the website from [vimdoc](https://github.com/barrettruth/vimdoc-language-server)
