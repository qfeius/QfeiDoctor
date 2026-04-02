import type { DiagnosticReport } from "../types/diagnostic";
import { StatusBadge } from "./StatusBadge";

interface SummaryCardProps {
  report: DiagnosticReport;
}

export function SummaryCard({ report }: SummaryCardProps) {
  return (
    <div className="card">
      <div className="card__title">Analysis Summary</div>
      <div className="summary-row">
        <span className="summary-row__label">Overall Status</span>
        <StatusBadge status={report.overall_status} />
      </div>
      <div className="summary-row">
        <span className="summary-row__label">Total Duration</span>
        <span className="summary-row__value">
          {report.total_duration_ms} ms
        </span>
      </div>
      <div className="summary-row">
        <span className="summary-row__label">Resolved IP</span>
        <span className="summary-row__value">{report.resolved_ip ?? "—"}</span>
      </div>
      {report.failure_stage && (
        <div className="summary-row">
          <span className="summary-row__label">Failure Stage</span>
          <span className="summary-row__value">{report.failure_stage}</span>
        </div>
      )}
    </div>
  );
}
