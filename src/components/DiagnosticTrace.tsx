import type { PhaseDiagnostic } from "../types/diagnostic";
import { PhaseSection } from "./PhaseSection";

interface DiagnosticTraceProps {
  phases: PhaseDiagnostic[];
}

export function DiagnosticTrace({ phases }: DiagnosticTraceProps) {
  return (
    <div className="trace">
      <div className="trace__title">diagnostic_trace.log</div>
      {phases.map((phase) => (
        <PhaseSection key={phase.name} phase={phase} />
      ))}
    </div>
  );
}
