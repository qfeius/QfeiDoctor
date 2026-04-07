pub mod dns;
pub mod http;
pub mod ipinfo;
pub mod result;
pub mod system;
pub mod tcp;
pub mod tls;

use result::*;

/// Run all diagnostic phases for a given target.
pub async fn run_diagnostics(input: &str) -> DiagnosticReport {
    let start = std::time::Instant::now();

    let target = parse_target(input);

    // Fire ipinfo in parallel (independent of diagnostic phases)
    let ipinfo_handle = tokio::spawn(ipinfo::fetch());

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

    // Phase 5: System/Proxy (pass HTTP Date header for clock skew detection)
    let server_date = http_result.details.headers.get("date").map(|s| s.as_str());
    let system_result = system::diagnose(server_date);

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

    // Collect ipinfo result (best-effort)
    let ipinfo_result = ipinfo_handle.await.ok().flatten();

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
        ipinfo: ipinfo_result,
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

    // ── COMBO-01: Proxy + TLS cert replaced → corporate MITM ──
    if system.details.proxy.enabled && (tls.status == Status::Fail || tls.details.cert.self_signed)
    {
        manual.push(
            "检测到系统代理且 TLS 异常，极可能是企业代理拦截 HTTPS 流量（中间人代理）。\
             请关闭代理或联系 IT 部门将目标域名加入代理白名单"
                .to_string(),
        );
        if let Some(uri) = &system.details.proxy.settings_uri {
            quick.push(QuickAction {
                id: "open_proxy".to_string(),
                label: "打开代理设置".to_string(),
                kind: "open_uri".to_string(),
                target: uri.clone(),
            });
        }
    }

    // ── COMBO-02: DNS hijack + HTTP anomaly → DNS hijack/pollution ──
    if dns.details.suspected_hijack
        && (http.status == Status::Fail || http.details.downgraded || http.details.empty_body)
    {
        manual.push(
            "域名解析到内网地址且 HTTP 响应异常，高度怀疑 DNS 被劫持或 hosts 文件被篡改。\
             建议检查 hosts 文件并尝试更换 DNS 服务器（如 8.8.8.8 或 114.114.114.114）"
                .to_string(),
        );
        quick.push(QuickAction {
            id: "flush_dns".to_string(),
            label: "刷新 DNS 缓存".to_string(),
            kind: "open_uri".to_string(),
            target: "ipconfig /flushdns".to_string(),
        });
        quick.push(QuickAction {
            id: "open_hosts".to_string(),
            label: "打开 hosts 文件".to_string(),
            kind: "open_uri".to_string(),
            target: "notepad C:\\Windows\\System32\\drivers\\etc\\hosts".to_string(),
        });
    }

    // ── COMBO-03: TCP ok + HTTP 5xx → server-side issue, not client network ──
    if tcp.status == Status::Pass {
        if let Some(code) = http.details.status_code {
            if code >= 500 {
                manual.push(format!(
                    "TCP 连接正常但 HTTP 返回 {}，问题在站点服务端而非客户网络。建议联系站点运维",
                    code
                ));
            }
        }
    }

    // ── Clock skew + TLS failure → clock causing cert rejection ──
    if system.details.clock_skewed && tls.status == Status::Fail {
        manual.push(format!(
            "系统时钟偏差约 {} 秒，这会导致 TLS 证书验证失败。请校准系统时间后重试",
            system.details.clock_offset_sec.unwrap_or(0).abs()
        ));
        quick.push(QuickAction {
            id: "open_datetime".to_string(),
            label: "打开日期和时间设置".to_string(),
            kind: "open_uri".to_string(),
            target: "ms-settings:dateandtime".to_string(),
        });
    }

    // ── Per-module actions (only if not already covered by combos above) ──

    // DNS
    if dns.status == Status::Fail {
        manual.push(
            "DNS 解析失败，检查域名是否正确，或尝试更换 DNS 服务器（如 8.8.8.8）".to_string(),
        );
        quick.push(QuickAction {
            id: "flush_dns".to_string(),
            label: "刷新 DNS 缓存".to_string(),
            kind: "open_uri".to_string(),
            target: "ipconfig /flushdns".to_string(),
        });
    } else if dns.details.suspected_hijack && !has_action(&manual, "DNS 被劫持") {
        manual.push("域名解析到内网地址，可能存在 DNS 劫持。建议更换 DNS 服务器验证".to_string());
    }
    if dns.details.private_ip {
        quick.push(QuickAction {
            id: "open_hosts".to_string(),
            label: "打开 hosts 文件".to_string(),
            kind: "open_uri".to_string(),
            target: "notepad C:\\Windows\\System32\\drivers\\etc\\hosts".to_string(),
        });
    }

    // TCP
    if tcp.status == Status::Fail {
        manual.push("目标端口不可达，检查防火墙设置或确认网络连接正常".to_string());
        quick.push(QuickAction {
            id: "open_network".to_string(),
            label: "打开网络状态".to_string(),
            kind: "open_uri".to_string(),
            target: "ms-settings:network-status".to_string(),
        });
    }

    // TLS (individual issues not covered by combos)
    if tls.status == Status::Fail || tls.status == Status::Warn {
        let cert = &tls.details.cert;
        if cert.expired {
            manual.push("SSL 证书已过期，联系网站管理员更新证书".to_string());
        }
        if cert.domain_mismatch {
            manual.push("SSL 证书域名不匹配，可能存在配置错误或中间人攻击".to_string());
        }
        if cert.self_signed && !system.details.proxy.enabled {
            manual.push("检测到自签名证书，可能存在网络劫持".to_string());
        }
        if cert.expiring_soon {
            manual.push(format!(
                "SSL 证书即将过期（剩余 {} 天），建议联系管理员尽快更新",
                cert.days_remaining.unwrap_or(0)
            ));
        }
        if cert.chain_incomplete && !cert.self_signed {
            manual.push("SSL 证书链不完整，可能导致部分客户端无法验证证书".to_string());
        }
        if !cert.expired
            && !cert.domain_mismatch
            && !cert.self_signed
            && !system.details.clock_skewed
            && tls.status == Status::Fail
        {
            manual.push("TLS 握手失败，可能是网络设备干扰或协议版本不兼容".to_string());
        }
    }

    // HTTP (standalone, not already in combos)
    if http.status == Status::Fail && !dns.details.suspected_hijack {
        let is_5xx = http.details.status_code.is_some_and(|c| c >= 500);
        if !is_5xx {
            // 5xx already covered by COMBO-03
            manual.push("HTTP 请求失败，服务端可能存在问题，请稍后重试".to_string());
        }
    }
    if http.status == Status::Warn {
        if let Some(code) = http.details.status_code {
            manual.push(format!("HTTP 返回状态码 {}，请检查 URL 是否正确", code));
        }
    }
    if http.details.downgraded && !dns.details.suspected_hijack {
        manual.push("检测到 HTTPS 被降级为 HTTP，数据可能不安全".to_string());
    }
    if http.details.empty_body && http.status == Status::Pass {
        manual.push("HTTP 响应内容为空，服务端可能未正确响应".to_string());
    }

    // System (standalone, not already in combos)
    if system.details.proxy.enabled && tls.status != Status::Fail && !tls.details.cert.self_signed {
        manual.push("检测到系统代理已开启，如遇问题建议关闭代理后重试".to_string());
        if let Some(uri) = &system.details.proxy.settings_uri {
            quick.push(QuickAction {
                id: "open_proxy".to_string(),
                label: "打开代理设置".to_string(),
                kind: "open_uri".to_string(),
                target: uri.clone(),
            });
        }
    }

    if system.details.clock_skewed && tls.status != Status::Fail {
        manual.push(format!(
            "系统时钟偏差约 {} 秒，建议校准以避免潜在问题",
            system.details.clock_offset_sec.unwrap_or(0).abs()
        ));
        quick.push(QuickAction {
            id: "open_datetime".to_string(),
            label: "打开日期和时间设置".to_string(),
            kind: "open_uri".to_string(),
            target: "ms-settings:dateandtime".to_string(),
        });
    }

    if system.details.hosts_override {
        manual.push("检测到 hosts 文件有自定义条目，可能影响域名解析结果".to_string());
        quick.push(QuickAction {
            id: "open_hosts".to_string(),
            label: "打开 hosts 文件".to_string(),
            kind: "open_uri".to_string(),
            target: "notepad C:\\Windows\\System32\\drivers\\etc\\hosts".to_string(),
        });
    }

    // Deduplicate quick actions by id
    let mut seen = std::collections::HashSet::new();
    quick.retain(|a| seen.insert(a.id.clone()));

    RecommendedActions {
        manual_actions: manual,
        quick_actions: quick,
    }
}

