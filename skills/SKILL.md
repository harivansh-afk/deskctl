---
name: deskctl
description: Desktop control CLI for AI agents 
allowed-tools: Bash(deskctl:*)
---

# deskctl

Desktop control CLI for AI agents on Linux X11. Provides a unified interface for screenshots, mouse/keyboard input, and window management with compact `@wN` window references.

## Core Workflow

1. **Snapshot** to see the desktop and get window refs
2. **Query / wait** using grouped `get` and `wait` commands
3. **Act** using refs, explicit selectors, or coordinates
4. **Repeat** as needed

## Quick Reference

### See the Desktop

```bash
deskctl snapshot              # Screenshot + window tree with @wN refs
deskctl snapshot --annotate   # Screenshot with bounding boxes and labels
deskctl snapshot --json       # Structured JSON output
deskctl list-windows          # Window tree without screenshot
deskctl screenshot /tmp/s.png # Screenshot only (no window tree)
deskctl get active-window     # Currently focused window
deskctl get monitors          # Monitor geometry
deskctl get version           # deskctl version + backend
deskctl get systeminfo        # Runtime-scoped diagnostics
deskctl wait window --selector 'title=Firefox' --timeout 10
deskctl wait focus --selector 'class=firefox' --timeout 5
```

### Click and Type

```bash
deskctl click @w1             # Click center of window @w1
deskctl click 500,300         # Click absolute coordinates
deskctl dblclick @w2          # Double-click window @w2
deskctl type "hello world"    # Type text into focused window
deskctl press enter           # Press a key
deskctl hotkey ctrl c         # Send Ctrl+C
deskctl hotkey ctrl shift t   # Send Ctrl+Shift+T
```

### Mouse Control

```bash
deskctl mouse move 500 300    # Move cursor to coordinates
deskctl mouse scroll 3        # Scroll down 3 units
deskctl mouse scroll -3       # Scroll up 3 units
deskctl mouse drag 100 100 500 500  # Drag from (100,100) to (500,500)
```

### Window Management

```bash
deskctl focus @w2             # Focus window by ref
deskctl focus 'title=Firefox' # Focus by explicit title selector
deskctl focus 'class=firefox' # Focus by explicit class selector
deskctl focus "firefox"       # Fuzzy substring match (fails on ambiguity)
deskctl close @w3             # Close window gracefully
deskctl move-window @w1 100 200     # Move window to position
deskctl resize-window @w1 800 600   # Resize window
```

### Utilities

```bash
deskctl doctor                # Diagnose X11, screenshot, and daemon health
deskctl get-screen-size       # Screen resolution
deskctl get-mouse-position    # Current cursor position
deskctl launch firefox        # Launch an application
deskctl launch code -- --new-window  # Launch with arguments
```

### Daemon

```bash
deskctl daemon start          # Start daemon manually
deskctl daemon stop           # Stop daemon
deskctl daemon status         # Check daemon status
```

## Global Options

- `--json` : Output as structured JSON (all commands)
- `--session NAME` : Session name for multiple daemon instances (default: "default")
- `--socket PATH` : Custom Unix socket path

## Window Refs

After `snapshot` or `list-windows`, windows are assigned short refs:
- `@w1` is the topmost (usually focused) window
- `@w2`, `@w3`, etc. follow z-order (front to back)
- Refs reset on each `snapshot` call
- Use `--json` to see stable `window_id` values for programmatic tracking within the current daemon session

## Selector Contract

Prefer explicit selectors when an agent needs deterministic targeting:

```bash
ref=w1
id=win1
title=Firefox
class=firefox
focused
```

Bare selectors such as `firefox` still work as fuzzy substring matches, but they now fail with candidate windows if multiple matches exist.

## Example Agent Workflow

```bash
# 1. See what's on screen
deskctl snapshot --annotate

# 2. Wait for the browser and focus it deterministically
deskctl wait window --selector 'class=firefox' --timeout 10
deskctl focus 'class=firefox'

# 3. Navigate to a URL
deskctl hotkey ctrl l
deskctl type "https://example.com"
deskctl press enter

# 4. Take a new snapshot to see the result
deskctl snapshot
```

## Key Names for press/hotkey

Modifiers: `ctrl`, `alt`, `shift`, `super`
Navigation: `enter`, `tab`, `escape`, `backspace`, `delete`, `space`
Arrows: `up`, `down`, `left`, `right`
Page: `home`, `end`, `pageup`, `pagedown`
Function: `f1` through `f12`
Characters: any single character (e.g. `a`, `1`, `/`)
