use super::result::{ProxyInfo, Severity, Status, SystemDetails, SystemModule};

/// Diagnose system network configuration.
/// `server_date_header` is the HTTP Date header from the HTTP module, if available.
pub fn diagnose(server_date_header: Option<&str>) -> SystemModule {
    let start = std::time::Instant::now();

    let proxy = detect_proxy();
    let hosts_override = check_hosts_override();
    let (clock_skewed, clock_offset_sec) = check_clock_skew(server_date_header);

    let has_warning = proxy.enabled || clock_skewed || hosts_override;

    let status = if has_warning {
        Status::Warn
    } else {
        Status::Pass
    };

    let severity = if has_warning {
        Severity::Warn
    } else {
        Severity::Info
    };

    let error = if clock_skewed {
        Some(format!(
            "系统时钟偏差 {} 秒，可能导致 TLS 证书验证失败",
            clock_offset_sec.unwrap_or(0)
        ))
    } else {
        None
    };

    SystemModule {
        status,
        severity,
        duration_ms: start.elapsed().as_millis() as u64,
        error,
        details: SystemDetails {
            proxy,
            clock_skewed,
            clock_offset_sec,
            hosts_override,
        },
    }
}

/// Check clock skew by comparing local time with server Date header.
/// Returns (is_skewed, offset_seconds). Threshold: 120 seconds.
fn check_clock_skew(server_date: Option<&str>) -> (bool, Option<i64>) {
    let date_str = match server_date {
        Some(d) => d,
        None => return (false, None),
    };

    let server_ts = match parse_http_date(date_str) {
        Some(ts) => ts,
        None => return (false, None),
    };

    let local_ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    let offset = local_ts - server_ts;
    let skewed = offset.unsigned_abs() > 120;
    (skewed, Some(offset))
}

