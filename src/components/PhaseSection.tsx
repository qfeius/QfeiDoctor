import type { Status } from "../types/diagnostic";
import { StatusBadge } from "./StatusBadge";

const PHASE_LABELS: Record<string, string> = {
  dns: "Phase 1: DNS Resolution",
  tcp: "Phase 2: Network Reachability",
  tls: "Phase 3: TLS/SSL Negotiation",
  http: "Phase 4: HTTP Request",
  system: "Phase 5: System / Proxy",
};

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

  return (
    <div className="phase-section">
      <div className="phase-section__header">
        <StatusBadge status={status} />
        <span>{label}</span>
        <span className="phase-section__duration">{duration_ms}ms</span>
      </div>
      <div className="phase-section__detail">
        {error && <div>Error: {error}</div>}
        {Object.entries(details).map(([key, value]) => (
          <div key={key}>
            {key}:{" "}
            {typeof value === "object" ? JSON.stringify(value) : String(value)}
          </div>
        ))}
      </div>
    </div>
  );
}
