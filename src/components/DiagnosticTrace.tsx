import type { DiagnosticResult } from "../types/diagnostic";
import { PhaseSection } from "./PhaseSection";

interface DiagnosticTraceProps {
  result: DiagnosticResult;
}

const STATUS_ORDER: Record<string, number> = {
  fail: 0,
  warn: 1,
  pass: 2,
  skip: 3,
};

export function DiagnosticTrace({ result }: DiagnosticTraceProps) {
  const phases = [
    { name: "dns", ...result.dns },
    { name: "tcp", ...result.tcp },
    { name: "tls", ...result.tls },
    { name: "http", ...result.http },
    { name: "system", ...result.system },
  ].sort((a, b) => (STATUS_ORDER[a.status] ?? 9) - (STATUS_ORDER[b.status] ?? 9));

  return (
    <div className="trace">
      <div className="trace__title">diagnostic_trace.log</div>
      {phases.map((phase) => (
        <PhaseSection
          key={phase.name}
          name={phase.name}
          status={phase.status}
          duration_ms={phase.duration_ms}
          error={phase.error}
          details={phase.details as unknown as Record<string, unknown>}
        />
      ))}
    </div>
  );
}
