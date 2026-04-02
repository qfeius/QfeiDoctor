use super::result::{PhaseDiagnostic, PhaseStatus};
use serde_json::json;
use std::net::ToSocketAddrs;
use tokio::net::TcpStream;
use tokio::time::{timeout, Duration};

const TCP_TIMEOUT_SECS: u64 = 10;

/// Diagnose TCP connectivity to an IP:port.
pub async fn diagnose(ip: &str, port: u16) -> PhaseDiagnostic {
    let start = std::time::Instant::now();
    let addr = format!("{}:{}", ip, port);

    // Resolve socket address
    let socket_addr = match addr.to_socket_addrs() {
        Ok(mut addrs) => match addrs.next() {
            Some(a) => a,
            None => {
                return PhaseDiagnostic {
                    name: "tcp".to_string(),
                    status: PhaseStatus::Fail,
                    duration_ms: start.elapsed().as_millis() as u64,
                    details: None,
                    error: Some(format!("Could not resolve address: {}", addr)),
                };
            }
        },
        Err(e) => {
            return PhaseDiagnostic {
                name: "tcp".to_string(),
                status: PhaseStatus::Fail,
                duration_ms: start.elapsed().as_millis() as u64,
                details: None,
                error: Some(format!("Invalid address {}: {}", addr, e)),
            };
        }
    };

    // Attempt TCP connection with timeout
    match timeout(
        Duration::from_secs(TCP_TIMEOUT_SECS),
        TcpStream::connect(socket_addr),
    )
    .await
    {
        Ok(Ok(_stream)) => PhaseDiagnostic {
            name: "tcp".to_string(),
            status: PhaseStatus::Pass,
            duration_ms: start.elapsed().as_millis() as u64,
            details: Some(json!({
                "address": addr,
                "connected": true,
            })),
            error: None,
        },
        Ok(Err(e)) => PhaseDiagnostic {
            name: "tcp".to_string(),
            status: PhaseStatus::Fail,
            duration_ms: start.elapsed().as_millis() as u64,
            details: Some(json!({
                "address": addr,
                "connected": false,
            })),
            error: Some(format!("TCP connection failed: {}", e)),
        },
        Err(_) => PhaseDiagnostic {
            name: "tcp".to_string(),
            status: PhaseStatus::Fail,
            duration_ms: start.elapsed().as_millis() as u64,
            details: Some(json!({
                "address": addr,
                "connected": false,
            })),
            error: Some(format!("TCP connection timed out ({}s)", TCP_TIMEOUT_SECS)),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_tcp_connect_known_host() {
        // example.com port 80 should be reachable
        let result = diagnose("93.184.216.34", 80).await;
        assert_eq!(result.name, "tcp");
        // May pass or fail depending on network, but should not panic
    }
}
