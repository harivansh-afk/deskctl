# deskctl runtime contract

All commands support `--json` and use the same top-level envelope:

```json
{
  "success": true,
  "data": {},
  "error": null
}
```

Use `--json` whenever you need to parse output programmatically.

## Stable window fields

Whenever a response includes a window payload, these fields are stable:

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

Use `window_id` for stable targeting inside a live daemon session. Use
`ref_id` or `@wN` for short-lived follow-up actions after `snapshot` or
`list-windows`.

## Stable grouped reads

- `deskctl get active-window` -> `data.window`
- `deskctl get monitors` -> `data.count`, `data.monitors`
- `deskctl get version` -> `data.version`, `data.backend`
- `deskctl get systeminfo` -> runtime-scoped diagnostic fields such as
  `backend`, `display`, `session_type`, `session`, `socket_path`, `screen`,
  `monitor_count`, and `monitors`

## Stable waits

- `deskctl wait window` -> `data.wait`, `data.selector`, `data.elapsed_ms`,
  `data.window`
- `deskctl wait focus` -> `data.wait`, `data.selector`, `data.elapsed_ms`,
  `data.window`

## Stable structured error kinds

When a command fails with structured JSON data, these `kind` values are stable:

- `selector_not_found`
- `selector_ambiguous`
- `selector_invalid`
- `timeout`
- `not_found`

Wait failures may also include `window_not_focused` in the last observation
payload.

## Best-effort fields

Treat these as useful but non-contractual:

- exact monitor names
- incidental text formatting in non-JSON mode
- default screenshot file names when no explicit path was provided
- environment-dependent ordering details from the window manager

For the full repo copy, see `docs/runtime-contract.md`.
