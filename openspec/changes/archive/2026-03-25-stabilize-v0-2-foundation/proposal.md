## Why

`deskctl` already works as a useful X11 desktop control CLI, but the current contract is not stable enough to build packaging, skills, or broader agent workflows on top of it yet. Public output still leaks X11-specific identifiers, some read-only commands still perform screenshot capture and write files, setup failures are not self-diagnosing, and daemon lifecycle behavior needs to be more predictable before the repo is treated as a reliable primitive.

## What Changes

- Stabilize the public desktop runtime contract around backend-neutral window identity and explicit selector semantics.
- Separate cheap read paths from screenshot-producing paths so read-only commands do not capture or write screenshots unless explicitly requested.
- Add a first-run `deskctl doctor` command that verifies X11 runtime prerequisites and reports exact remediation steps.
- Harden daemon startup and health behavior enough for reliable reuse from CLI commands and future higher-level tooling.
- Document the Phase 1 support boundary: X11 is the supported runtime today; Wayland is out of scope for this change.

## Capabilities

### New Capabilities
- `desktop-runtime`: Stable Phase 1 desktop runtime behavior covering public window identity, cheap read commands, runtime diagnostics, and foundational daemon health behavior.

### Modified Capabilities
- None.

## Impact

- Affected CLI surface in `src/cli/`
- Affected daemon request handling and state in `src/daemon/`
- Affected backend contract and X11 implementation in `src/backend/`
- Affected shared protocol and types in `src/core/`
- New tests for unit behavior and X11 integration coverage
- Follow-on docs updates for usage and troubleshooting once implementation lands
