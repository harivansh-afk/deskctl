# Agent Guidelines

## Build

```bash
cargo build
cargo clippy
```

## Run

Requires an X11 session with `DISPLAY` set.

```bash
cargo run -- snapshot
cargo run -- --json snapshot --annotate
```

## Code Style

- No emojis in code or comments
- Use `anyhow::Result` for all fallible functions
- All daemon handler functions are async
- Match field naming between CLI args, NDJSON protocol, and JSON output

## Architecture

- `src/cli/` - clap CLI parser and client-side socket connection
- `src/daemon/` - tokio async daemon, request handler, state management
- `src/backend/` - DesktopBackend trait and X11 implementation
- `src/core/` - shared types, protocol, ref map, session detection

## Adding a New Command

1. Add the variant to `Command` enum in `src/cli/mod.rs`
2. Add request building in `build_request()` in `src/cli/mod.rs`
3. Add the action handler in `src/daemon/handler.rs`
4. Add the backend method to `DesktopBackend` trait in `src/backend/mod.rs`
5. Implement in `src/backend/x11.rs`
6. Update `SKILL.md`
