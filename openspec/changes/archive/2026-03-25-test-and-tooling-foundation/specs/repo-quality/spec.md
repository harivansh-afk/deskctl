## ADDED Requirements

### Requirement: Repository exposes a clean integration test architecture
The repository SHALL provide a top-level integration test architecture that allows runtime flows to be tested outside in-module unit tests.

#### Scenario: Integration tests import the crate cleanly
- **WHEN** an integration test under `/tests` needs to exercise repository code
- **THEN** it imports the project through a library target rather than depending on binary-only wiring

#### Scenario: Integration helpers are centralized
- **WHEN** multiple integration tests need shared X11 or daemon helpers
- **THEN** those helpers live in a shared test-support location under `/tests`
- **AND** production code does not need to host integration-only support files

### Requirement: CI validates real X11 integration behavior
The repository SHALL run Xvfb-backed integration coverage in CI for the supported X11 runtime.

#### Scenario: Pull requests run X11 integration tests
- **WHEN** a pull request modifies runtime or test-relevant code
- **THEN** CI runs the repository's Xvfb-backed integration test lane
- **AND** fails the change if the integration lane does not pass

#### Scenario: Integration lane covers core runtime flows
- **WHEN** the Xvfb integration lane runs
- **THEN** it exercises at least runtime diagnostics, window enumeration, and daemon startup/recovery behavior

### Requirement: Repository defines one local validation workflow
The repository SHALL define one coherent local validation workflow that contributors can run before pushing and that CI can mirror.

#### Scenario: Local formatting and linting entrypoints are documented
- **WHEN** a contributor wants to validate a change locally
- **THEN** the repository provides documented commands for formatting, linting, unit tests, and integration tests

#### Scenario: CI and local validation stay aligned
- **WHEN** CI validates the repository
- **THEN** it uses the same validation categories that contributors are expected to run locally
- **AND** avoids introducing a separate undocumented CI-only workflow

### Requirement: Repository uses a single hook system
The repository SHALL standardize on one pre-commit hook workflow for contributor checks.

#### Scenario: Hook workflow does not require root Node ownership
- **WHEN** a contributor installs the hook workflow
- **THEN** the repository can run Rust and site checks without requiring a root-level Node package workflow

#### Scenario: Hook scope stays fast and focused
- **WHEN** the pre-commit hook runs
- **THEN** it executes only fast checks appropriate for commit-time feedback
- **AND** slower validation remains in pre-push or CI lanes
