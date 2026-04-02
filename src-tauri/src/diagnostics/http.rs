use super::result::{PhaseDiagnostic, PhaseStatus};
use serde_json::json;

/// Diagnose HTTP(S) request to a URL.
pub async fn diagnose(url: &str) -> PhaseDiagnostic {
    let start = std::time::Instant::now();

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .redirect(reqwest::redirect::Policy::none())
        .danger_accept_invalid_certs(false)
        .build();

    let client = match client {
        Ok(c) => c,
        Err(e) => {
            return PhaseDiagnostic {
                name: "http".to_string(),
                status: PhaseStatus::Fail,
                duration_ms: start.elapsed().as_millis() as u64,
                details: None,
                error: Some(format!("Failed to create HTTP client: {}", e)),
            };
        }
    };

    match client.get(url).send().await {
        Ok(response) => {
            let status_code = response.status().as_u16();
            let headers: std::collections::HashMap<String, String> = response
                .headers()
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
                .collect();

            let phase_status = if (200..400).contains(&status_code) {
                PhaseStatus::Pass
            } else if (400..500).contains(&status_code) {
                PhaseStatus::Warn
            } else {
                PhaseStatus::Fail
            };

            PhaseDiagnostic {
                name: "http".to_string(),
                status: phase_status,
                duration_ms: start.elapsed().as_millis() as u64,
                details: Some(json!({
                    "status_code": status_code,
                    "headers": headers,
                })),
                error: None,
            }
        }
        Err(e) => {
            let error_msg = if e.is_timeout() {
                "HTTP request timed out".to_string()
            } else if e.is_connect() {
                format!("HTTP connection error: {}", e)
            } else {
                format!("HTTP request failed: {}", e)
            };

            PhaseDiagnostic {
                name: "http".to_string(),
                status: PhaseStatus::Fail,
                duration_ms: start.elapsed().as_millis() as u64,
                details: None,
                error: Some(error_msg),
            }
        }
    }
}
