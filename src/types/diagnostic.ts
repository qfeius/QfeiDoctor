/**
 * TypeScript types mirroring the Rust DiagnosticReport.
 * Field names intentionally kept loose until PE's JSON schema v0 locks them.
 */

export type PhaseStatus = "pass" | "warn" | "fail" | "skip";

export interface PhaseDiagnostic {
  name: string;
  status: PhaseStatus;
  duration_ms: number;
  details?: Record<string, unknown>;
  error?: string;
}

export interface Suggestion {
  cause: string;
  action: string;
  quick_action?: string;
}

export interface DiagnosticReport {
  target: string;
  timestamp: string;
  overall_status: PhaseStatus;
  total_duration_ms: number;
  resolved_ip?: string;
  failure_stage?: string;
  phases: PhaseDiagnostic[];
  suggestions: Suggestion[];
}