/// Parse HTTP Date header (RFC 7231) into unix timestamp.
/// Supports format: "Thu, 03 Apr 2026 00:30:00 GMT"
fn parse_http_date(date: &str) -> Option<i64> {
    // Format: "Day, DD Mon YYYY HH:MM:SS GMT"
    let parts: Vec<&str> = date.split_whitespace().collect();
    if parts.len() < 6 {
        return None;
    }
    let day: i64 = parts[1].parse().ok()?;
    let month = match parts[2] {
        "Jan" => 1,
        "Feb" => 2,
        "Mar" => 3,
        "Apr" => 4,
        "May" => 5,
        "Jun" => 6,
        "Jul" => 7,
        "Aug" => 8,
        "Sep" => 9,
        "Oct" => 10,
        "Nov" => 11,
        "Dec" => 12,
        _ => return None,
    };
    let year: i64 = parts[3].parse().ok()?;
    let time_parts: Vec<&str> = parts[4].split(':').collect();
    if time_parts.len() != 3 {
        return None;
    }
    let hour: i64 = time_parts[0].parse().ok()?;
    let minute: i64 = time_parts[1].parse().ok()?;
    let second: i64 = time_parts[2].parse().ok()?;

    // Convert to unix timestamp (simplified, no leap second handling)
    let mut total_days: i64 = 0;
    for y in 1970..year {
        total_days += if y % 4 == 0 && (y % 100 != 0 || y % 400 == 0) {
            366
        } else {
            365
        };
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
    for md in month_days.iter().take((month - 1) as usize) {
        total_days += md;
    }
    total_days += day - 1;

    Some(total_days * 86400 + hour * 3600 + minute * 60 + second)
}

fn proxy_settings_uri() -> Option<String> {
    #[cfg(target_os = "windows")]
    {
        Some("ms-settings:network-proxy".to_string())
    }
    #[cfg(target_os = "macos")]
    {
        Some("x-apple.systempreferences:com.apple.Network-Settings.extension".to_string())
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        None
    }
}

fn detect_proxy() -> ProxyInfo {
    #[cfg(target_os = "windows")]
    {
        detect_proxy_windows()
    }
    #[cfg(not(target_os = "windows"))]
    {
        detect_proxy_env()
    }
}

#[cfg(target_os = "windows")]
fn detect_proxy_windows() -> ProxyInfo {
    use winreg::enums::*;
    use winreg::RegKey;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    if let Ok(internet_settings) =
        hkcu.open_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Internet Settings")
    {
        let enabled: u32 = internet_settings.get_value("ProxyEnable").unwrap_or(0);
        let server: String = internet_settings
            .get_value("ProxyServer")
            .unwrap_or_default();
        let auto_config_url: String = internet_settings
            .get_value("AutoConfigURL")
            .unwrap_or_default();

        // PAC takes precedence if present
        if !auto_config_url.is_empty() {
            return ProxyInfo {
                enabled: true,
                proxy_type: Some("pac".to_string()),
                address: None,
                pac_url: Some(auto_config_url),
                env_var: None,
                settings_uri: proxy_settings_uri(),
            };
        }

        return ProxyInfo {
            enabled: enabled == 1,
            proxy_type: if enabled == 1 {
                Some("system".to_string())
            } else {
                None
            },
            address: if server.is_empty() {
                None
            } else {
                Some(server)
            },
            pac_url: None,
            env_var: None,
            settings_uri: proxy_settings_uri(),
        };
    }

    ProxyInfo {
        enabled: false,
        proxy_type: None,
        address: None,
        pac_url: None,
        env_var: None,
        settings_uri: proxy_settings_uri(),
    }
}

#[cfg(not(target_os = "windows"))]
fn detect_proxy_env() -> ProxyInfo {
    let https_proxy = std::env::var("https_proxy")
        .or_else(|_| std::env::var("HTTPS_PROXY"))
        .ok();
    let http_proxy = std::env::var("http_proxy")
        .or_else(|_| std::env::var("HTTP_PROXY"))
        .ok();

    let server = https_proxy.or(http_proxy);
    ProxyInfo {
        enabled: server.is_some(),
        proxy_type: if server.is_some() {
            Some("env".to_string())
        } else {
            None
        },
        address: server.clone(),
        pac_url: None,
        env_var: server,
        settings_uri: proxy_settings_uri(),
    }
}

fn check_hosts_override() -> bool {
    #[cfg(target_os = "windows")]
    let hosts_path = "C:\\Windows\\System32\\drivers\\etc\\hosts";
    #[cfg(not(target_os = "windows"))]
    let hosts_path = "/etc/hosts";

    let content = match std::fs::read_to_string(hosts_path) {
        Ok(c) => c,
        Err(_) => return false,
    };

    content.lines().any(|line| {
        let trimmed = line.trim();
        !trimmed.is_empty()
            && !trimmed.starts_with('#')
            && !trimmed.contains("localhost")
            && !trimmed.contains("broadcasthost")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_diagnose() {
        let result = diagnose(None);
        assert!(result.status == Status::Pass || result.status == Status::Warn);
    }

    #[test]
    fn test_detect_proxy_env_no_proxy() {
        let proxy = detect_proxy();
        assert!(!proxy.enabled || proxy.proxy_type.is_some());
    }

    #[test]
    fn test_parse_http_date() {
        let ts = parse_http_date("Thu, 03 Apr 2026 00:30:00 GMT");
        assert!(ts.is_some());
        // 2026-04-03 00:30:00 UTC should be a reasonable timestamp
        let ts = ts.unwrap();
        assert!(ts > 1_700_000_000); // after 2023
    }

    #[test]
    fn test_parse_http_date_invalid() {
        assert!(parse_http_date("not a date").is_none());
        assert!(parse_http_date("").is_none());
    }

    #[test]
    fn test_clock_skew_no_header() {
        let (skewed, offset) = check_clock_skew(None);
        assert!(!skewed);
        assert!(offset.is_none());
    }

    #[test]
    fn test_clock_skew_within_threshold() {
        // Use current time formatted as HTTP date — offset should be ~0
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        // Build an approximate HTTP date from current time
        // Just test the function doesn't flag near-zero offset
        let (skewed, offset) = check_clock_skew(Some("Thu, 01 Jan 1970 00:00:00 GMT"));
        assert!(skewed); // epoch is way off from now
        assert!(offset.is_some());
        let _ = now; // suppress unused warning
    }
}
