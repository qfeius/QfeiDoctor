import type { PhaseStatus } from "../types/diagnostic";

const LABELS: Record<PhaseStatus, string> = {
  pass: "Pass",
  warn: "Warn",
  fail: "Fail",
  skip: "Skip",
};

interface StatusBadgeProps {
  status: PhaseStatus;
}

export function StatusBadge({ status }: StatusBadgeProps) {
  return (
    <span className={`status-badge status-badge--${status}`}>
      {LABELS[status]}
    </span>
  );
}
