use super::result::{CertInfo, Severity, Status, TlsDetails, TlsModule};
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

/// Diagnose TLS/SSL for a domain.
pub async fn diagnose(domain: &str, ip: &str, port: u16) -> TlsModule {
    let start = std::time::Instant::now();

    // Ensure crypto provider is installed (rustls 0.23 requires explicit init)
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();

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
            return TlsModule {
                status: Status::Fail,
                severity: Severity::Fail,
                duration_ms: start.elapsed().as_millis() as u64,
                error: Some(format!("Invalid server name: {}", e)),
                details: TlsDetails {
                    handshake: false,
                    version: None,
                    cert: CertInfo::empty(),
                },
            };
        }
    };

    let addr = format!("{}:{}", ip, port);
    let tcp_stream = match TcpStream::connect(&addr).await {
        Ok(s) => s,
        Err(e) => {
            return TlsModule {
                status: Status::Fail,
                severity: Severity::Fail,
                duration_ms: start.elapsed().as_millis() as u64,
                error: Some(format!("TCP connect for TLS failed: {}", e)),
                details: TlsDetails {
                    handshake: false,
                    version: None,
                    cert: CertInfo::empty(),
                },
            };
        }
    };

    let connector = tokio_rustls::TlsConnector::from(config);
    match connector.connect(server_name, tcp_stream).await {
        Ok(mut tls_stream) => {
            let (_, server_conn) = tls_stream.get_ref();
            let tls_version = server_conn.protocol_version().map(|v| format!("{:?}", v));
            let peer_certs = server_conn.peer_certificates();

            let cert = extract_cert_info(domain, peer_certs);

            let status = if cert.expired || cert.domain_mismatch || !cert.valid {
                Status::Fail
            } else if cert.expiring_soon || cert.self_signed {
                Status::Warn
            } else {
                Status::Pass
            };

            let severity = match status {
                Status::Fail => Severity::Fail,
                Status::Warn => Severity::Warn,
                _ => Severity::Info,
            };

            let _ = tls_stream.shutdown().await;

            TlsModule {
                status,
                severity,
                duration_ms: start.elapsed().as_millis() as u64,
                error: None,
                details: TlsDetails {
                    handshake: true,
                    version: tls_version,
                    cert,
                },
            }
        }
        Err(e) => TlsModule {
            status: Status::Fail,
            severity: Severity::Fail,
            duration_ms: start.elapsed().as_millis() as u64,
            error: Some(format!("TLS handshake failed: {}", e)),
            details: TlsDetails {
                handshake: false,
                version: None,
                cert: CertInfo::empty(),
            },
        },
    }
}

fn extract_cert_info(
    domain: &str,
    peer_certs: Option<&[rustls::pki_types::CertificateDer<'_>]>,
) -> CertInfo {
    let certs = match peer_certs {
        Some(c) if !c.is_empty() => c,
        _ => return CertInfo::empty(),
    };

    let cert_der = &certs[0];
    let (_, cert) = match x509_parser::parse_x509_certificate(cert_der.as_ref()) {
        Ok(r) => r,
        Err(_) => return CertInfo::empty(),
    };

    let not_before = cert.validity().not_before.to_string();
    let not_after = cert.validity().not_after.to_string();
    let issuer = cert.issuer().to_string();
    let subject = cert.subject().to_string();

    // Check expiry
    let now_secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    let not_after_ts = cert.validity().not_after.timestamp();
    let not_before_ts = cert.validity().not_before.timestamp();
    let expired = not_after_ts < now_secs;
    let days_remaining = if not_after_ts > now_secs {
        Some((not_after_ts - now_secs) / 86400)
    } else {
        Some(0)
    };
    let expiring_soon = days_remaining.is_some_and(|d| d <= 30 && d > 0);

    // Check domain match
    let domain_matched = subject.contains(domain)
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

    // Check self-signed
    let self_signed = cert.issuer() == cert.subject();

    // Check chain completeness (rough: single cert in chain is suspicious)
    let chain_incomplete = certs.len() < 2;

    // Valid = not expired, domain matches, chain looks ok
    let valid = !expired && domain_matched && not_before_ts <= now_secs;

    CertInfo {
        valid,
        expired,
        expiring_soon,
        days_remaining,
        domain_mismatch: !domain_matched,
        chain_incomplete,
        self_signed,
        issuer: Some(issuer),
        subject: Some(subject),
        not_before: Some(not_before),
        not_after: Some(not_after),
    }
}
