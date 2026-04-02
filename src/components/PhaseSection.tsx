import type { Status } from "../types/diagnostic";
import { StatusBadge } from "./StatusBadge";

const PHASE_LABELS: Record<string, string> = {
  dns: "Phase 1: DNS Resolution",
  tcp: "Phase 2: Network Reachability",
  tls: "Phase 3: TLS/SSL Negotiation",
  http: "Phase 4: HTTP Request",
  system: "Phase 5: System / Proxy",
};

/** Human-readable labels for detail keys, grouped by phase */
const DETAIL_LABELS: Record<string, Record<string, string>> = {
  dns: {
    resolved: "Resolved",
    resolved_ip: "Resolved IP",
    suspected_hijack: "DNS Hijack Suspected",
    private_ip: "Private IP",
  },
  tcp: {
    connected: "Connected",
    ip: "IP Address",
    port: "Port",
  },
  tls: {
    handshake: "Handshake",
    version: "TLS Version",
  },
  http: {
    status_code: "Status Code",
    empty_body: "Empty Body",
    downgraded: "Downgraded to HTTP",
  },
  system: {
    clock_skewed: "Clock Skewed",
    clock_offset_sec: "Clock Offset",
    hosts_override: "Hosts Override",
  },
};

function formatValue(value: unknown): string {
  if (value === null || value === undefined) return "—";
  if (typeof value === "boolean") return value ? "Yes" : "No";
  if (typeof value === "number") return String(value);
  return String(value);
}

function renderCert(cert: Record<string, unknown>) {
  const rows: [string, string][] = [
    ["Valid", formatValue(cert.valid)],
    ["Issuer", formatValue(cert.issuer)],
    ["Subject", formatValue(cert.subject)],
    ["Expires", formatValue(cert.not_after)],
    ["Days Remaining", formatValue(cert.days_remaining)],
    ["Expired", formatValue(cert.expired)],
    ["Expiring Soon", formatValue(cert.expiring_soon)],
    ["Domain Mismatch", formatValue(cert.domain_mismatch)],
    ["Chain Incomplete", formatValue(cert.chain_incomplete)],
    ["Self-Signed", formatValue(cert.self_signed)],
  ];
  return rows.map(([label, val]) => (
    <div key={label}>
      {label}: {val}
    </div>
  ));
}

function renderProxy(proxy: Record<string, unknown>) {
  const rows: [string, string][] = [
    ["Enabled", formatValue(proxy.enabled)],
    ["Type", formatValue(proxy.type)],
    ["Address", formatValue(proxy.address)],
    ["PAC URL", formatValue(proxy.pac_url)],
    ["Env Var", formatValue(proxy.env_var)],
  ];
  return rows.map(([label, val]) => (
    <div key={label}>
      {label}: {val}
    </div>
  ));
}

function renderRedirectChain(chain: unknown[]) {
  if (chain.length === 0) return <div>Redirect Chain: (none)</div>;
  return (
    <>
      <div>Redirect Chain:</div>
      {chain.map((url, i) => (
        <div key={i} style={{ paddingLeft: 12 }}>
          {i + 1}. {String(url)}
        </div>
      ))}
    </>
  );
}

function renderHeaders(headers: Record<string, unknown>) {
  const entries = Object.entries(headers);
  if (entries.length === 0) return <div>Response Headers: (none)</div>;
  return (
    <>
      <div>Response Headers:</div>
      {entries.map(([k, v]) => (
        <div key={k} style={{ paddingLeft: 12 }}>
          {k}: {String(v)}
        </div>
      ))}
    </>
  );
}

interface PhaseSectionProps {
  name: string;
  status: Status;
  duration_ms: number;
  error: string | null;
  details: Record<string, unknown>;
}

export function PhaseSection({
  name,
  status,
  duration_ms,
  error,
  details,
}: PhaseSectionProps) {
  const label = PHASE_LABELS[name] ?? name;
  const labels = DETAIL_LABELS[name] ?? {};

  return (
    <div className="phase-section">
      <div className="phase-section__header">
        <StatusBadge status={status} />
        <span>{label}</span>
        <span className="phase-section__duration">{duration_ms}ms</span>
      </div>
      <div className="phase-section__detail">
        {error && (
          <div style={{ color: "var(--accent-red)" }}>Error: {error}</div>
        )}
        {Object.entries(details).map(([key, value]) => {
          // Special renderers for nested objects
          if (key === "cert" && typeof value === "object" && value !== null) {
            return (
              <div key={key}>
                <div style={{ marginTop: 4, marginBottom: 2 }}>
                  Certificate:
                </div>
                <div style={{ paddingLeft: 12 }}>
                  {renderCert(value as Record<string, unknown>)}
                </div>
              </div>
            );
          }
          if (key === "proxy" && typeof value === "object" && value !== null) {
            return (
              <div key={key}>
                <div style={{ marginTop: 4, marginBottom: 2 }}>Proxy:</div>
                <div style={{ paddingLeft: 12 }}>
                  {renderProxy(value as Record<string, unknown>)}
                </div>
              </div>
            );
          }
          if (key === "redirect_chain" && Array.isArray(value)) {
            return <div key={key}>{renderRedirectChain(value)}</div>;
          }
          if (
            key === "headers" &&
            typeof value === "object" &&
            value !== null
          ) {
            return (
              <div key={key}>
                {renderHeaders(value as Record<string, unknown>)}
              </div>
            );
          }
          // DNS records are shown in DnsRecordsCard — skip here
          if (key === "records") return null;

          const displayLabel = labels[key] ?? key;
          return (
            <div key={key}>
              {displayLabel}: {formatValue(value)}
            </div>
          );
        })}
      </div>
    </div>
  );
}
