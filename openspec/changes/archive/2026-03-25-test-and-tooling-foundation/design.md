## Context

Phase 1 stabilized the runtime contract, but the repo still lacks the structure and tooling needed to keep that contract stable as contributors add features. The current state is mixed:

- Rust checks exist in CI, but there is no explicit `cargo test` lane and no Xvfb integration lane.
- The repo has site-local Prettier config under `site/`, but no root-level contributor workflow for formatting or hooks.
- Integration-style tests are starting to appear, but the crate is still binary-first and does not yet have a clean top-level `tests/` structure.
- An empty `src/tests/` directory exists, which suggests the intended direction is not yet settled.

This change should establish the repo-quality foundation before Phase 3 agent features and Phase 4 distribution work expand the maintenance surface further.

## Goals / Non-Goals

**Goals:**
- Define a clean test architecture with a centralized top-level `tests/` directory for integration coverage.
- Introduce the crate structure needed for integration tests to import the project cleanly.
- Add a real Xvfb-backed CI lane and make CI validate the same commands contributors run locally.
- Define one formatting and hook workflow for the repo instead of ad hoc tool usage.
- Keep the site formatting story integrated without turning the entire repo into a Node-first project.

**Non-Goals:**
- New runtime capabilities such as `wait-for-window`, `version`, or broader read commands.
- npm distribution, crates.io publishing, or release automation changes.
- Introducing both Husky and pre-commit, or multiple competing hook systems.
- Adding `rustfmt.toml` unless we have a concrete non-default formatting requirement.

## Decisions

### 1. Convert the crate to library + binary

This change will introduce `src/lib.rs` and make `src/main.rs` a thin binary wrapper.

Rationale:
- Top-level Rust integration tests in `/tests` should import the crate cleanly.
- Shared test support and internal modules become easier to organize without abusing `src/main.rs`.
- This is the standard structure for a Rust project that needs both unit and integration coverage.

Alternatives considered:
- Keep the binary-only layout and continue placing all tests inside `src/`. Rejected because it makes integration coverage awkward and keeps test structure implicit.
- Put integration helpers in `src/tests` without a library target. Rejected because it preserves the binary-first coupling and does not create a clean external-test boundary.

### 2. Standardize on top-level `/tests` for integration coverage

Integration tests will live under a centralized `/tests` directory, with shared helpers under `tests/support/`.

Rationale:
- The runtime-facing flows are integration problems, not unit problems.
- A centralized `/tests` layout makes it clear which tests require Xvfb or daemon orchestration.
- It keeps `src/` focused on application code.

Alternatives considered:
- Keep helpers in `src/test_support.rs`. Rejected because it mixes production and integration concerns.

### 3. Standardize on `pre-commit`, not Husky

This change will define one hook system: `pre-commit`.

Rationale:
- The repo is Rust-first, not root-Node-managed.
- Husky would imply a root `package.json` and a Node-first workflow the repo does not currently have.
- `pre-commit` can run Rust and site checks without forcing the whole repo through npm.

Alternatives considered:
- Husky. Rejected because it introduces a root Node workflow for a repo that is not otherwise Node-based.
- Both Husky and pre-commit. Rejected because dual hook systems inevitably drift.
- No hooks. Rejected because contributor ergonomics and CI parity are explicit goals of this phase.

### 4. Keep formatting opinionated but minimal

This phase will use default `rustfmt` behavior and site-local Prettier behavior. A root `rustfmt.toml` will only be added if the implementation reveals a real non-default formatting need.

Rationale:
- A config file with no meaningful opinion is noise.
- What matters more is that CI and hooks actually run formatting checks.
- The site already has a working Prettier configuration; we should integrate it rather than duplicate it prematurely.

Alternatives considered:
- Add `rustfmt.toml` immediately. Rejected because there is no concrete formatting policy to encode yet.
- Add a root Prettier config for the whole repo. Rejected because it would broaden Node tooling scope before there is a clear need.

### 5. CI should call stable local entrypoints

This phase should define one local command surface for validation, and CI should call those same commands instead of hand-coded bespoke steps where practical.

Candidate checks:
- format check
- clippy
- unit tests
- Xvfb integration tests
- site formatting check

Rationale:
- Local/CI drift is one of the fastest ways to make an open source repo unpleasant to contribute to.
- Contributors should be able to run the same validation shape locally before pushing.

Alternatives considered:
- Keep all validation logic encoded only in GitHub Actions. Rejected because local parity matters.

## Risks / Trade-offs

- [Introducing `src/lib.rs` creates some churn] → Keep `main.rs` thin and preserve module names to minimize callsite disruption.
- [Xvfb CI can be flaky if test fixtures are underspecified] → Keep fixture windows simple and deterministic; avoid broad screenshot assertions early.
- [Pre-commit adds a Python-based contributor dependency] → Document installation clearly and keep hooks fast so the value exceeds the setup cost.
- [Formatting/tooling scope could sprawl into site build work] → Limit this phase to formatting and validation, not full site build architecture.

## Migration Plan

1. Introduce `src/lib.rs` and move reusable modules behind the library target.
2. Move integration support into top-level `tests/support/` and create real `/tests` coverage for Xvfb-backed flows.
3. Add local validation entrypoints for formatting, lint, and tests.
4. Add a root hook configuration using `pre-commit`.
5. Update CI to run unit tests, Xvfb integration tests, and relevant formatting checks on pull requests and main.
6. Update contributor docs so local validation, hooks, and test structure are discoverable.

Rollback strategy:
- This phase is repo-internal and pre-1.0, so rollback is a normal revert rather than a compatibility shim.

## Open Questions

- Whether the local validation entrypoint should be a `Justfile`, `Makefile`, or another lightweight wrapper.
- Whether site validation in this phase should be limited to Prettier checks or also include `astro check`.
