import { useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { DiagnosticResult } from "../types/diagnostic";

export function useDiagnostic() {
  const [isRunning, setIsRunning] = useState(false);
  const [result, setResult] = useState<DiagnosticResult | null>(null);
  const [error, setError] = useState<string | null>(null);

  const start = useCallback(async (target: string) => {
    setIsRunning(true);
    setError(null);
    setResult(null);
    try {
      const report = await invoke<DiagnosticResult>("run_diagnosis", {
        target,
      });
      setResult(report);
    } catch (err) {
      setError(String(err));
    } finally {
      setIsRunning(false);
    }
  }, []);

  return { isRunning, result, error, start };
}
