use super::result::{PhaseDiagnostic, PhaseStatus};
use serde_json::json;

/// Diagnose system network configuration (proxy, hosts, DNS settings).
pub fn diagnose() -> PhaseDiagnostic {
    let start = std::time::Instant::now();

    let proxy_info = detect_proxy();
    let hosts_info = check_hosts_file();

    let status = if proxy_info.enabled {
        PhaseStatus::Warn
    } else {
        PhaseStatus::Pass
    };

    PhaseDiagnostic {
        name: "system".to_string(),
        status,
        duration_ms: start.elapsed().as_millis() as u64,
        details: Some(json!({
            "proxy_enabled": proxy_info.enabled,
            "proxy_server": proxy_info.server,
            "hosts_modified": hosts_info.modified,
            "hosts_entries": hosts_info.custom_entries,
            "os": std::env::consts::OS,
            "arch": std::env::consts::ARCH,
        })),
        error: None,
    }
}

struct ProxyInfo {
    enabled: bool,
    server: Option<String>,
}

struct HostsInfo {
    modified: bool,
    custom_entries: usize,
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
            server: if server.is_empty() {
                None
            } else {
                Some(server)
            },
        };
    }

    ProxyInfo {
        enabled: false,
        server: None,
    }
}

#[cfg(not(target_os = "windows"))]
fn detect_proxy_env() -> ProxyInfo {
    let http_proxy = std::env::var("http_proxy")
        .or_else(|_| std::env::var("HTTP_PROXY"))
        .ok();
    let https_proxy = std::env::var("https_proxy")
        .or_else(|_| std::env::var("HTTPS_PROXY"))
        .ok();

    let server = https_proxy.or(http_proxy);
    ProxyInfo {
        enabled: server.is_some(),
        server,
    }
}

fn check_hosts_file() -> HostsInfo {
    #[cfg(target_os = "windows")]
    let hosts_path = "C:\\Windows\\System32\\drivers\\etc\\hosts";
    #[cfg(not(target_os = "windows"))]
    let hosts_path = "/etc/hosts";

    let content = match std::fs::read_to_string(hosts_path) {
        Ok(c) => c,
        Err(_) => {
            return HostsInfo {
                modified: false,
                custom_entries: 0,
            }
        }
    };

    let custom_entries = content
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            !trimmed.is_empty()
                && !trimmed.starts_with('#')
                && !trimmed.contains("localhost")
                && !trimmed.contains("broadcasthost")
        })
        .count();

    HostsInfo {
        modified: custom_entries > 0,
        custom_entries,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_diagnose() {
        let result = diagnose();
        assert_eq!(result.name, "system");
        assert!(result.details.is_some());
    }
}
