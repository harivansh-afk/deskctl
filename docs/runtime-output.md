# Runtime Output Contract

This document defines the current output contract for `deskctl`.

It is intentionally scoped to the current Linux X11 runtime surface.
It does not promise stability for future Wayland or window-manager-specific features.

## Goals

- Keep `deskctl` fully non-interactive
- Make text output actionable for quick terminal and agent loops
- Make `--json` safe for agent consumption without depending on incidental formatting

## JSON Envelope

Every runtime command uses the same top-level JSON envelope:

```json
{
  "success": true,
  "data": {},
  "error": null
}
```

Stable top-level fields:

- `success`
- `data`
- `error`

`success` is always the authoritative success/failure bit.
When `success` is `false`, the CLI exits non-zero in both text mode and `--json` mode.

## Stable Fields

These fields are stable for agent consumption in the current Phase 1 runtime contract.

### Window Identity

Whenever a runtime response includes a window payload, these fields are stable:

- `ref_id`
- `window_id`
- `title`
- `app_name`
- `x`
- `y`
- `width`
- `height`
- `focused`
- `minimized`

`window_id` is the stable public identifier for a live daemon session.
`ref_id` is a short-lived convenience handle for the current window snapshot/ref map.

### Grouped Reads

`deskctl get active-window`

- stable: `data.window`

`deskctl get monitors`

- stable: `data.count`
- stable: `data.monitors`
- stable per monitor:
  - `name`
  - `x`
  - `y`
  - `width`
  - `height`
  - `width_mm`
  - `height_mm`
  - `primary`
  - `automatic`

`deskctl get version`

- stable: `data.version`
- stable: `data.backend`

`deskctl get systeminfo`

- stable: `data.backend`
- stable: `data.display`
- stable: `data.session_type`
- stable: `data.session`
- stable: `data.socket_path`
- stable: `data.screen`
- stable: `data.monitor_count`
- stable: `data.monitors`

### Waits

`deskctl wait window`
`deskctl wait focus`

- stable: `data.wait`
- stable: `data.selector`
- stable: `data.elapsed_ms`
- stable: `data.window`

### Selector-Driven Action Success

For selector-driven action commands that resolve a window target, these identifiers are stable when present:

- `data.ref_id`
- `data.window_id`
- `data.title`
- `data.selector`

This applies to:

- `click`
- `dblclick`
- `focus`
- `close`
- `move-window`
- `resize-window`

The exact human-readable text rendering of those commands is not part of the JSON contract.

### Artifact-Producing Commands

`snapshot`
`screenshot`

- stable: `data.screenshot`

When the command also returns windows, `data.windows` uses the stable window payload documented above.

## Stable Structured Error Kinds

When a runtime command returns structured JSON failure data, these error kinds are stable:

- `selector_not_found`
- `selector_ambiguous`
- `selector_invalid`
- `timeout`
- `not_found`
- `window_not_focused` as `data.last_observation.kind` or equivalent observation payload

Stable structured failure fields include:

- `data.kind`
- `data.selector` when selector-related
- `data.mode` when selector-related
- `data.candidates` for ambiguous selector failures
- `data.message` for invalid selector failures
- `data.wait`
- `data.timeout_ms`
- `data.poll_ms`
- `data.last_observation`

## Best-Effort Fields

These values are useful but environment-dependent and should be treated as best-effort:

- exact monitor naming conventions
- EWMH/window-manager-dependent window ordering details
- cosmetic text formatting in non-JSON mode
- screenshot file names when the caller did not provide an explicit path
- command stderr wording outside the structured `kind` classifications above

## Text Mode Expectations

Text mode is intended to stay compact and follow-up-useful.

The exact whitespace/alignment of text output is not stable.
The following expectations are stable at the behavioral level:

- important runtime reads print actionable identifiers or geometry
- selector failures print enough detail to recover without `--json`
- artifact-producing commands print the artifact path
- window listings print both `@wN` refs and `window_id` values

If an agent needs strict parsing, it should use `--json`.
