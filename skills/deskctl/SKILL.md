---
name: deskctl
description: Desktop control CLI for AI agents on Linux X11. Use when operating an X11 desktop in a sandbox, VM, or sandbox-agent session via screenshots, grouped get/wait commands, selectors, and mouse or keyboard input. Prefer this skill when the task is "control the desktop", "inspect windows", "wait for a window", "click/type in the sandbox desktop", or "use deskctl inside sandbox-agent".
allowed-tools: Bash(deskctl:*), Bash(npx deskctl-cli:*), Bash(npm:*), Bash(which:*), Bash(printenv:*), Bash(echo:*), Bash(sandbox-agent:*)
---

# deskctl

`deskctl` is a non-interactive desktop control CLI for Linux X11 agents. It works well inside sandbox-agent desktop environments because it gives agents a tight `observe -> wait -> act -> verify` loop.

## Install skill (optional)

### npx

```bash
npx skills add harivansh-afk/deskctl -s deskctl
```

### bunx

```bash
bunx skills add harivansh-afk/deskctl -s deskctl
```

## Install the CLI

Preferred install path:

```bash
npm install -g deskctl-cli
deskctl --help
```

If global npm installs are not writable, use a user prefix:

```bash
mkdir -p "$HOME/.local/bin"
npm install -g --prefix "$HOME/.local" deskctl-cli
export PATH="$HOME/.local/bin:$PATH"
deskctl --help
```

One-shot usage also works:

```bash
npx deskctl-cli --help
```

For install details and fallback paths, see [references/install.md](references/install.md).

## Sandbox-Agent Notes

Before using `deskctl` inside sandbox-agent:

1. Make sure the sandbox has desktop runtime packages installed.
2. Make sure the session is actually running X11.
3. Run `deskctl doctor` before trying to click or type.

Typical sandbox-agent prep:

```bash
sandbox-agent install desktop --yes
deskctl doctor
```

If `doctor` fails, inspect `DISPLAY`, `XDG_SESSION_TYPE`, and whether the sandbox actually has a desktop session. See [references/sandbox-agent.md](references/sandbox-agent.md).

## Core Workflow

Every desktop task should follow this loop:

1. **Observe**
2. **Target**
3. **Wait**
4. **Act**
5. **Verify**

```bash
deskctl doctor
deskctl snapshot --annotate
deskctl get active-window
deskctl wait window --selector 'class=firefox' --timeout 10
deskctl focus 'class=firefox'
deskctl hotkey ctrl l
deskctl type "https://example.com"
deskctl press enter
deskctl snapshot
```

## What To Reach For First

- `deskctl doctor`
- `deskctl snapshot --annotate`
- `deskctl list-windows`
- `deskctl get active-window`
- `deskctl wait window --selector ...`
- `deskctl wait focus --selector ...`

Use `--json` when you need strict parsing. Use explicit selectors when you need deterministic targeting.

## Selector Rules

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

Bare strings such as `firefox` are fuzzy substring selectors. They fail on ambiguity instead of silently picking the wrong window.

## References

- [references/install.md](references/install.md) - install paths, npm-first bootstrap, runtime prerequisites
- [references/commands.md](references/commands.md) - grouped reads, waits, selectors, and core action commands
- [references/sandbox-agent.md](references/sandbox-agent.md) - using `deskctl` inside sandbox-agent desktop sessions

## Templates

- [templates/install-deskctl-npm.sh](templates/install-deskctl-npm.sh) - install `deskctl-cli` into a user prefix
- [templates/sandbox-agent-desktop-loop.sh](templates/sandbox-agent-desktop-loop.sh) - minimal observe/wait/act loop for desktop tasks
