import type { Status } from "../types/diagnostic";

const LABELS: Record<Status, string> = {
  pass: "Pass",
  warn: "Warn",
  fail: "Fail",
  skip: "Skip",
};

interface StatusBadgeProps {
  status: Status;
}

export function StatusBadge({ status }: StatusBadgeProps) {
  return (
    <span className={`status-badge status-badge--${status}`}>
      {LABELS[status]}
    </span>
  );
}
