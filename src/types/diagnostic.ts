/**
 * TypeScript types matching docs/diagnostic-result.schema.json v0.
 * This is the single source of truth for the diagnostic result model.
 */

export type Status = "pass" | "warn" | "fail" | "skip";
export type Severity = "info" | "warn" | "fail";
export type FailureStage = "dns" | "tcp" | "tls" | "http" | "system" | null;

export interface DiagnosticResult {
  version: "v0";
  generated_at: string;
  target: Target;
  summary: Summary;
  dns: DnsModule;
  tcp: TcpModule;
  tls: TlsModule;
  http: HttpModule;
  system: SystemModule;
  recommended_actions: RecommendedActions;
}

export interface Target {
  input: string;
  kind: "url" | "domain";
  normalized_url: string;
  domain: string;
  port: number;
}

export interface Summary {
  status: Status;
  severity: Severity;
  total_duration_ms: number;
  failure_stage: FailureStage;
  resolved_ip: string | null;
}

// Base module shape shared by all diagnostic phases
interface ModuleBase {
  status: Status;
  severity: Severity;
  duration_ms: number;
  error: string | null;
}

// DNS
export interface DnsModule extends ModuleBase {
  details: DnsDetails;
}

export interface DnsDetails {
  records: DnsRecord[];
  resolved: boolean;
  resolved_ip: string | null;
  suspected_hijack: boolean;
  private_ip: boolean;
}

export interface DnsRecord {
  type: "A" | "AAAA" | "CNAME";
  value: string;
  ttl: number;
}

// TCP
export interface TcpModule extends ModuleBase {
  details: TcpDetails;
}

export interface TcpDetails {
  connected: boolean;
  ip: string | null;
  port: number;
}

// TLS
export interface TlsModule extends ModuleBase {
  details: TlsDetails;
}

export interface TlsDetails {
  handshake: boolean;
  version: string | null;
  cert: CertInfo;
}

export interface CertInfo {
  valid: boolean;
  expired: boolean;
  expiring_soon: boolean;
  days_remaining: number | null;
  domain_mismatch: boolean;
  chain_incomplete: boolean;
  self_signed: boolean;
  issuer: string | null;
  subject: string | null;
  not_before: string | null;
  not_after: string | null;
}

// HTTP
export interface HttpModule extends ModuleBase {
  details: HttpDetails;
}

export interface HttpDetails {
  status_code: number | null;
  redirect_chain: string[];
  headers: Record<string, string>;
  empty_body: boolean;
  downgraded: boolean;
}

// System
export interface SystemModule extends ModuleBase {
  details: SystemDetails;
}

export interface SystemDetails {
  proxy: ProxyInfo;
  clock_skewed: boolean;
  clock_offset_sec: number | null;
  hosts_override: boolean;
}

export interface ProxyInfo {
  enabled: boolean;
  type: "system" | "pac" | "env" | null;
  address: string | null;
  pac_url: string | null;
  env_var: string | null;
}

// Recommended Actions
export interface RecommendedActions {
  manual_actions: string[];
  quick_actions: QuickAction[];
}

export interface QuickAction {
  id: string;
  label: string;
  kind: "open_uri";
  target: string;
}
