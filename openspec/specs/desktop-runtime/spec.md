## ADDED Requirements

### Requirement: Backend-neutral window identity
The desktop runtime SHALL expose a backend-neutral `window_id` for each enumerated window. Public runtime responses MUST NOT require backend-specific handles such as X11 window IDs for targeting or automation.

#### Scenario: Listing windows returns stable public identity
- **WHEN** a client lists windows through the runtime
- **THEN** each window result includes a `window_id` that the client can use for later targeting
- **AND** the result does not require the client to know the underlying X11 handle format

#### Scenario: Selector resolution accepts public identity
- **WHEN** a client targets a window by `window_id`
- **THEN** the runtime resolves the request to the correct live window
- **AND** performs the requested action without exposing backend handles in the public contract

### Requirement: Read-only window enumeration is side-effect free
The desktop runtime SHALL provide a read-only window enumeration path that does not capture screenshots and does not write screenshot artifacts unless a screenshot-producing command was explicitly requested.

#### Scenario: Listing windows does not write a screenshot
- **WHEN** a client runs a read-only window listing command
- **THEN** the runtime returns current window data
- **AND** does not capture a screenshot
- **AND** does not create a screenshot file as a side effect

#### Scenario: Snapshot remains an explicit screenshot-producing command
- **WHEN** a client runs a snapshot command
- **THEN** the runtime returns window data together with screenshot output
- **AND** any screenshot artifact is created only because the snapshot command explicitly requested it

### Requirement: Runtime diagnostics are first-class
The CLI SHALL provide a `doctor` command that checks runtime prerequisites for the supported X11 environment and reports actionable remediation guidance for each failed check.

#### Scenario: Doctor reports missing display configuration
- **WHEN** `deskctl doctor` runs without a usable `DISPLAY`
- **THEN** it reports that the X11 display is unavailable
- **AND** includes a concrete remediation message describing what environment setup is required

#### Scenario: Doctor verifies basic runtime operations
- **WHEN** `deskctl doctor` runs in a healthy supported environment
- **THEN** it verifies X11 connectivity, basic window enumeration, screenshot viability, and socket path health
- **AND** reports a successful diagnostic result for each check

### Requirement: Daemon startup failures are recoverable and diagnosable
The runtime SHALL detect stale daemon socket state and surface actionable startup or connection errors instead of failing with ambiguous transport errors.

#### Scenario: Client encounters a stale socket
- **WHEN** the client finds a socket path whose daemon is no longer serving requests
- **THEN** the runtime removes or replaces the stale socket state safely
- **AND** proceeds with a healthy daemon startup or reports a specific failure if recovery does not succeed

#### Scenario: Health probing distinguishes startup failure from runtime failure
- **WHEN** a client attempts to use the runtime and the daemon cannot become healthy
- **THEN** the returned error explains whether the failure occurred during spawn, health probing, or request handling
- **AND** does not report the problem as a generic connection failure alone
