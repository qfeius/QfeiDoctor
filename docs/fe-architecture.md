# QfeiDoctor — Frontend Architecture Plan

## Page Structure (Single Page App)

Based on the prototype and PE's guidance, the UI is a single-page layout with these zones:

```
+------------------------------------------------------------------+
| Header: Logo + App Name (智书诊断助手 QfeiDoctor)                   |
+------------------------------------------------------------------+
| ProxyAlert (conditional — only shown when proxy detected)         |
| [warning icon] 检测到系统代理已开启...    [打开代理设置] [x]          |
+------------------------------------------------------------------+
| TARGET ADDRESS (URL / DOMAIN)                                     |
| [___input___________________________] [开始分析] [复制反馈信息]      |
+------------------------------------------------------------------+
| LEFT PANEL (30%)          | RIGHT PANEL (70%)                     |
|                           |                                       |
| SummaryCard               | DiagnosticTrace                       |
|  - Overall Status         |  - Phase 1: DNS Resolution            |
|  - Total Duration         |  - Phase 2: TCP Connectivity          |
|  - Resolved IP            |  - Phase 3: TLS/SSL Negotiation       |
|  - Failure Stage          |  - Phase 4: HTTP Request              |
|                           |  - Phase 5: System/Proxy              |
| DnsRecords                |                                       |
|  - A records              |                                       |
|  - AAAA records           |                                       |
|  - CNAME records          |                                       |
+---------------------------+---------------------------------------+
| RecommendedActions                                                |
|  - Probable cause text                                            |
|  - [打开代理设置] [打开DNS设置] [打开证书管理器]                      |
+------------------------------------------------------------------+
```

## Component Tree

```
App
├── Header
├── ProxyAlert (conditional)
├── InputBar
│   ├── TargetInput
│   ├── StartButton
│   └── CopyJsonButton
├── ResultLayout (two-column)
│   ├── LeftPanel
│   │   ├── SummaryCard
│   │   │   ├── StatusBadge (pass/warn/fail)
│   │   │   ├── DurationDisplay
│   │   │   ├── ResolvedIpDisplay
│   │   │   └── FailureStageDisplay
│   │   └── DnsRecordsCard
│   │       └── DnsRecordRow (repeated)
│   └── RightPanel
│       └── DiagnosticTrace
│           ├── PhaseSection (dns)
│           ├── PhaseSection (tcp)
│           ├── PhaseSection (tls)
│           ├── PhaseSection (http)
│           └── PhaseSection (system)
└── RecommendedActions
    ├── CauseDescription
    └── QuickActionButton (repeated)
```

## File Structure

```
src/
├── App.tsx                     # Root layout
├── main.tsx                    # Entry point
├── types/
│   └── diagnostic.ts           # TypeScript interfaces mirroring JSON schema
├── components/
│   ├── Header.tsx
│   ├── ProxyAlert.tsx
│   ├── InputBar.tsx
│   ├── SummaryCard.tsx
│   ├── DnsRecordsCard.tsx
│   ├── DiagnosticTrace.tsx
│   ├── PhaseSection.tsx
│   ├── RecommendedActions.tsx
│   ├── QuickActionButton.tsx
│   ├── StatusBadge.tsx
│   └── CopyJsonButton.tsx
├── hooks/
│   └── useDiagnostic.ts        # Tauri invoke wrapper + state
├── styles/
│   └── index.css               # Global styles (neutral theme)
└── lib/
    └── tauri.ts                # Tauri command bindings
```

## TypeScript Interfaces (mirrors JSON schema 1:1)

```typescript
// Matches diagnostic result JSON — each section = one UI zone
interface DiagnosticResult {
  target: string;
  timestamp: string;
  summary: Summary;
  dns: DnsResult;
  tcp: TcpResult;
  tls: TlsResult;
  http: HttpResult;
  system: SystemResult;
  recommended_actions: RecommendedAction[];
}

interface Summary {
  overall_status: 'pass' | 'warn' | 'fail';
  total_duration_ms: number;
  resolved_ip: string;
  failure_stage: string | null;
}

interface DnsResult {
  status: 'pass' | 'warn' | 'fail';
  duration_ms: number;
  records: DnsRecord[];
  details: Record<string, any>;
  suggestions: string[];
}

interface DnsRecord {
  type: 'A' | 'AAAA' | 'CNAME';
  value: string;
  ttl: number;
}

interface TcpResult {
  status: 'pass' | 'warn' | 'fail';
  duration_ms: number;
  details: Record<string, any>;
  suggestions: string[];
}

interface TlsResult {
  status: 'pass' | 'warn' | 'fail';
  duration_ms: number;
  certificate: CertificateInfo | null;
  details: Record<string, any>;
  suggestions: string[];
}

interface CertificateInfo {
  issuer: string;
  subject: string;
  not_before: string;
  not_after: string;
  expired: boolean;
  hostname_matched: boolean;
  chain_complete: boolean;
  tls_version: string;
  self_signed: boolean;
}

interface HttpResult {
  status: 'pass' | 'warn' | 'fail';
  duration_ms: number;
  status_code: number | null;
  redirect_chain: string[];
  details: Record<string, any>;
  suggestions: string[];
}

interface SystemResult {
  status: 'pass' | 'warn' | 'fail';
  proxy_detected: boolean;
  proxy_type: string | null;
  proxy_address: string | null;
  hosts_modified: boolean;
  details: Record<string, any>;
  suggestions: string[];
}

interface RecommendedAction {
  type: 'suggestion' | 'quick_action';
  description: string;
  action?: string;  // e.g. 'open_proxy_settings', 'open_cert_manager'
}
```

## State Management

Simple — single page, one diagnostic flow. No need for Redux/Zustand.

```
useState in useDiagnostic hook:
  - target: string (user input)
  - isRunning: boolean
  - result: DiagnosticResult | null
  - error: string | null
```

The `useDiagnostic` hook wraps `invoke('run_diagnostic', { target })` from Tauri.

## Data Flow

```
User types URL/domain → InputBar
  ↓
Click "开始分析" → useDiagnostic.start(target)
  ↓
invoke('run_diagnostic', { target }) → Rust engine
  ↓
Rust returns DiagnosticResult JSON
  ↓
React renders:
  - SummaryCard ← result.summary
  - DnsRecordsCard ← result.dns.records
  - DiagnosticTrace ← result.dns/tcp/tls/http/system
  - RecommendedActions ← result.recommended_actions
  - ProxyAlert ← result.system.proxy_detected

Click "复制反馈信息" → JSON.stringify(result) → clipboard
```

## Quick Actions (Tauri shell.open)

```typescript
const QUICK_ACTIONS: Record<string, string> = {
  open_proxy_settings: 'ms-settings:network-proxy',
  open_network_settings: 'ms-settings:network-ethernet',
  open_cert_manager: 'certmgr.msc',
  open_dns_settings: 'ncpa.cpl',
};
```

Invoked via `import { open } from '@tauri-apps/plugin-shell'`.

## Tech Stack Summary

- React 18 + TypeScript
- Vite (bundled with Tauri)
- No CSS framework initially (neutral, clean skeleton per PE)
- No router needed (single page)
- No state library needed (useState sufficient)
