---
name: deskctl
description: Non-interactive X11 desktop control for AI agents. Use when the task involves controlling a Linux desktop - clicking, typing, reading windows, waiting for UI state, or taking screenshots inside a sandbox or VM.
allowed-tools: Bash(deskctl:*), Bash(npx deskctl:*), Bash(npm:*), Bash(which:*), Bash(printenv:*), Bash(echo:*)
---

# deskctl

Non-interactive desktop control CLI for Linux X11 agents.

All output follows the runtime contract defined in [references/runtime-contract.md](references/runtime-contract.md). Every command returns a stable JSON envelope when called with `--json`. Use `--json` whenever you need to parse output programmatically.

## Quick start

```bash
npm install -g deskctl
deskctl doctor
deskctl snapshot --annotate
```

If `deskctl` was installed through npm, refresh it later with:

```bash
deskctl upgrade --yes
```

## Agent loop

Every desktop interaction follows: **observe -> wait -> act -> verify**.

```bash
deskctl snapshot --annotate        # observe
deskctl wait window --selector 'title=Firefox' --timeout 10  # wait
deskctl click 'title=Firefox'      # act
deskctl snapshot                   # verify
```

See [workflows/observe-act.sh](workflows/observe-act.sh) for a reusable script. See [workflows/poll-condition.sh](workflows/poll-condition.sh) for polling loops.

## Selectors

```bash
ref=w1          # snapshot ref (short-lived)
id=win1         # stable window ID (session-scoped)
title=Firefox   # match by title
class=firefox   # match by WM class
focused         # currently focused window
```

Bare strings like `firefox` do fuzzy matching but fail on ambiguity. Prefer explicit selectors.

## References

- [references/runtime-contract.md](references/runtime-contract.md) - output contract, stable fields, error kinds
- [references/commands.md](references/commands.md) - all available commands

## Workflows

- [workflows/observe-act.sh](workflows/observe-act.sh) - main observe-act loop
- [workflows/poll-condition.sh](workflows/poll-condition.sh) - poll for a condition on screen
