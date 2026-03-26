# Releasing deskctl

This document covers the operator flow for shipping `deskctl` across:

- GitHub Releases
- crates.io
- npm
- the repo flake

GitHub Releases are the canonical binary source. The npm package consumes those release assets instead of building a separate binary.

## Package Names

- crate: `deskctl`
- npm package: `deskctl-cli`
- installed command: `deskctl`

## Prerequisites

Before the first live publish on each registry:

- npm ownership for `deskctl-cli`
- crates.io ownership for `deskctl`
- repository secrets:
  - `NPM_TOKEN`
  - `CARGO_REGISTRY_TOKEN`

These are user-owned prerequisites. The repo can validate and automate the rest, but it cannot create registry ownership for you.

## Normal Release Flow

1. Merge release-ready changes to `main`.
2. Let CI run:
   - validation
   - integration
   - distribution validation
   - release asset build
3. Confirm the GitHub Release exists for the version tag and includes:
   - `deskctl-linux-x86_64`
   - `checksums.txt`
4. Trigger the `Publish Registries` workflow with:
   - `tag`
   - `publish_npm`
   - `publish_crates`
5. Confirm the publish summary for each channel.

## What CI Validates

The repository validates:

- `cargo publish --dry-run --locked`
- npm package metadata and packability
- npm install smoke path on Linux using the packaged `deskctl` command
- repo flake evaluation/build

The repository release workflow:

- builds the Linux release binary
- publishes the canonical GitHub Release asset
- uploads `checksums.txt`

The registry publish workflow:

- targets an existing release tag
- checks that Cargo, npm, and the requested tag all agree on version
- checks whether that version is already published on npm and crates.io
- only publishes the channels explicitly requested

## Rerun Safety

Registry publishing is intentionally separate from release asset creation.

If a partial failure happens:

- GitHub Release assets remain the source of truth
- rerun the `Publish Registries` workflow for the same tag
- already-published channels are reported and skipped
- remaining channels can still be published

## Local Validation

Run the distribution checks locally with:

```bash
make cargo-publish-dry-run
make npm-package-check
make nix-flake-check
make dist-validate
```

Notes:

- `make npm-package-check` does a runtime smoke test only on Linux
- `make nix-flake-check` requires a local Nix installation
- Docker remains a local Linux build convenience, not the canonical release path

## Nix Boundary

The repo-owned `flake.nix` is the supported Nix surface in this phase.

In scope:

- `nix run github:harivansh-afk/deskctl`
- `nix profile install github:harivansh-afk/deskctl`
- CI validation for the repo flake

Out of scope for this phase:

- `nixpkgs` upstreaming
- extra distro packaging outside the repo
