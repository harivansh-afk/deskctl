---
name: desktop-ctl
description: Desktop control CLI for AI agents - screenshot, click, type, window management on Linux X11
allowed-tools: Bash(desktop-ctl:*)
---

# desktop-ctl

Desktop control CLI for AI agents on Linux X11. Provides a unified interface for screenshots, mouse/keyboard input, and window management with compact `@wN` window references.

## Core Workflow

1. **Snapshot** to see the desktop and get window refs
2. **Act** using refs or coordinates (click, type, focus)
3. **Repeat** as needed

## Quick Reference

### See the Desktop

```bash
desktop-ctl snapshot              # Screenshot + window tree with @wN refs
desktop-ctl snapshot --annotate   # Screenshot with bounding boxes and labels
desktop-ctl snapshot --json       # Structured JSON output
desktop-ctl list-windows          # Window tree without screenshot
desktop-ctl screenshot /tmp/s.png # Screenshot only (no window tree)
```

### Click and Type

```bash
desktop-ctl click @w1             # Click center of window @w1
desktop-ctl click 500,300         # Click absolute coordinates
desktop-ctl dblclick @w2          # Double-click window @w2
desktop-ctl type "hello world"    # Type text into focused window
desktop-ctl press enter           # Press a key
desktop-ctl hotkey ctrl c         # Send Ctrl+C
desktop-ctl hotkey ctrl shift t   # Send Ctrl+Shift+T
```

### Mouse Control

```bash
desktop-ctl mouse move 500 300    # Move cursor to coordinates
desktop-ctl mouse scroll 3        # Scroll down 3 units
desktop-ctl mouse scroll -3       # Scroll up 3 units
desktop-ctl mouse drag 100 100 500 500  # Drag from (100,100) to (500,500)
```

### Window Management

```bash
desktop-ctl focus @w2             # Focus window by ref
desktop-ctl focus "firefox"       # Focus window by name (substring match)
desktop-ctl close @w3             # Close window gracefully
desktop-ctl move-window @w1 100 200     # Move window to position
desktop-ctl resize-window @w1 800 600   # Resize window
```

### Utilities

```bash
desktop-ctl get-screen-size       # Screen resolution
desktop-ctl get-mouse-position    # Current cursor position
desktop-ctl launch firefox        # Launch an application
desktop-ctl launch code -- --new-window  # Launch with arguments
```

### Daemon

```bash
desktop-ctl daemon start          # Start daemon manually
desktop-ctl daemon stop           # Stop daemon
desktop-ctl daemon status         # Check daemon status
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
- Use `--json` to see stable `xcb_id` for programmatic tracking

## Example Agent Workflow

```bash
# 1. See what's on screen
desktop-ctl snapshot --annotate

# 2. Focus the browser
desktop-ctl focus "firefox"

# 3. Navigate to a URL
desktop-ctl hotkey ctrl l
desktop-ctl type "https://example.com"
desktop-ctl press enter

# 4. Take a new snapshot to see the result
desktop-ctl snapshot
```

## Key Names for press/hotkey

Modifiers: `ctrl`, `alt`, `shift`, `super`
Navigation: `enter`, `tab`, `escape`, `backspace`, `delete`, `space`
Arrows: `up`, `down`, `left`, `right`
Page: `home`, `end`, `pageup`, `pagedown`
Function: `f1` through `f12`
Characters: any single character (e.g. `a`, `1`, `/`)
