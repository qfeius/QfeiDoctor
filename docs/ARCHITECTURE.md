# QfeiDoctor Architecture v0

## 1. Stack

- desktop shell: `Tauri`
- frontend: `React + TypeScript`
- diagnostic engine: `Rust`
- packaging: Tauri native bundles

## 2. Execution Model

v1 uses a single-process desktop model.

### Chosen approach

- Rust diagnostic engine runs in-process inside Tauri
- React UI calls Rust via `invoke(...)`

### Rejected for v1

- Go sidecar
- local microservice architecture

Reason:

- simpler installation
- simpler signing and distribution
- fewer process-lifecycle issues
- better fit for support-tool usage

## 3. Module Boundaries

### Frontend

Frontend responsibilities:

- target input and normalization UX
- render result sections
- render recommendations and quick actions
- copy structured JSON

Frontend must not invent a separate trace data model.

`DiagnosticTrace` is a view derived from canonical result sections.

### Rust backend

Rust responsibilities:

- run five diagnosis modules
- produce canonical result JSON
- derive recommended actions
- expose Windows quick-action URIs

## 4. Canonical Result Contract

The canonical top-level contract is:

```text
summary
dns
tcp
tls
http
system
recommended_actions
```

Anything else is derived presentation, not source-of-truth domain data.

## 5. Quick Actions

Quick actions are part of the result contract, not UI-only decorations.

Each quick action contains:

- `id`
- `label`
- `kind`
- `target`

For v1, `kind` is expected to be `open_uri`.

## 6. CI / Build Rules

Local developer commands must be:

- `make format`
- `make lint`
- `make test`

CI should converge to:

- `make format-check`
- `make lint`
- `make test`
- cross-platform build artifacts

Target artifacts:

- Windows: `.msi`, `.exe`
- macOS: `.dmg`
- Linux: `.deb`, `.AppImage`

## 7. Test Strategy

### Unit tests

- must be offline-stable
- must not depend on public Internet access

### Ignored integration / smoke tests

- allowed to use real network
- must not be in default `make test`

### Acceptance coverage

Acceptance is driven by SQE's matrix:

- 5 phases
- single-point anomalies
- selected combo anomalies
- evidence field mapping
- recommended-actions mapping

## 8. Current Constraints

- Windows is the v1 primary platform
- UI should stay close to the approved design reference
- no persistent history
- no backend service dependency
- no automatic mutation of system settings
