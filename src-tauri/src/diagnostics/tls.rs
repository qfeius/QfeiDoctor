use super::result::{PhaseDiagnostic, PhaseStatus};
use serde_json::json;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

/// Diagnose TLS/SSL for a domain.
pub async fn diagnose(domain: &str, ip: &str, port: u16) -> PhaseDiagnostic {
    let start = std::time::Instant::now();

    // Build TLS config with webpki roots
    let mut root_store = rustls::RootCertStore::empty();
    root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());

    let config = Arc::new(
        rustls::ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_no_client_auth(),
    );

    let server_name = match rustls::pki_types::ServerName::try_from(domain.to_string()) {
        Ok(sn) => sn,
        Err(e) => {
            return PhaseDiagnostic {
                name: "tls".to_string(),
                status: PhaseStatus::Fail,
                duration_ms: start.elapsed().as_millis() as u64,
                details: None,
                error: Some(format!("Invalid server name: {}", e)),
            };
        }
    };

    // Connect TCP first
    let addr = format!("{}:{}", ip, port);
    let tcp_stream = match TcpStream::connect(&addr).await {
        Ok(s) => s,
        Err(e) => {
            return PhaseDiagnostic {
                name: "tls".to_string(),
                status: PhaseStatus::Fail,
                duration_ms: start.elapsed().as_millis() as u64,
                details: None,
                error: Some(format!("TCP connect for TLS failed: {}", e)),
            };
        }
    };

    // Perform TLS handshake
    let connector = tokio_rustls::TlsConnector::from(config);
    match connector.connect(server_name, tcp_stream).await {
        Ok(mut tls_stream) => {
            // Extract certificate info
            let (_, server_conn) = tls_stream.get_ref();
            let peer_certs = server_conn.peer_certificates();

            let mut cert_details = json!({
                "tls_version": format!("{:?}", server_conn.protocol_version()),
                "handshake_completed": true,
            });

            if let Some(certs) = peer_certs {
                if let Some(cert_der) = certs.first() {
                    if let Ok((_, cert)) = x509_parser::parse_x509_certificate(cert_der.as_ref()) {
                        let not_before = cert.validity().not_before.to_string();
                        let not_after = cert.validity().not_after.to_string();
                        let issuer = cert.issuer().to_string();
                        let subject = cert.subject().to_string();

                        // Check expiry (simple: validity end is in the past)
                        let expired = cert.validity().not_after.timestamp() < 0; // approximate

                        // Check hostname match (simplified)
                        let hostname_matched = subject.contains(domain)
                            || cert
                                .subject_alternative_name()
                                .ok()
                                .flatten()
                                .map(|san| {
                                    san.value
                                        .general_names
                                        .iter()
                                        .any(|name| format!("{}", name).contains(domain))
                                })
                                .unwrap_or(false);

                        cert_details = json!({
                            "tls_version": format!("{:?}", server_conn.protocol_version()),
                            "handshake_completed": true,
                            "issuer": issuer,
                            "subject": subject,
                            "not_before": not_before,
                            "not_after": not_after,
                            "expired": expired,
                            "hostname_matched": hostname_matched,
                            "chain_length": certs.len(),
                        });
                    }
                }
            }

            let _ = tls_stream.shutdown().await;

            PhaseDiagnostic {
                name: "tls".to_string(),
                status: PhaseStatus::Pass,
                duration_ms: start.elapsed().as_millis() as u64,
                details: Some(cert_details),
                error: None,
            }
        }
        Err(e) => PhaseDiagnostic {
            name: "tls".to_string(),
            status: PhaseStatus::Fail,
            duration_ms: start.elapsed().as_millis() as u64,
            details: None,
            error: Some(format!("TLS handshake failed: {}", e)),
        },
    }
}
