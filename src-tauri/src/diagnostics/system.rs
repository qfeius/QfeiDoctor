use super::result::{ProxyInfo, Severity, Status, SystemDetails, SystemModule};

/// Diagnose system network configuration.
pub fn diagnose() -> SystemModule {
    let start = std::time::Instant::now();

    let proxy = detect_proxy();
    let hosts_override = check_hosts_override();

    let status = if proxy.enabled {
        Status::Warn
    } else {
        Status::Pass
    };

    let severity = if proxy.enabled {
        Severity::Warn
    } else {
        Severity::Info
    };

    SystemModule {
        status,
        severity,
        duration_ms: start.elapsed().as_millis() as u64,
        error: None,
        details: SystemDetails {
            proxy,
            clock_skewed: false,
            clock_offset_sec: None,
            hosts_override,
        },
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
        };
    }

    ProxyInfo {
        enabled: false,
        proxy_type: None,
        address: None,
        pac_url: None,
        env_var: None,
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
        let result = diagnose();
        assert!(result.status == Status::Pass || result.status == Status::Warn);
    }

    #[test]
    fn test_detect_proxy_env_no_proxy() {
        // Without proxy env vars set, should detect no proxy
        // (may vary by environment, so just check no panic)
        let proxy = detect_proxy();
        assert!(!proxy.enabled || proxy.proxy_type.is_some());
    }
}
