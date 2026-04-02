use super::result::{DnsDetails, DnsModule, DnsRecord, Severity, Status};
use hickory_resolver::Resolver;

/// Diagnose DNS resolution for a domain.
pub async fn diagnose(domain: &str) -> DnsModule {
    let start = std::time::Instant::now();

    let resolver = match Resolver::builder_tokio() {
        Ok(builder) => builder.build(),
        Err(e) => {
            return DnsModule {
                status: Status::Fail,
                severity: Severity::Fail,
                duration_ms: start.elapsed().as_millis() as u64,
                error: Some(format!("Failed to create DNS resolver: {}", e)),
                details: DnsDetails {
                    records: vec![],
                    resolved: false,
                    resolved_ip: None,
                    suspected_hijack: false,
                    private_ip: false,
                },
            };
        }
    };

    let records = match resolver.lookup_ip(domain).await {
        Ok(response) => {
            let ips: Vec<String> = response.iter().map(|ip| ip.to_string()).collect();
            ips
        }
        Err(e) => {
            return DnsModule {
                status: Status::Fail,
                severity: Severity::Fail,
                duration_ms: start.elapsed().as_millis() as u64,
                error: Some(format!("DNS resolution failed: {}", e)),
                details: DnsDetails {
                    records: vec![],
                    resolved: false,
                    resolved_ip: None,
                    suspected_hijack: false,
                    private_ip: false,
                },
            };
        }
    };

    if records.is_empty() {
        return DnsModule {
            status: Status::Fail,
            severity: Severity::Fail,
            duration_ms: start.elapsed().as_millis() as u64,
            error: Some("No DNS records found".to_string()),
            details: DnsDetails {
                records: vec![],
                resolved: false,
                resolved_ip: None,
                suspected_hijack: false,
                private_ip: false,
            },
        };
    }

    let resolved_ip = records.first().cloned().unwrap_or_default();
    let private_ip = is_private_ip(&resolved_ip);
    // All IPs resolving to private addresses for a public domain is suspicious
    let all_private = records.iter().all(|ip| is_private_ip(ip));
    let suspected_hijack = all_private && private_ip;

    let dns_records: Vec<DnsRecord> = records
        .iter()
        .map(|ip| {
            let record_type = if ip.contains(':') {
                "AAAA".to_string()
            } else {
                "A".to_string()
            };
            DnsRecord {
                record_type,
                value: ip.clone(),
                ttl: 0, // TTL not available from lookup_ip
            }
        })
        .collect();

    // Warn if resolved to private IP (possible DNS hijack or local override)
    let (status, severity, error) = if suspected_hijack {
        (
            Status::Warn,
            Severity::Warn,
            Some(format!(
                "域名解析到内网地址 {}，可能存在 DNS 劫持或 hosts 文件篡改",
                resolved_ip
            )),
        )
    } else {
        (Status::Pass, Severity::Info, None)
    };

    DnsModule {
        status,
        severity,
        duration_ms: start.elapsed().as_millis() as u64,
        error,
        details: DnsDetails {
            records: dns_records,
            resolved: true,
            resolved_ip: Some(resolved_ip),
            suspected_hijack,
            private_ip,
        },
    }
}

fn is_private_ip(ip: &str) -> bool {
    if ip.starts_with("10.") || ip.starts_with("192.168.") || ip.starts_with("127.") {
        return true;
    }
    // RFC 1918: 172.16.0.0 - 172.31.255.255
    if let Some(rest) = ip.strip_prefix("172.") {
        if let Some(second_octet) = rest.split('.').next() {
            if let Ok(octet) = second_octet.parse::<u8>() {
                return (16..=31).contains(&octet);
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // requires network
    async fn test_dns_resolve_known_domain() {
        let result = diagnose("example.com").await;
        assert_eq!(result.status, Status::Pass);
        assert!(result.details.resolved);
        assert!(result.details.resolved_ip.is_some());
    }

    #[tokio::test]
    #[ignore] // requires network
    async fn test_dns_resolve_invalid_domain() {
        let result = diagnose("thisdomaindoesnotexist12345.invalid").await;
        // Just verify no panic
        assert!(!result.details.records.is_empty() || result.error.is_some());
    }

    #[test]
    fn test_is_private_ip() {
        assert!(is_private_ip("192.168.1.1"));
        assert!(is_private_ip("10.0.0.1"));
        assert!(is_private_ip("127.0.0.1"));
        assert!(!is_private_ip("8.8.8.8"));
        assert!(!is_private_ip("104.18.22.45"));
    }

    #[test]
    fn test_is_private_ip_172_range() {
        assert!(is_private_ip("172.16.0.1"));
        assert!(is_private_ip("172.31.255.254"));
        assert!(!is_private_ip("172.32.0.1"));
        assert!(!is_private_ip("172.15.0.1"));
    }
}
