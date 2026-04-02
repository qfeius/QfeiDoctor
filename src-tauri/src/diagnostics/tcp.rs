use super::result::{Severity, Status, TcpDetails, TcpModule};
use std::net::ToSocketAddrs;
use tokio::net::TcpStream;
use tokio::time::{timeout, Duration};

const TCP_TIMEOUT_SECS: u64 = 10;

/// Diagnose TCP connectivity to an IP:port.
pub async fn diagnose(ip: &str, port: u16) -> TcpModule {
    let start = std::time::Instant::now();
    let addr = format!("{}:{}", ip, port);

    let socket_addr = match addr.to_socket_addrs() {
        Ok(mut addrs) => match addrs.next() {
            Some(a) => a,
            None => {
                return TcpModule {
                    status: Status::Fail,
                    severity: Severity::Fail,
                    duration_ms: start.elapsed().as_millis() as u64,
                    error: Some(format!("Could not resolve address: {}", addr)),
                    details: TcpDetails {
                        connected: false,
                        ip: Some(ip.to_string()),
                        port,
                    },
                };
            }
        },
        Err(e) => {
            return TcpModule {
                status: Status::Fail,
                severity: Severity::Fail,
                duration_ms: start.elapsed().as_millis() as u64,
                error: Some(format!("Invalid address {}: {}", addr, e)),
                details: TcpDetails {
                    connected: false,
                    ip: Some(ip.to_string()),
                    port,
                },
            };
        }
    };

    match timeout(
        Duration::from_secs(TCP_TIMEOUT_SECS),
        TcpStream::connect(socket_addr),
    )
    .await
    {
        Ok(Ok(_stream)) => TcpModule {
            status: Status::Pass,
            severity: Severity::Info,
            duration_ms: start.elapsed().as_millis() as u64,
            error: None,
            details: TcpDetails {
                connected: true,
                ip: Some(ip.to_string()),
                port,
            },
        },
        Ok(Err(e)) => TcpModule {
            status: Status::Fail,
            severity: Severity::Fail,
            duration_ms: start.elapsed().as_millis() as u64,
            error: Some(format!("TCP connection failed: {}", e)),
            details: TcpDetails {
                connected: false,
                ip: Some(ip.to_string()),
                port,
            },
        },
        Err(_) => TcpModule {
            status: Status::Fail,
            severity: Severity::Fail,
            duration_ms: start.elapsed().as_millis() as u64,
            error: Some(format!("TCP connection timed out ({}s)", TCP_TIMEOUT_SECS)),
            details: TcpDetails {
                connected: false,
                ip: Some(ip.to_string()),
                port,
            },
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // requires network
    async fn test_tcp_connect_known_host() {
        let result = diagnose("93.184.216.34", 80).await;
        assert_eq!(result.details.port, 80);
    }
}
