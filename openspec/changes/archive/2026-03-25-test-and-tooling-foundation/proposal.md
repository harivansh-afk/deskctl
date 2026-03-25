## Why

`deskctl` now has a decent Phase 1 runtime contract, but the repo still lacks the test architecture and local tooling discipline needed to keep that contract stable as the project grows. Before moving into broader agent primitives or distribution polish, the repo needs a real integration-test story, CI coverage that exercises it, and one coherent developer workflow for formatting and pre-commit checks.

## What Changes

- Introduce a proper repository quality foundation for testing, CI, and local developer tooling.
- Establish a centralized top-level integration test layout and the crate structure needed to support it cleanly.
- Add an explicit Xvfb-backed CI lane for runtime integration tests.
- Define one local formatting and hook workflow for Rust and site content instead of ad hoc tool usage.
- Add contributor-facing commands and docs for running the same checks locally that CI will enforce.

## Capabilities

### New Capabilities
- `repo-quality`: Repository-level quality guarantees covering test architecture, CI validation, formatting, and local hook workflow.

### Modified Capabilities
- None.

## Impact

- Rust crate layout in `src/` and likely a new `src/lib.rs`
- New top-level `tests/` structure and shared integration test support
- GitHub Actions workflow(s) under `.github/workflows/`
- Root-level contributor tooling files such as `.pre-commit-config.yaml` and related local task entrypoints
- Site formatting integration for files under `site/`
- README and contributor documentation describing local validation workflows
