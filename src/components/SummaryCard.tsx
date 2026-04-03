import type { DiagnosticResult } from "../types/diagnostic";
import { StatusBadge } from "./StatusBadge";

interface SummaryCardProps {
  result: DiagnosticResult;
}

export function SummaryCard({ result }: SummaryCardProps) {
  const { summary, ipinfo } = result;

  return (
    <div className="card">
      <div className="card__title">诊断摘要</div>
      <div className="summary-row">
        <span className="summary-row__label">整体状态</span>
        <StatusBadge status={summary.status} />
      </div>
      <div className="summary-row">
        <span className="summary-row__label">总耗时</span>
        <span className="summary-row__value">
          {summary.total_duration_ms} ms
        </span>
      </div>
      <div className="summary-row">
        <span className="summary-row__label">解析 IP</span>
        <span className="summary-row__value">
          {summary.resolved_ip ?? "\u2014"}
        </span>
      </div>
      {summary.failure_stage && (
        <div className="summary-row">
          <span className="summary-row__label">失败阶段</span>
          <span className="summary-row__value">{summary.failure_stage}</span>
        </div>
      )}
      {ipinfo && (
        <>
          <div className="card__title" style={{ marginTop: 16 }}>
            客户端网络
          </div>
          <div className="summary-row">
            <span className="summary-row__label">公网 IP</span>
            <span className="summary-row__value">{ipinfo.ip}</span>
          </div>
          <div className="summary-row">
            <span className="summary-row__label">位置</span>
            <span className="summary-row__value">
              {ipinfo.city}, {ipinfo.region}, {ipinfo.country}
            </span>
          </div>
          <div className="summary-row">
            <span className="summary-row__label">运营商</span>
            <span className="summary-row__value">{ipinfo.org}</span>
          </div>
          <div className="summary-row">
            <span className="summary-row__label">时区</span>
            <span className="summary-row__value">{ipinfo.timezone}</span>
          </div>
        </>
      )}
    </div>
  );
}
