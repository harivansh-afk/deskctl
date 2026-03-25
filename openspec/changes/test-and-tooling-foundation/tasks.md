## 1. Repository Structure

- [ ] 1.1 Introduce `src/lib.rs`, keep `src/main.rs` as a thin binary wrapper, and preserve existing module boundaries so integration tests can import the crate cleanly
- [ ] 1.2 Move integration-only helpers out of production modules into `tests/support/` and remove the unused `src/tests` direction so the test layout is unambiguous
- [ ] 1.3 Add top-level integration tests under `tests/` that exercise at least diagnostics, window enumeration, and daemon startup/recovery flows through the library target

## 2. Local Validation Tooling

- [ ] 2.1 Add one documented local validation entrypoint for formatting, linting, unit tests, integration tests, and site formatting checks
- [ ] 2.2 Add a root `.pre-commit-config.yaml` that standardizes on `pre-commit` for fast commit-time checks without introducing a root Node workflow
- [ ] 2.3 Keep formatting configuration minimal by using default `rustfmt`, reusing the existing site-local Prettier setup, and only adding new config where implementation requires it

## 3. CI Hardening

- [ ] 3.1 Update GitHub Actions to validate pull requests in addition to `main` pushes and to run the same validation categories contributors run locally
- [ ] 3.2 Add an explicit Xvfb-backed CI lane that runs the integration tests covering diagnostics, window enumeration, and daemon recovery behavior
- [ ] 3.3 Ensure CI also runs the repository's formatting, clippy, unit test, and site formatting checks through the shared local entrypoints where practical

## 4. Documentation

- [ ] 4.1 Update contributor-facing docs to explain the new crate/test layout, including where integration tests and shared helpers live
- [ ] 4.2 Document the local validation workflow and `pre-commit` installation/use so contributors can reproduce CI expectations locally
- [ ] 4.3 Update the Phase 2 planning/docs references so the repo-quality foundation clearly lands before later packaging and distribution phases
