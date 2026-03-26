# deskctl commands

All commands support `--json` for machine-parseable output following the runtime contract.

## Observe

```bash
deskctl doctor                          # check X11 runtime and daemon health
deskctl snapshot                        # screenshot + window list
deskctl snapshot --annotate             # screenshot with @wN labels overlaid
deskctl list-windows                    # window list only (no screenshot)
deskctl screenshot /tmp/screen.png      # screenshot to explicit path
deskctl get active-window               # focused window info
deskctl get monitors                    # monitor geometry
deskctl get version                     # version and backend
deskctl get systeminfo                  # full runtime diagnostics
deskctl get-screen-size                 # screen resolution
deskctl get-mouse-position              # cursor coordinates
```

## Wait

```bash
deskctl wait window --selector 'title=Firefox' --timeout 10
deskctl wait focus --selector 'class=firefox' --timeout 5
```

Returns the matched window payload on success. Failures include structured `kind` values in `--json` mode.

## Selectors

```bash
ref=w1          # snapshot ref (short-lived, from last snapshot)
id=win1         # stable window ID (session-scoped)
title=Firefox   # match by window title
class=firefox   # match by WM class
focused         # currently focused window
```

Legacy shorthand: `@w1`, `w1`, `win1`. Bare strings do fuzzy matching but fail on ambiguity.

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
deskctl mouse scroll 3 --axis horizontal
deskctl mouse drag 100 100 500 500
deskctl move-window @w1 100 120
deskctl resize-window @w1 1280 720
deskctl close @w3
deskctl launch firefox
```

## Daemon

```bash
deskctl daemon start
deskctl daemon stop
deskctl daemon status
```

The daemon starts automatically on first command. Manual control is rarely needed.
