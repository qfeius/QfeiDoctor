import type { Status } from "../types/diagnostic";

const LABELS: Record<Status, string> = {
  pass: "通过",
  warn: "警告",
  fail: "失败",
  skip: "跳过",
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
