import type { Status } from "../types/diagnostic";
import { StatusBadge } from "./StatusBadge";

const PHASE_LABELS: Record<string, string> = {
  dns: "阶段 1：DNS 解析",
  tcp: "阶段 2：网络连通性",
  tls: "阶段 3：TLS/SSL 握手",
  http: "阶段 4：HTTP 请求",
  system: "阶段 5：系统 / 代理",
};

/** Human-readable labels for detail keys, grouped by phase */
const DETAIL_LABELS: Record<string, Record<string, string>> = {
  dns: {
    resolved: "已解析",
    resolved_ip: "解析 IP",
    suspected_hijack: "疑似 DNS 劫持",
    private_ip: "内网 IP",
  },
  tcp: {
    connected: "已连接",
    ip: "IP 地址",
    port: "端口",
  },
  tls: {
    handshake: "握手",
    version: "TLS 版本",
  },
  http: {
    status_code: "状态码",
    empty_body: "响应体为空",
    downgraded: "降级为 HTTP",
  },
  system: {
    clock_skewed: "时钟偏移",
    clock_offset_sec: "偏移量",
    hosts_override: "Hosts 覆盖",
  },
};

function formatValue(value: unknown): string {
  if (value === null || value === undefined) return "—";
  if (typeof value === "boolean") return value ? "是" : "否";
  if (typeof value === "number") return String(value);
  return String(value);
}

function renderCert(cert: Record<string, unknown>) {
  const rows: [string, string][] = [
    ["有效", formatValue(cert.valid)],
    ["颁发者", formatValue(cert.issuer)],
    ["主题", formatValue(cert.subject)],
    ["过期时间", formatValue(cert.not_after)],
    ["剩余天数", formatValue(cert.days_remaining)],
    ["已过期", formatValue(cert.expired)],
    ["即将过期", formatValue(cert.expiring_soon)],
    ["域名不匹配", formatValue(cert.domain_mismatch)],
    ["证书链不完整", formatValue(cert.chain_incomplete)],
    ["自签名", formatValue(cert.self_signed)],
  ];
  return rows.map(([label, val]) => (
    <div key={label}>
      {label}: {val}
    </div>
  ));
}

function renderProxy(proxy: Record<string, unknown>) {
  const rows: [string, string][] = [
    ["已启用", formatValue(proxy.enabled)],
    ["类型", formatValue(proxy.type)],
    ["地址", formatValue(proxy.address)],
    ["PAC URL", formatValue(proxy.pac_url)],
    ["环境变量", formatValue(proxy.env_var)],
  ];
  return rows.map(([label, val]) => (
    <div key={label}>
      {label}: {val}
    </div>
  ));
}

function renderRedirectChain(chain: unknown[]) {
  if (chain.length === 0) return <div>重定向链：（无）</div>;
  return (
    <>
      <div>重定向链：</div>
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
  if (entries.length === 0) return <div>响应头：（无）</div>;
  return (
    <>
      <div>响应头：</div>
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
          <div style={{ color: "var(--accent-red)" }}>错误：{error}</div>
        )}
        {Object.entries(details).map(([key, value]) => {
          // Special renderers for nested objects
          if (key === "cert" && typeof value === "object" && value !== null) {
            return (
              <div key={key}>
                <div style={{ marginTop: 4, marginBottom: 2 }}>
                  证书：
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
                <div style={{ marginTop: 4, marginBottom: 2 }}>代理：</div>
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
