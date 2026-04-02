use serde::{Deserialize, Serialize};

/// Overall diagnostic report — this is what gets copied as JSON.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticReport {
    pub target: String,
    pub timestamp: String,
    pub overall_status: PhaseStatus,
    pub total_duration_ms: u64,
    pub resolved_ip: Option<String>,
    pub failure_stage: Option<String>,
    pub phases: Vec<PhaseDiagnostic>,
    pub suggestions: Vec<Suggestion>,
}

/// Result of a single diagnostic phase.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseDiagnostic {
    pub name: String,
    pub status: PhaseStatus,
    pub duration_ms: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Status of a diagnostic phase.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PhaseStatus {
    Pass,
    Warn,
    Fail,
    Skip,
}

/// A suggested action based on diagnostic results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Suggestion {
    pub cause: String,
    pub action: String,
    /// Windows settings URI for quick action button (e.g. "ms-settings:network-proxy")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quick_action: Option<String>,
}
