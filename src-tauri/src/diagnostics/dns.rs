use super::result::{PhaseDiagnostic, PhaseStatus};
use hickory_resolver::Resolver;
use serde_json::json;

/// Diagnose DNS resolution for a domain.
pub async fn diagnose(domain: &str) -> PhaseDiagnostic {
    let start = std::time::Instant::now();

    let resolver = match Resolver::builder_tokio() {
        Ok(builder) => builder.build(),
        Err(e) => {
            return PhaseDiagnostic {
                name: "dns".to_string(),
                status: PhaseStatus::Fail,
                duration_ms: start.elapsed().as_millis() as u64,
                details: None,
                error: Some(format!("Failed to create DNS resolver: {}", e)),
            };
        }
    };

    // Resolve A records
    let a_records: Vec<String> = match resolver.lookup_ip(domain).await {
        Ok(response) => response.iter().map(|ip| ip.to_string()).collect(),
        Err(e) => {
            return PhaseDiagnostic {
                name: "dns".to_string(),
                status: PhaseStatus::Fail,
                duration_ms: start.elapsed().as_millis() as u64,
                details: None,
                error: Some(format!("DNS resolution failed: {}", e)),
            };
        }
    };

    if a_records.is_empty() {
        return PhaseDiagnostic {
            name: "dns".to_string(),
            status: PhaseStatus::Fail,
            duration_ms: start.elapsed().as_millis() as u64,
            details: None,
            error: Some("No DNS records found".to_string()),
        };
    }

    let resolved_ip = a_records.first().cloned().unwrap_or_default();

    PhaseDiagnostic {
        name: "dns".to_string(),
        status: PhaseStatus::Pass,
        duration_ms: start.elapsed().as_millis() as u64,
        details: Some(json!({
            "resolved_ip": resolved_ip,
            "records": a_records,
        })),
        error: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // requires network
    async fn test_dns_resolve_known_domain() {
        let result = diagnose("example.com").await;
        assert_eq!(result.name, "dns");
        assert_eq!(result.status, PhaseStatus::Pass);
        assert!(result.details.is_some());
    }

    #[tokio::test]
    #[ignore] // requires network
    async fn test_dns_resolve_invalid_domain() {
        let result = diagnose("thisdomaindoesnotexist12345.invalid").await;
        assert_eq!(result.name, "dns");
    }
}
