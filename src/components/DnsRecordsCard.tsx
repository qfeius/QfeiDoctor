import type { PhaseDiagnostic } from "../types/diagnostic";

interface DnsRecordsCardProps {
  dnsPhase: PhaseDiagnostic;
}

interface DnsRecord {
  type: string;
  value: string;
  ttl?: number;
}

export function DnsRecordsCard({ dnsPhase }: DnsRecordsCardProps) {
  const records = (dnsPhase.details?.records as DnsRecord[] | undefined) ?? [];

  if (records.length === 0) return null;

  return (
    <div className="card" style={{ marginTop: 16 }}>
      <div className="card__title">
        DNS Records{" "}
        <span style={{ fontWeight: 400, color: "#999" }}>
          {records.length} found
        </span>
      </div>
      {records.map((record, i) => (
        <div className="dns-record" key={i}>
          <span className="dns-record__type">{record.type}</span>
          <span>{record.value}</span>
          {record.ttl != null && (
            <span style={{ color: "#999", marginLeft: "auto" }}>
              TTL: {record.ttl}s
            </span>
          )}
        </div>
      ))}
    </div>
  );
}
