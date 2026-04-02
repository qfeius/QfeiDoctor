use super::result::{HttpDetails, HttpModule, Severity, Status};
use std::collections::HashMap;

/// Diagnose HTTP(S) request to a URL.
pub async fn diagnose(url: &str) -> HttpModule {
    let start = std::time::Instant::now();

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .redirect(reqwest::redirect::Policy::limited(10))
        .danger_accept_invalid_certs(false)
        .build();

    let client = match client {
        Ok(c) => c,
        Err(e) => {
            return HttpModule {
                status: Status::Fail,
                severity: Severity::Fail,
                duration_ms: start.elapsed().as_millis() as u64,
                error: Some(format!("Failed to create HTTP client: {}", e)),
                details: HttpDetails {
                    status_code: None,
                    redirect_chain: vec![],
                    headers: HashMap::new(),
                    empty_body: false,
                    downgraded: false,
                },
            };
        }
    };

    match client.get(url).send().await {
        Ok(response) => {
            let status_code = response.status().as_u16();
            let final_url = response.url().to_string();

            let headers: HashMap<String, String> = response
                .headers()
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
                .collect();

            // Detect HTTPS → HTTP downgrade
            let downgraded = url.starts_with("https://") && final_url.starts_with("http://");

            // Build redirect chain if final URL differs from input
            let redirect_chain = if final_url != url {
                vec![final_url]
            } else {
                vec![]
            };

            // Check for empty body
            let content_length = headers
                .get("content-length")
                .and_then(|v| v.parse::<u64>().ok());
            let empty_body = content_length == Some(0);

            let phase_status = if (200..400).contains(&status_code) {
                Status::Pass
            } else if (400..500).contains(&status_code) {
                Status::Warn
            } else {
                Status::Fail
            };

            let severity = match phase_status {
                Status::Fail => Severity::Fail,
                Status::Warn => Severity::Warn,
                _ => Severity::Info,
            };

            HttpModule {
                status: phase_status,
                severity,
                duration_ms: start.elapsed().as_millis() as u64,
                error: None,
                details: HttpDetails {
                    status_code: Some(status_code),
                    redirect_chain,
                    headers,
                    empty_body,
                    downgraded,
                },
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

            HttpModule {
                status: Status::Fail,
                severity: Severity::Fail,
                duration_ms: start.elapsed().as_millis() as u64,
                error: Some(error_msg),
                details: HttpDetails {
                    status_code: None,
                    redirect_chain: vec![],
                    headers: HashMap::new(),
                    empty_body: false,
                    downgraded: false,
                },
            }
        }
    }
}
