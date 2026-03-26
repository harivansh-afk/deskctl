# deskctl

[![npm](https://img.shields.io/npm/v/deskctl-cli?label=npm)](https://www.npmjs.com/package/deskctl-cli)
[![release](https://img.shields.io/github/v/release/harivansh-afk/deskctl?label=release)](https://github.com/harivansh-afk/deskctl/releases)
[![runtime](https://img.shields.io/badge/runtime-linux--x11-111827)](#support-boundary)
[![skill](https://img.shields.io/badge/skills.sh-deskctl-111827)](skills/deskctl)

Non-interactive desktop control for AI agents on Linux X11.

## Install

```bash
npm install -g deskctl-cli
deskctl doctor
deskctl snapshot --annotate
```

One-shot execution also works:

```bash
npx deskctl-cli --help
```

`deskctl-cli` installs the `deskctl` command by downloading the matching GitHub Release asset for the supported runtime target.

## Installable skill

```bash
npx skills add harivansh-afk/deskctl -s deskctl
```

The installable skill lives in [`skills/deskctl`](skills/deskctl) and is built around the same observe -> wait -> act -> verify loop as the CLI.

## Quick example

```bash
deskctl doctor
deskctl snapshot --annotate
deskctl wait window --selector 'title=Firefox' --timeout 10
deskctl focus 'title=Firefox'
deskctl type "hello world"
```

## Docs

- runtime contract: [docs/runtime-contract.md](docs/runtime-contract.md)
- release flow: [docs/releasing.md](docs/releasing.md)
- installable skill: [skills/deskctl](skills/deskctl)
- contributor workflow: [CONTRIBUTING.md](CONTRIBUTING.md)

## Other install paths

Nix:

```bash
nix run github:harivansh-afk/deskctl -- --help
nix profile install github:harivansh-afk/deskctl
```

Source build:

```bash
cargo build
```

## Support boundary

`deskctl` currently supports Linux X11. Use `--json` for stable machine parsing, use `window_id` for programmatic targeting inside a live session, and use `deskctl doctor` first when the runtime looks broken.
