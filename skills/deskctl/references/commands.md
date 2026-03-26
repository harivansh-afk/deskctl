# deskctl command guide

## Observe

```bash
deskctl doctor
deskctl snapshot
deskctl snapshot --annotate
deskctl list-windows
deskctl screenshot /tmp/current.png
deskctl get active-window
deskctl get monitors
deskctl get version
deskctl get systeminfo
```

Use `snapshot --annotate` when you need both the screenshot artifact and the short `@wN` labels. Use `list-windows` when you only need the window tree and do not want screenshot side effects.

## Wait

```bash
deskctl wait window --selector 'title=Firefox' --timeout 10
deskctl wait focus --selector 'class=firefox' --timeout 5
```

Wait commands return the matched window payload on success. In `--json` mode, failures include structured `kind` values so the caller can recover without string parsing.

## Selectors

Prefer explicit selectors:

```bash
ref=w1
id=win1
title=Firefox
class=firefox
focused
```

Legacy refs still work:

```bash
@w1
w1
win1
```

Bare fuzzy selectors such as `firefox` are supported, but they fail on ambiguity.

## Act

```bash
deskctl focus 'class=firefox'
deskctl click @w1
deskctl dblclick @w2
deskctl type "hello world"
deskctl press enter
deskctl hotkey ctrl shift t
deskctl mouse move 500 300
deskctl mouse scroll 3
deskctl mouse drag 100 100 500 500
deskctl move-window @w1 100 120
deskctl resize-window @w1 1280 720
deskctl close @w3
deskctl launch firefox
```

## Agent loop

The safe pattern is:

1. Observe with `snapshot`, `list-windows`, or `get ...`
2. Wait for the target window if needed
3. Act using explicit selectors or refs
4. Snapshot again to verify the result
