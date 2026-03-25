## 1. Contract and protocol stabilization

- [ ] 1.1 Define the public `window_id` contract in shared types/protocol code and remove backend-handle assumptions from public runtime responses
- [ ] 1.2 Update daemon state and selector resolution to map `window_id` and refs to internal backend handles without exposing X11-specific IDs publicly
- [ ] 1.3 Update CLI text and JSON response handling to use the new public identity consistently

## 2. Cheap reads and diagnostics

- [ ] 2.1 Split backend window enumeration from screenshot capture and route `list-windows` through a read-only path with no screenshot side effects
- [ ] 2.2 Add a daemon-independent `deskctl doctor` command that probes X11 environment setup, socket health, window enumeration, and screenshot viability
- [ ] 2.3 Harden daemon startup and reconnect behavior with stale socket cleanup, health probing, and clearer failure messages

## 3. Validation and follow-through

- [ ] 3.1 Add unit tests for selector parsing, public ID resolution, and read-only behavior
- [ ] 3.2 Add X11 integration coverage for `doctor`, `list-windows`, and daemon recovery behavior
- [ ] 3.3 Update user-facing docs and examples to reflect the new contract, `doctor`, and the explicit X11 support boundary
