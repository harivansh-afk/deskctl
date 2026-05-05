# deskctl
[![npm](https://img.shields.io/npm/v/deskctl?label=npm)](https://www.npmjs.com/package/deskctl)
[![skill](https://img.shields.io/badge/skills.sh-deskctl-111827)](skills/deskctl)

Desktop control cli for AI agents on X11.

<video controls src="docs/assets/deskctl-demo.mp4"></video>


## Install

```bash
npm install -g deskctl
```

```bash
deskctl doctor
deskctl snapshot --annotate
```

## Skill

```bash
npx skills add harivansh-afk/deskctl
```

## Docs

- runtime contract: [docs/runtime-contract.md](docs/runtime-contract.md)
- releasing: [docs/releasing.md](docs/releasing.md)
- contributing: [CONTRIBUTING.md](CONTRIBUTING.md)

## Install paths

Nix:

```bash
nix run github:harivansh-afk/deskctl -- --help
nix profile install github:harivansh-afk/deskctl
```

Rust:

```bash
cargo build
```
