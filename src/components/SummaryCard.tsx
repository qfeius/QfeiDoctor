import type { DiagnosticResult } from "../types/diagnostic";
import { StatusBadge } from "./StatusBadge";

interface SummaryCardProps {
  result: DiagnosticResult;
}

export function SummaryCard({ result }: SummaryCardProps) {
  const { summary } = result;

  return (
    <div className="card">
      <div className="card__title">Analysis Summary</div>
      <div className="summary-row">
        <span className="summary-row__label">Overall Status</span>
        <StatusBadge status={summary.status} />
      </div>
      <div className="summary-row">
        <span className="summary-row__label">Total Duration</span>
        <span className="summary-row__value">
          {summary.total_duration_ms} ms
        </span>
      </div>
      <div className="summary-row">
        <span className="summary-row__label">Resolved IP</span>
        <span className="summary-row__value">
          {summary.resolved_ip ?? "\u2014"}
        </span>
      </div>
      {summary.failure_stage && (
        <div className="summary-row">
          <span className="summary-row__label">Failure Stage</span>
          <span className="summary-row__value">{summary.failure_stage}</span>
        </div>
      )}
    </div>
  );
}
