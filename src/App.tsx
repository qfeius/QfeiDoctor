import { useState, useEffect, useCallback } from "react";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import { Header } from "./components/Header";
import { ProxyAlert } from "./components/ProxyAlert";
import { InputBar } from "./components/InputBar";
import { SummaryCard } from "./components/SummaryCard";
import { DnsRecordsCard } from "./components/DnsRecordsCard";
import { DiagnosticTrace } from "./components/DiagnosticTrace";

import { useDiagnostic } from "./hooks/useDiagnostic";

function App() {
  const { isRunning, result, error, start } = useDiagnostic();
  const [target, setTarget] = useState("https://contract.qfei.cn");
  const [copyToast, setCopyToast] = useState(false);

  const handleStart = useCallback(() => {
    const trimmed = target.trim();
    if (trimmed && !isRunning) start(trimmed);
  }, [target, isRunning, start]);

  const handleCopy = useCallback(async () => {
    if (!result) return;
    try {
      await writeText(JSON.stringify(result, null, 2));
      setCopyToast(true);
      setTimeout(() => setCopyToast(false), 1000);
    } catch (e) {
      console.error("clipboard write failed:", e);
    }
  }, [result]);

  useEffect(() => {
    const onKeyDown = (e: KeyboardEvent) => {
      const mod = e.metaKey || e.ctrlKey;
      if (mod && e.key === "c" && !window.getSelection()?.toString()) {
        e.preventDefault();
        handleCopy();
      }
      if (mod && e.key === "r") {
        e.preventDefault();
        handleStart();
      }
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [handleCopy, handleStart]);

  return (
    <div className="app">
      <Header />

      {result && <ProxyAlert proxy={result.system.details.proxy} />}

      <InputBar
        target={target}
        onTargetChange={setTarget}
        onStart={handleStart}
        onCopy={handleCopy}
        isRunning={isRunning}
        hasResult={!!result}
      />

      {error && <div className="error-banner">{error}</div>}

      {copyToast && <div className="toast">已复制到剪贴板</div>}

      {isRunning && <div className="loading">正在诊断中，请稍候...</div>}

      {result && (
        <>
          <div className="result-layout">
            <div>
              <SummaryCard result={result} />
              <DnsRecordsCard dns={result.dns} />
            </div>
            <DiagnosticTrace result={result} />
          </div>
        </>
      )}
    </div>
  );
}

export default App;
