import type { PhaseDiagnostic } from "../types/diagnostic";
import { StatusBadge } from "./StatusBadge";

const PHASE_LABELS: Record<string, string> = {
  dns: "Phase 1: DNS Resolution",
  tcp: "Phase 2: Network Reachability",
  tls: "Phase 3: TLS/SSL Negotiation",
  http: "Phase 4: HTTP Request",
  system: "Phase 5: System / Proxy",
};

interface PhaseSectionProps {
  phase: PhaseDiagnostic;
}

export function PhaseSection({ phase }: PhaseSectionProps) {
  const label = PHASE_LABELS[phase.name] ?? phase.name;

  return (
    <div className="phase-section">
      <div className="phase-section__header">
        <StatusBadge status={phase.status} />
        <span>{label}</span>
        <span className="phase-section__duration">{phase.duration_ms}ms</span>
      </div>
      <div className="phase-section__detail">
        {phase.error && <div>Error: {phase.error}</div>}
        {phase.details &&
          Object.entries(phase.details).map(([key, value]) => (
            <div key={key}>
              {key}:{" "}
              {typeof value === "object"
                ? JSON.stringify(value)
                : String(value)}
            </div>
          ))}
      </div>
    </div>
  );
}
