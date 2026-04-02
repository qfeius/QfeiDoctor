pub mod dns;
pub mod http;
pub mod result;
pub mod system;
pub mod tcp;
pub mod tls;

use result::{DiagnosticReport, PhaseDiagnostic, PhaseStatus, Suggestion};

/// Run all diagnostic phases for a given target.
pub async fn run_diagnostics(target: &str) -> DiagnosticReport {
    let start = std::time::Instant::now();

    // Parse target: URL or domain
    let (domain, port, url) = parse_target(target);

    // Phase 1: DNS
    let dns_result = dns::diagnose(&domain).await;

    // Determine resolved IP for subsequent phases
    let resolved_ip = dns_result
        .details
        .as_ref()
        .and_then(|d| d.get("resolved_ip"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    // Phase 2: TCP
    let tcp_result = if !resolved_ip.is_empty() {
        tcp::diagnose(&resolved_ip, port).await
    } else {
        PhaseDiagnostic {
            name: "tcp".to_string(),
            status: PhaseStatus::Fail,
            duration_ms: 0,
            details: None,
            error: Some("No resolved IP from DNS phase".to_string()),
        }
    };

    // Phase 3: TLS
    let tls_result = if port == 443 && tcp_result.status != PhaseStatus::Fail {
        tls::diagnose(&domain, &resolved_ip, port).await
    } else if port != 443 {
        PhaseDiagnostic {
            name: "tls".to_string(),
            status: PhaseStatus::Skip,
            duration_ms: 0,
            details: None,
            error: Some("Skipped: non-HTTPS port".to_string()),
        }
    } else {
        PhaseDiagnostic {
            name: "tls".to_string(),
            status: PhaseStatus::Fail,
            duration_ms: 0,
            details: None,
            error: Some("Skipped: TCP connection failed".to_string()),
        }
    };

    // Phase 4: HTTP
    let http_result = if tcp_result.status != PhaseStatus::Fail {
        http::diagnose(&url).await
    } else {
        PhaseDiagnostic {
            name: "http".to_string(),
            status: PhaseStatus::Fail,
            duration_ms: 0,
            details: None,
            error: Some("Skipped: TCP connection failed".to_string()),
        }
    };

    // Phase 5: System/Proxy
    let system_result = system::diagnose();

    // Collect all phases
    let phases = vec![
        dns_result,
        tcp_result,
        tls_result,
        http_result,
        system_result,
    ];

    // Determine overall status and failure stage
    let failure_stage = phases
        .iter()
        .find(|p| p.status == PhaseStatus::Fail)
        .map(|p| p.name.clone());

    let overall_status = if failure_stage.is_some() {
        PhaseStatus::Fail
    } else if phases.iter().any(|p| p.status == PhaseStatus::Warn) {
        PhaseStatus::Warn
    } else {
        PhaseStatus::Pass
    };

    // Generate suggestions based on results
    let suggestions = generate_suggestions(&phases);

    let total_duration_ms = start.elapsed().as_millis() as u64;

    DiagnosticReport {
        target: target.to_string(),
        timestamp: chrono_now(),
        overall_status,
        total_duration_ms,
        resolved_ip: if resolved_ip.is_empty() {
            None
        } else {
            Some(resolved_ip)
        },
        failure_stage,
        phases,
        suggestions,
    }
}

/// Parse a target string into (domain, port, full_url).
fn parse_target(target: &str) -> (String, u16, String) {
    if target.starts_with("http://") || target.starts_with("https://") {
        // It's a URL
        if let Ok(url) = url::Url::parse(target) {
            let domain = url.host_str().unwrap_or("").to_string();
            let port = url.port_or_known_default().unwrap_or(443);
            return (domain, port, target.to_string());
        }
    }

    // Treat as domain
    let domain = target.trim().to_string();
    let port = 443u16;
    let url = format!("https://{}", domain);
    (domain, port, url)
}

fn generate_suggestions(phases: &[PhaseDiagnostic]) -> Vec<Suggestion> {
    let mut suggestions = Vec::new();

    for phase in phases {
        if phase.status != PhaseStatus::Fail {
            continue;
        }
        match phase.name.as_str() {
            "dns" => {
                suggestions.push(Suggestion {
                    cause: "DNS 解析失败".to_string(),
                    action: "检查域名是否正确，或尝试更换 DNS 服务器（如 8.8.8.8）".to_string(),
                    quick_action: None,
                });
            }
            "tcp" => {
                suggestions.push(Suggestion {
                    cause: "TCP 连接失败".to_string(),
                    action: "目标端口不可达，检查防火墙或网络连接".to_string(),
                    quick_action: Some("ms-settings:network-status".to_string()),
                });
            }
            "tls" => {
                if let Some(details) = &phase.details {
                    if details.get("expired").and_then(|v| v.as_bool()) == Some(true) {
                        suggestions.push(Suggestion {
                            cause: "SSL 证书已过期".to_string(),
                            action: "联系网站管理员更新 SSL 证书".to_string(),
                            quick_action: None,
                        });
                    }
                    if details.get("hostname_matched").and_then(|v| v.as_bool()) == Some(false) {
                        suggestions.push(Suggestion {
                            cause: "SSL 证书域名不匹配".to_string(),
                            action: "证书不是为此域名签发的，可能存在配置错误或中间人攻击"
                                .to_string(),
                            quick_action: None,
                        });
                    }
                }
                suggestions.push(Suggestion {
                    cause: "TLS 握手失败".to_string(),
                    action: "检查是否有企业代理拦截 HTTPS 流量".to_string(),
                    quick_action: Some("ms-settings:network-proxy".to_string()),
                });
            }
            "http" => {
                suggestions.push(Suggestion {
                    cause: "HTTP 请求失败".to_string(),
                    action: "服务端可能存在问题，请稍后重试或联系网站管理员".to_string(),
                    quick_action: None,
                });
            }
            "system" => {
                if let Some(details) = &phase.details {
                    if details.get("proxy_enabled").and_then(|v| v.as_bool()) == Some(true) {
                        suggestions.push(Suggestion {
                            cause: "检测到系统代理".to_string(),
                            action: "系统代理可能影响网络访问，建议关闭代理后重试".to_string(),
                            quick_action: Some("ms-settings:network-proxy".to_string()),
                        });
                    }
                }
            }
            _ => {}
        }
    }

    suggestions
}

fn chrono_now() -> String {
    // Simple ISO 8601 timestamp without chrono dependency
    let duration = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = duration.as_secs();
    // Return Unix timestamp as string; proper formatting can be added later
    format!("{}", secs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_target_url() {
        let (domain, port, url) = parse_target("https://example.com/api/health");
        assert_eq!(domain, "example.com");
        assert_eq!(port, 443);
        assert_eq!(url, "https://example.com/api/health");
    }

    #[test]
    fn test_parse_target_domain() {
        let (domain, port, url) = parse_target("example.com");
        assert_eq!(domain, "example.com");
        assert_eq!(port, 443);
        assert_eq!(url, "https://example.com");
    }

    #[test]
    fn test_parse_target_http() {
        let (domain, port, _url) = parse_target("http://example.com");
        assert_eq!(domain, "example.com");
        assert_eq!(port, 80);
    }
}
