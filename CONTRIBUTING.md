# Contributing

## Prerequisites

- Rust toolchain
- `make`
- `pre-commit` for commit-time hooks
- `pnpm` for site formatting checks
- Linux with `xvfb-run` for integration tests

Install site dependencies before running site checks:

```bash
pnpm --dir site install
```

## Repository Layout

- `src/lib.rs` exposes the library target used by integration tests
- `src/main.rs` is the thin CLI binary wrapper
- `src/` holds production code and unit tests
- `tests/` holds integration tests
- `tests/support/` holds shared X11 and daemon helpers for integration coverage
- `docs/runtime-output.md` is the stable-vs-best-effort runtime output contract for agent-facing CLI work

Keep integration-only helpers out of `src/`.

## Local Validation

The repo uses one local validation surface through `make`:

```bash
make fmt-check
make lint
make test-unit
make test-integration
make site-format-check
make validate
```

`make validate` runs the full Phase 2 validation stack. It requires Linux, `xvfb-run`, and site dependencies to be installed.

## Pre-commit Hooks

Install the hook workflow once:

```bash
pre-commit install
```

Run hooks across the repo on demand:

```bash
pre-commit run --all-files
```

The hook config intentionally stays small:

- Rust files use default `rustfmt`
- Site files reuse the existing `site/` Prettier setup
- Slower checks stay in CI or `make validate`

## Integration Tests

Integration coverage is Linux/X11-only in this phase. The supported local entrypoint is:

```bash
make test-integration
```

That command runs the top-level X11 integration tests under `xvfb-run` with one test thread so the shared display/session environment stays deterministic.
