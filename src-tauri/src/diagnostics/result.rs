use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Top-level diagnostic report — matches docs/diagnostic-result.schema.json v0.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticReport {
    pub version: String,
    pub generated_at: String,
    pub target: Target,
    pub summary: Summary,
    pub dns: DnsModule,
    pub tcp: TcpModule,
    pub tls: TlsModule,
    pub http: HttpModule,
    pub system: SystemModule,
    pub recommended_actions: RecommendedActions,
    pub ipinfo: Option<IpInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Target {
    pub input: String,
    pub kind: String,
    pub normalized_url: String,
    pub domain: String,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Summary {
    pub status: Status,
    pub severity: Severity,
    pub total_duration_ms: u64,
    pub failure_stage: Option<String>,
    pub resolved_ip: Option<String>,
}

// --- Module base fields ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsModule {
    pub status: Status,
    pub severity: Severity,
    pub duration_ms: u64,
    pub error: Option<String>,
    pub details: DnsDetails,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsDetails {
    pub records: Vec<DnsRecord>,
    pub resolved: bool,
    pub resolved_ip: Option<String>,
    pub suspected_hijack: bool,
    pub private_ip: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsRecord {
    #[serde(rename = "type")]
    pub record_type: String,
    pub value: String,
    pub ttl: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TcpModule {
    pub status: Status,
    pub severity: Severity,
    pub duration_ms: u64,
    pub error: Option<String>,
    pub details: TcpDetails,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TcpDetails {
    pub connected: bool,
    pub ip: Option<String>,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsModule {
    pub status: Status,
    pub severity: Severity,
    pub duration_ms: u64,
    pub error: Option<String>,
    pub details: TlsDetails,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsDetails {
    pub handshake: bool,
    pub version: Option<String>,
    pub cert: CertInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertInfo {
    pub valid: bool,
    pub expired: bool,
    pub expiring_soon: bool,
    pub days_remaining: Option<i64>,
    pub domain_mismatch: bool,
    pub chain_incomplete: bool,
    pub self_signed: bool,
    pub issuer: Option<String>,
    pub subject: Option<String>,
    pub not_before: Option<String>,
    pub not_after: Option<String>,
}

impl CertInfo {
    pub fn empty() -> Self {
        Self {
            valid: false,
            expired: false,
            expiring_soon: false,
            days_remaining: None,
            domain_mismatch: false,
            chain_incomplete: false,
            self_signed: false,
            issuer: None,
            subject: None,
            not_before: None,
            not_after: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpModule {
    pub status: Status,
    pub severity: Severity,
    pub duration_ms: u64,
    pub error: Option<String>,
    pub details: HttpDetails,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpDetails {
    pub status_code: Option<u16>,
    pub redirect_chain: Vec<String>,
    pub headers: HashMap<String, String>,
    pub empty_body: bool,
    pub downgraded: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemModule {
    pub status: Status,
    pub severity: Severity,
    pub duration_ms: u64,
    pub error: Option<String>,
    pub details: SystemDetails,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemDetails {
    pub proxy: ProxyInfo,
    pub clock_skewed: bool,
    pub clock_offset_sec: Option<i64>,
    pub hosts_override: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyInfo {
    pub enabled: bool,
    #[serde(rename = "type")]
    pub proxy_type: Option<String>,
    pub address: Option<String>,
    pub pac_url: Option<String>,
    pub env_var: Option<String>,
    pub settings_uri: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendedActions {
    pub manual_actions: Vec<String>,
    pub quick_actions: Vec<QuickAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuickAction {
    pub id: String,
    pub label: String,
    pub kind: String,
    pub target: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpInfo {
    pub ip: String,
    pub city: String,
    pub region: String,
    pub country: String,
    pub loc: String,
    pub org: String,
    pub postal: String,
    pub timezone: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Pass,
    Warn,
    Fail,
    Skip,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Info,
    Warn,
    Fail,
}