/// Check if any existing manual action contains a keyword (to avoid duplicates).
fn has_action(actions: &[String], keyword: &str) -> bool {
    actions.iter().any(|a| a.contains(keyword))
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
    use std::collections::HashMap;

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

    // ── Helper factories for test modules ──

    fn ok_dns() -> DnsModule {
        DnsModule {
            status: Status::Pass,
            severity: Severity::Info,
            duration_ms: 10,
            error: None,
            details: DnsDetails {
                records: vec![DnsRecord {
                    record_type: "A".into(),
                    value: "93.184.216.34".into(),
                    ttl: 300,
                }],
                resolved: true,
                resolved_ip: Some("93.184.216.34".into()),
                suspected_hijack: false,
                private_ip: false,
            },
        }
    }

    fn hijack_dns() -> DnsModule {
        DnsModule {
            status: Status::Warn,
            severity: Severity::Warn,
            duration_ms: 10,
            error: None,
            details: DnsDetails {
                records: vec![DnsRecord {
                    record_type: "A".into(),
                    value: "192.168.1.1".into(),
                    ttl: 60,
                }],
                resolved: true,
                resolved_ip: Some("192.168.1.1".into()),
                suspected_hijack: true,
                private_ip: true,
            },
        }
    }

    fn ok_tcp() -> TcpModule {
        TcpModule {
            status: Status::Pass,
            severity: Severity::Info,
            duration_ms: 20,
            error: None,
            details: TcpDetails {
                connected: true,
                ip: Some("93.184.216.34".into()),
                port: 443,
            },
        }
    }

    fn fail_tcp() -> TcpModule {
        TcpModule {
            status: Status::Fail,
            severity: Severity::Fail,
            duration_ms: 10000,
            error: Some("TCP connection timed out (10s)".into()),
            details: TcpDetails {
                connected: false,
                ip: Some("93.184.216.34".into()),
                port: 443,
            },
        }
    }

    fn ok_tls() -> TlsModule {
        TlsModule {
            status: Status::Pass,
            severity: Severity::Info,
            duration_ms: 50,
            error: None,
            details: TlsDetails {
                handshake: true,
                version: Some("TLSv1_3".into()),
                cert: CertInfo {
                    valid: true,
                    expired: false,
                    expiring_soon: false,
                    days_remaining: Some(200),
                    domain_mismatch: false,
                    chain_incomplete: false,
                    self_signed: false,
                    issuer: Some("CN=R3, O=Let's Encrypt".into()),
                    subject: Some("CN=example.com".into()),
                    not_before: Some("2026-01-01".into()),
                    not_after: Some("2026-10-01".into()),
                },
            },
        }
    }

    fn fail_tls_self_signed() -> TlsModule {
        TlsModule {
            status: Status::Fail,
            severity: Severity::Fail,
            duration_ms: 50,
            error: Some("TLS handshake failed".into()),
            details: TlsDetails {
                handshake: false,
                version: None,
                cert: CertInfo {
                    valid: false,
                    expired: false,
                    expiring_soon: false,
                    days_remaining: Some(100),
                    domain_mismatch: false,
                    chain_incomplete: true,
                    self_signed: true,
                    issuer: Some("CN=Corporate Proxy".into()),
                    subject: Some("CN=Corporate Proxy".into()),
                    not_before: Some("2026-01-01".into()),
                    not_after: Some("2027-01-01".into()),
                },
            },
        }
    }

    fn ok_http() -> HttpModule {
        HttpModule {
            status: Status::Pass,
            severity: Severity::Info,
            duration_ms: 100,
            error: None,
            details: HttpDetails {
                status_code: Some(200),
                redirect_chain: vec![],
                headers: HashMap::new(),
                empty_body: false,
                downgraded: false,
            },
        }
    }

    fn fail_http() -> HttpModule {
        HttpModule {
            status: Status::Fail,
            severity: Severity::Fail,
            duration_ms: 100,
            error: Some("HTTP request failed".into()),
            details: HttpDetails {
                status_code: None,
                redirect_chain: vec![],
                headers: HashMap::new(),
                empty_body: true,
                downgraded: false,
            },
        }
    }

    fn ok_system() -> SystemModule {
        SystemModule {
            status: Status::Pass,
            severity: Severity::Info,
            duration_ms: 1,
            error: None,
            details: SystemDetails {
                proxy: ProxyInfo {
                    enabled: false,
                    proxy_type: None,
                    address: None,
                    pac_url: None,
                    env_var: None,
                    settings_uri: Some("ms-settings:network-proxy".into()),
                },
                clock_skewed: false,
                clock_offset_sec: None,
                hosts_override: false,
            },
        }
    }

    fn proxy_system() -> SystemModule {
        SystemModule {
            status: Status::Warn,
            severity: Severity::Warn,
            duration_ms: 1,
            error: None,
            details: SystemDetails {
                proxy: ProxyInfo {
                    enabled: true,
                    proxy_type: Some("system".into()),
                    address: Some("10.0.0.1:8080".into()),
                    pac_url: None,
                    env_var: None,
                    settings_uri: Some("ms-settings:network-proxy".into()),
                },
                clock_skewed: false,
                clock_offset_sec: None,
                hosts_override: false,
            },
        }
    }

    fn clock_skew_system() -> SystemModule {
        SystemModule {
            status: Status::Warn,
            severity: Severity::Warn,
            duration_ms: 1,
            error: Some("系统时钟偏差 3600 秒".into()),
            details: SystemDetails {
                proxy: ProxyInfo {
                    enabled: false,
                    proxy_type: None,
                    address: None,
                    pac_url: None,
                    env_var: None,
                    settings_uri: Some("ms-settings:network-proxy".into()),
                },
                clock_skewed: true,
                clock_offset_sec: Some(3600),
                hosts_override: false,
            },
        }
    }

    // ── Combo tests ──

    #[test]
    fn test_actions_all_pass() {
        let actions = generate_actions(&ok_dns(), &ok_tcp(), &ok_tls(), &ok_http(), &ok_system());
        assert!(actions.manual_actions.is_empty());
        assert!(actions.quick_actions.is_empty());
    }

    #[test]
    fn test_actions_combo_proxy_tls_mitm() {
        let actions = generate_actions(
            &ok_dns(),
            &ok_tcp(),
            &fail_tls_self_signed(),
            &ok_http(),
            &proxy_system(),
        );
        // Should have the corporate MITM combo action
        assert!(actions
            .manual_actions
            .iter()
            .any(|a| a.contains("企业代理拦截")));
        assert!(actions.quick_actions.iter().any(|a| a.id == "open_proxy"));
    }

    #[test]
    fn test_actions_combo_dns_hijack() {
        let actions = generate_actions(
            &hijack_dns(),
            &ok_tcp(),
            &ok_tls(),
            &fail_http(),
            &ok_system(),
        );
        assert!(actions
            .manual_actions
            .iter()
            .any(|a| a.contains("DNS 被劫持")));
    }

    #[test]
    fn test_actions_combo_clock_skew_tls() {
        let fail_tls = TlsModule {
            status: Status::Fail,
            severity: Severity::Fail,
            duration_ms: 50,
            error: Some("TLS handshake failed".into()),
            details: TlsDetails {
                handshake: false,
                version: None,
                cert: CertInfo::empty(),
            },
        };
        let actions = generate_actions(
            &ok_dns(),
            &ok_tcp(),
            &fail_tls,
            &ok_http(),
            &clock_skew_system(),
        );
        assert!(actions
            .manual_actions
            .iter()
            .any(|a| a.contains("时钟偏差")));
        assert!(actions
            .quick_actions
            .iter()
            .any(|a| a.id == "open_datetime"));
    }

    #[test]
    fn test_actions_tcp_fail() {
        let actions = generate_actions(&ok_dns(), &fail_tcp(), &ok_tls(), &ok_http(), &ok_system());
        assert!(actions
            .manual_actions
            .iter()
            .any(|a| a.contains("端口不可达")));
        assert!(actions.quick_actions.iter().any(|a| a.id == "open_network"));
    }

    #[test]
    fn test_actions_combo_tcp_ok_http_5xx() {
        let http_5xx = HttpModule {
            status: Status::Fail,
            severity: Severity::Fail,
            duration_ms: 100,
            error: None,
            details: HttpDetails {
                status_code: Some(502),
                redirect_chain: vec![],
                headers: HashMap::new(),
                empty_body: false,
                downgraded: false,
            },
        };
        let actions = generate_actions(&ok_dns(), &ok_tcp(), &ok_tls(), &http_5xx, &ok_system());
        // COMBO-03: should say it's a server-side issue
        assert!(actions
            .manual_actions
            .iter()
            .any(|a| a.contains("站点服务端")));
    }

    #[test]
    fn test_actions_hosts_override() {
        let mut sys = ok_system();
        sys.details.hosts_override = true;
        sys.status = Status::Warn;
        sys.severity = Severity::Warn;
        let actions = generate_actions(&ok_dns(), &ok_tcp(), &ok_tls(), &ok_http(), &sys);
        assert!(actions.manual_actions.iter().any(|a| a.contains("hosts")));
        assert!(actions.quick_actions.iter().any(|a| a.id == "open_hosts"));
    }

    #[test]
    fn test_actions_quick_actions_deduplicated() {
        // Proxy + TLS fail both want open_proxy — should appear only once
        let actions = generate_actions(
            &ok_dns(),
            &ok_tcp(),
            &fail_tls_self_signed(),
            &ok_http(),
            &proxy_system(),
        );
        let proxy_count = actions
            .quick_actions
            .iter()
            .filter(|a| a.id == "open_proxy")
            .count();
        assert_eq!(proxy_count, 1);
    }
}
