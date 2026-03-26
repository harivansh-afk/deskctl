# deskctl inside sandbox-agent

Use `deskctl` when the sandbox-agent session includes a Linux desktop and you want a tight local desktop-control loop from the shell.

## When it fits

`deskctl` is a good fit when:

- the sandbox already has an X11 desktop session
- you want fast local desktop control from inside the sandbox
- you want short-lived refs like `@w1` and grouped `get` or `wait` primitives

It is not a replacement for sandbox-agent session orchestration itself. Use sandbox-agent to provision the sandbox and desktop runtime, then use `deskctl` inside that environment to control the GUI.

## Minimal bootstrap

```bash
sandbox-agent install desktop --yes
npm install -g deskctl-cli
deskctl doctor
deskctl snapshot --annotate
```

If npm global installs are not writable:

```bash
mkdir -p "$HOME/.local/bin"
npm install -g --prefix "$HOME/.local" deskctl-cli
export PATH="$HOME/.local/bin:$PATH"
deskctl doctor
```

## Expected environment

Check:

```bash
printenv DISPLAY
printenv XDG_SESSION_TYPE
deskctl --json get systeminfo
```

Healthy `deskctl` usage usually means:

- `DISPLAY` is set
- `XDG_SESSION_TYPE=x11`
- `deskctl doctor` succeeds

## Recommended workflow

```bash
deskctl snapshot --annotate
deskctl wait window --selector 'class=firefox' --timeout 10
deskctl focus 'class=firefox'
deskctl hotkey ctrl l
deskctl type "https://example.com"
deskctl press enter
deskctl snapshot
```

Prefer `--json` for strict machine parsing and explicit selectors for deterministic targeting.
