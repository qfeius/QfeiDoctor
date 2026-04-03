/*!
 * [INPUT]: reqwest HTTP client
 * [OUTPUT]: IpInfo struct with client's public IP metadata
 * [POS]: diagnostics module — independent probe, runs parallel to main phases
 * [PROTOCOL]: 变更时更新此头部，然后检查 CLAUDE.md
 */

use super::result::IpInfo;

/// Fetch client IP info from ipinfo.io (best-effort, 5s timeout).
pub async fn fetch() -> Option<IpInfo> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .ok()?;

    let resp: serde_json::Value = client
        .get("https://ipinfo.io/json")
        .send()
        .await
        .ok()?
        .json()
        .await
        .ok()?;

    Some(IpInfo {
        ip: json_str(&resp, "ip"),
        city: json_str(&resp, "city"),
        region: json_str(&resp, "region"),
        country: json_str(&resp, "country"),
        loc: json_str(&resp, "loc"),
        org: json_str(&resp, "org"),
        postal: json_str(&resp, "postal"),
        timezone: json_str(&resp, "timezone"),
    })
}

fn json_str(v: &serde_json::Value, key: &str) -> String {
    v.get(key)
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string()
}
