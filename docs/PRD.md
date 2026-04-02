# QfeiDoctor PRD v0

## 1. Product Definition

`QfeiDoctor` is a Windows-first desktop app for after-sales/support staff to diagnose website access problems.

The product goal is not generic operations management. The goal is to turn vague user reports like "the website cannot be accessed" into a structured, reproducible, engineer-readable diagnosis.

Core value chain:

1. Support staff enters a `URL` or `domain`
2. App runs a layered diagnosis locally
3. App shows clear findings and recommended actions
4. App copies a structured JSON report for engineering follow-up

## 2. Users

### Primary user

- after-sales / support staff

### Secondary reader

- engineers reading copied JSON output

## 3. Non-Goals for v1

- no persistent history
- no report export files
- no remote backend service
- no automatic system mutation
- no generic observability platform positioning
- no multi-target batch diagnosis

## 4. Supported Input

### Required

- `URL`
- `domain`

### Normalization rules

- if input starts with `http://` or `https://`, treat it as a URL
- otherwise treat it as a domain and normalize to `https://<domain>`
- default ports:
  - `443` for `https`
  - `80` for `http`

## 5. Diagnosis Layers

v1 must implement these five layers.

### 5.1 DNS

- resolve `A / AAAA / CNAME`
- return DNS latency
- surface suspected hijack / private-IP anomalies when applicable

### 5.2 TCP

- test target `IP:port` reachability
- return connection latency
- distinguish timeout / refused / reset

### 5.3 TLS / SSL

- perform TLS handshake for HTTPS targets
- validate certificate
- detect:
  - expired certificate
  - hostname mismatch
  - incomplete chain
  - self-signed certificate
  - outdated TLS version
  - handshake failure

### 5.4 HTTP / HTTPS

- execute request to normalized URL
- capture:
  - status code
  - redirect chain
  - response headers
  - latency
  - timeout / empty-body / downgrade anomalies

### 5.5 System / Proxy

- detect Windows system proxy / PAC / env proxy
- inspect hosts override
- inspect clock skew when possible

## 6. Output Principles

The output is not raw logs alone. It must be structured in three layers:

1. `summary`
2. `diagnostic modules`
3. `recommended actions`

### 6.1 Summary

Must include:

- overall status
- overall severity
- total duration
- resolved IP
- failure stage

### 6.2 Diagnostic Modules

The result model must map one-to-one to these top-level blocks:

- `summary`
- `dns`
- `tcp`
- `tls`
- `http`
- `system`
- `recommended_actions`

This is the canonical shape for BE, FE, and SQE.

### 6.3 Recommended Actions

Recommended actions are guidance, not automatic repair.

They are split into:

- `manual_actions[]`
- `quick_actions[]`

`quick_actions[]` represent Windows actions such as:

- open proxy settings
- open network settings
- open date/time settings
- open certificate manager

## 7. UI Information Architecture

The app is a single-screen workflow.

### 7.1 Main layout

1. `Header`
2. `ProxyAlert` (conditional)
3. `InputBar`
4. `Summary + DNS Records` left column
5. `Trace / Phase details` right column
6. `RecommendedActions`

### 7.2 UI rules

- no default dark theme
- no history sidebar
- no export button
- one `Copy JSON` action only
- each new diagnosis overwrites the previous result on screen

## 8. Status Model

### 8.1 Phase status

Allowed values:

- `pass`
- `warn`
- `fail`
- `skip`

### 8.2 Severity

Allowed values:

- `info`
- `warn`
- `fail`

Guideline:

- informational success or expected redirect -> `info`
- degraded but still usable -> `warn`
- diagnosis conclusively failed or serious risk found -> `fail`

## 9. Acceptance Scope for v1

v1 is considered usable when:

1. support staff can input a URL/domain and run diagnosis
2. the app returns structured results for all five layers
3. proxy warning is shown only when detected
4. TLS certificate issues are explicitly surfaced
5. recommended actions are shown as both text and quick actions
6. JSON result can be copied successfully
7. app can be built in CI for `windows / macos / linux`

## 10. Immediate Follow-Up

This document directly unblocks:

- BE `#17`: align Rust result model and diagnostic engine output
- FE `#16`: align UI data binding to the canonical result shape
- SQE `#18`: align matrix field names and acceptance checks
- BE `#15`: finish CI aggregation after FE make targets are merged
