pub mod dns;
pub mod http;
pub mod result;
pub mod system;
pub mod tcp;
pub mod tls;

use result::*;

/// Run all diagnostic phases for a given target.
pub async fn run_diagnostics(input: &str) -> DiagnosticReport {
    let start = std::time::Instant::now();

    let target = parse_target(input);

    // Phase 1: DNS
    let dns_result = dns::diagnose(&target.domain).await;

    let resolved_ip = dns_result.details.resolved_ip.clone();

    // Phase 2: TCP
    let tcp_result = if let Some(ref ip) = resolved_ip {
        tcp::diagnose(ip, target.port).await
    } else {
        TcpModule {
            status: Status::Fail,
            severity: Severity::Fail,
            duration_ms: 0,
            error: Some("No resolved IP from DNS phase".to_string()),
            details: TcpDetails {
                connected: false,
                ip: None,
                port: target.port,
            },
        }
    };

    // Phase 3: TLS
    let tls_result = if target.port == 443 && tcp_result.status != Status::Fail {
        tls::diagnose(
            &target.domain,
            resolved_ip.as_deref().unwrap_or(""),
            target.port,
        )
        .await
    } else if target.port != 443 {
        TlsModule {
            status: Status::Skip,
            severity: Severity::Info,
            duration_ms: 0,
            error: Some("Skipped: non-HTTPS port".to_string()),
            details: TlsDetails {
                handshake: false,
                version: None,
                cert: CertInfo::empty(),
            },
        }
    } else {
        TlsModule {
            status: Status::Fail,
            severity: Severity::Fail,
            duration_ms: 0,
            error: Some("Skipped: TCP connection failed".to_string()),
            details: TlsDetails {
                handshake: false,
                version: None,
                cert: CertInfo::empty(),
            },
        }
    };

    // Phase 4: HTTP
    let http_result = if tcp_result.status != Status::Fail {
        http::diagnose(&target.normalized_url).await
    } else {
        HttpModule {
            status: Status::Fail,
            severity: Severity::Fail,
            duration_ms: 0,
            error: Some("Skipped: TCP connection failed".to_string()),
            details: HttpDetails {
                status_code: None,
                redirect_chain: vec![],
                headers: std::collections::HashMap::new(),
                empty_body: false,
                downgraded: false,
            },
        }
    };

    // Phase 5: System/Proxy
    let system_result = system::diagnose();

    // Determine overall status
    let modules: Vec<(&str, &Status)> = vec![
        ("dns", &dns_result.status),
        ("tcp", &tcp_result.status),
        ("tls", &tls_result.status),
        ("http", &http_result.status),
        ("system", &system_result.status),
    ];

    let failure_stage = modules
        .iter()
        .find(|(_, s)| **s == Status::Fail)
        .map(|(name, _)| name.to_string());

    let overall_status = if failure_stage.is_some() {
        Status::Fail
    } else if modules.iter().any(|(_, s)| **s == Status::Warn) {
        Status::Warn
    } else {
        Status::Pass
    };

    let overall_severity = match overall_status {
        Status::Fail => Severity::Fail,
        Status::Warn => Severity::Warn,
        _ => Severity::Info,
    };

    let recommended_actions = generate_actions(
        &dns_result,
        &tcp_result,
        &tls_result,
        &http_result,
        &system_result,
    );

    let total_duration_ms = start.elapsed().as_millis() as u64;

    DiagnosticReport {
        version: "v0".to_string(),
        generated_at: iso8601_now(),
        target,
        summary: Summary {
            status: overall_status,
            severity: overall_severity,
            total_duration_ms,
            failure_stage,
            resolved_ip,
        },
        dns: dns_result,
        tcp: tcp_result,
        tls: tls_result,
        http: http_result,
        system: system_result,
        recommended_actions,
    }
}

fn parse_target(input: &str) -> Target {
    if input.starts_with("http://") || input.starts_with("https://") {
        if let Ok(url) = url::Url::parse(input) {
            let domain = url.host_str().unwrap_or("").to_string();
            let port = url.port_or_known_default().unwrap_or(443);
            return Target {
                input: input.to_string(),
                kind: "url".to_string(),
                normalized_url: input.to_string(),
                domain,
                port,
            };
        }
    }

    let domain = input.trim().to_string();
    let port = 443u16;
    Target {
        input: input.to_string(),
        kind: "domain".to_string(),
        normalized_url: format!("https://{}", domain),
        domain,
        port,
    }
}

