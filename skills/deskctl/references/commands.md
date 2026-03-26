# deskctl commands

All commands support `--json` for machine-parseable output following the
runtime contract.

## Observe

```bash
deskctl doctor
deskctl upgrade
deskctl snapshot
deskctl snapshot --annotate
deskctl list-windows
deskctl screenshot /tmp/screen.png
deskctl get active-window
deskctl get monitors
deskctl get version
deskctl get systeminfo
deskctl get-screen-size
deskctl get-mouse-position
```

## Wait

```bash
deskctl wait window --selector 'title=Firefox' --timeout 10
deskctl wait focus --selector 'class=firefox' --timeout 5
```

Returns the matched window payload on success. Failures include structured
`kind` values in `--json` mode.

## Selectors

```bash
ref=w1
id=win1
title=Firefox
class=firefox
focused
```

Legacy shorthand: `@w1`, `w1`, `win1`. Bare strings do fuzzy matching but fail
on ambiguity.

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

The daemon starts automatically on first command. In normal usage you should
not need to manage it directly.
