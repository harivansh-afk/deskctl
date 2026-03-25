## Context

`deskctl` already exposes a useful X11 runtime for screenshots, input, and window management, but the current implementation mixes together concerns that need to be separated before the repo can become a stable primitive. Public window data still exposes `xcb_id`, `list-windows` is routed through screenshot-producing code, setup failures are opaque, and daemon startup behavior is not yet explicit enough for reliable reuse by higher-level tooling.

Phase 1 is the foundation tranche. It should make the runtime contract clean and cheap to consume without expanding into packaging, skills, or non-X11 backends.

## Goals / Non-Goals

**Goals:**
- Define a backend-neutral public window identity and selector contract.
- Make read-only window enumeration cheap and side-effect free.
- Add a first-run diagnostics command that explains broken environments precisely.
- Harden daemon startup and health behavior enough for predictable CLI use.
- Keep the Phase 1 scope implementable in one focused change.

**Non-Goals:**
- Wayland support or any additional backend implementation.
- npm distribution, crates.io publishing, or release automation changes.
- Broad new read surface such as monitors, workspaces, clipboard, or batching.
- Agent skills, config files, or policy/confirmation features.

## Decisions

### 1. Public window identity becomes `window_id`, not `xcb_id`

The stable contract will expose an opaque `window_id` for programmatic targeting. Backend-specific handles such as X11 window IDs stay internal to daemon/backend state.

Rationale:
- This removes X11 leakage from the public interface.
- It keeps the future backend boundary open without promising a Wayland implementation now.
- It makes selector behavior explicit: users and agents target `@wN`, window names, or `window_id`, not backend handles.

Alternatives considered:
- Keep exposing `xcb_id` and add `window_id` alongside it. Rejected because it cements the wrong contract and encourages downstream dependence on X11 internals.
- Hide programmatic identity entirely and rely only on refs. Rejected because refs are intentionally ephemeral and not sufficient for durable automation.

### 2. Window enumeration and screenshot capture become separate backend operations

The backend API will separate:
- window listing / state collection
- screenshot capture
- composed snapshot behavior

`list-windows` will call the cheap enumeration path directly. `snapshot` remains the convenience command that combines enumeration plus screenshot generation.

Rationale:
- Read-only commands must not capture screenshots or write `/tmp` files.
- This reduces latency and unintended side effects.
- It clarifies which operations are safe to call frequently in agent loops.

Alternatives considered:
- Keep the current `snapshot(false)` path and optimize it internally. Rejected because the coupling itself is the product problem; the API shape needs to reflect the intended behavior.

### 3. `deskctl doctor` runs without requiring a healthy daemon

`doctor` will be implemented as a CLI command that can run before daemon startup. It will probe environment prerequisites directly and optionally inspect daemon/socket state as one of the checks.

Expected checks:
- `DISPLAY` present
- X11 session expectation (`XDG_SESSION_TYPE=x11` or explicit note if missing)
- X server connectivity
- required extensions or equivalent runtime capabilities used by the backend
- socket directory existence and permissions
- basic window enumeration
- screenshot capture viability

Rationale:
- Diagnostics must work when daemon startup is the thing that is broken.
- The command should return actionable failure messages, not “failed to connect to daemon.”

Alternatives considered:
- Implement `doctor` as a normal daemon request. Rejected because it hides the startup path and cannot diagnose stale socket or spawn failures well.

### 4. Daemon hardening stays minimal and focused in this phase

Phase 1 daemon work will cover:
- stale socket detection and cleanup on startup/connect
- clearer startup/connect failure messages
- a lightweight health-check request or equivalent status probe used by the client path

Rationale:
- This is enough to make CLI behavior predictable without turning the spec into a full daemon observability project.

Alternatives considered:
- Include idle timeout, structured logging, and full lifecycle policy now. Rejected because those are useful but not necessary to stabilize the basic runtime contract.

### 5. X11 remains the explicit support boundary for this change

The spec and implementation will define expected behavior for X11 environments only. Unsupported-session diagnostics are in scope; a second backend is not.

Rationale:
- The repo needs a stable foundation more than a nominally portable abstraction.
- Clear support boundaries are better than implying near-term Wayland support.

## Risks / Trade-offs

- [Breaking JSON contract for existing users] → Treat this as a deliberate pre-1.0 stabilization change, update docs/examples, and keep the new shape simple.
- [More internal mapping state between public IDs and backend handles] → Keep one canonical mapping in daemon state and use it for selector resolution.
- [Separate read and screenshot paths could drift] → Share the same window collection logic underneath both operations.
- [`doctor` checks may be environment-specific] → Keep the checks narrow, tied to actual backend requirements, and report concrete pass/fail reasons.

## Migration Plan

1. Introduce the new public `window_id` contract and update selector resolution to accept it.
2. Refactor backend and daemon code so `list-windows` uses a pure read path.
3. Add `deskctl doctor` and wire it into the CLI as a daemon-independent command.
4. Add unit and integration coverage for the new contract and cheap read behavior.
5. Update user-facing docs and examples after implementation so the new output shape is canonical.

Rollback strategy:
- This is a pre-1.0 contract cleanup, so rollback is a normal code revert rather than a compatibility shim plan.

## Open Questions

- Whether the health check should be an explicit public `daemon ping` action or an internal client/daemon probe.
- Whether `doctor` should expose machine-readable JSON on day one or land text output first and add JSON immediately afterward in the same tranche if implementation cost is low.