fn generate_actions(
    dns: &DnsModule,
    tcp: &TcpModule,
    tls: &TlsModule,
    http: &HttpModule,
    system: &SystemModule,
) -> RecommendedActions {
    let mut manual = Vec::new();
    let mut quick = Vec::new();

    if dns.status == Status::Fail {
        manual.push("检查域名是否正确，或尝试更换 DNS 服务器（如 8.8.8.8）".to_string());
    }

    if tcp.status == Status::Fail {
        manual.push("目标端口不可达，检查防火墙或网络连接".to_string());
        quick.push(QuickAction {
            id: "open_network_status".to_string(),
            label: "打开网络状态".to_string(),
            kind: "open_uri".to_string(),
            target: "ms-settings:network-status".to_string(),
        });
    }

    if tls.status == Status::Fail || tls.status == Status::Warn {
        let cert = &tls.details.cert;
        if cert.expired {
            manual.push("SSL 证书已过期，联系网站管理员更新证书".to_string());
        }
        if cert.domain_mismatch {
            manual.push("SSL 证书域名不匹配，可能存在配置错误或中间人攻击".to_string());
        }
        if cert.self_signed {
            manual.push("检测到自签名证书，可能存在企业代理拦截 HTTPS 流量".to_string());
        }
        if cert.expiring_soon {
            manual.push(format!(
                "SSL 证书即将过期（剩余 {} 天），建议尽快更新",
                cert.days_remaining.unwrap_or(0)
            ));
        }
        if !cert.expired && !cert.domain_mismatch && tls.status == Status::Fail {
            manual.push("TLS 握手失败，检查是否有企业代理拦截 HTTPS 流量".to_string());
            quick.push(QuickAction {
                id: "open_proxy_settings".to_string(),
                label: "打开代理设置".to_string(),
                kind: "open_uri".to_string(),
                target: "ms-settings:network-proxy".to_string(),
            });
        }
    }

    if http.status == Status::Fail {
        manual.push("HTTP 请求失败，服务端可能存在问题，请稍后重试".to_string());
    }
    if http.status == Status::Warn {
        if let Some(code) = http.details.status_code {
            manual.push(format!("HTTP 返回状态码 {}，请检查请求是否正确", code));
        }
    }

    if system.details.proxy.enabled {
        manual.push("检测到系统代理，建议关闭代理后重试".to_string());
        quick.push(QuickAction {
            id: "open_proxy_settings".to_string(),
            label: "打开代理设置".to_string(),
            kind: "open_uri".to_string(),
            target: "ms-settings:network-proxy".to_string(),
        });
    }

    if system.details.hosts_override {
        manual.push("检测到 hosts 文件有自定义条目，可能影响域名解析".to_string());
    }

    // Deduplicate quick actions by id
    quick.dedup_by(|a, b| a.id == b.id);

    RecommendedActions {
        manual_actions: manual,
        quick_actions: quick,
    }
}

fn iso8601_now() -> String {
    let duration = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = duration.as_secs();
    // Convert to rough ISO 8601 (good enough for v0, can use chrono later)
    let days_since_epoch = secs / 86400;
    let time_of_day = secs % 86400;
    let hours = time_of_day / 3600;
    let minutes = (time_of_day % 3600) / 60;
    let seconds = time_of_day % 60;

    // Simple date calculation from epoch
    let mut year = 1970i64;
    let mut remaining_days = days_since_epoch as i64;
    loop {
        let days_in_year = if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) {
            366
        } else {
            365
        };
        if remaining_days < days_in_year {
            break;
        }
        remaining_days -= days_in_year;
        year += 1;
    }
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let month_days = [
        31,
        if leap { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    let mut month = 1u32;
    for &md in &month_days {
        if remaining_days < md {
            break;
        }
        remaining_days -= md;
        month += 1;
    }
    let day = remaining_days + 1;

    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        year, month, day, hours, minutes, seconds
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_target_url() {
        let t = parse_target("https://example.com/api/health");
        assert_eq!(t.domain, "example.com");
        assert_eq!(t.port, 443);
        assert_eq!(t.kind, "url");
        assert_eq!(t.normalized_url, "https://example.com/api/health");
    }

    #[test]
    fn test_parse_target_domain() {
        let t = parse_target("example.com");
        assert_eq!(t.domain, "example.com");
        assert_eq!(t.port, 443);
        assert_eq!(t.kind, "domain");
        assert_eq!(t.normalized_url, "https://example.com");
    }

    #[test]
    fn test_parse_target_http() {
        let t = parse_target("http://example.com");
        assert_eq!(t.domain, "example.com");
        assert_eq!(t.port, 80);
        assert_eq!(t.kind, "url");
    }

    #[test]
    fn test_iso8601_now() {
        let ts = iso8601_now();
        assert!(ts.ends_with('Z'));
        assert!(ts.contains('T'));
        assert_eq!(ts.len(), 20); // "2026-04-03T00:00:00Z"
    }
}
