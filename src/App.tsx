import { useState } from "react";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import { Header } from "./components/Header";
import { ProxyAlert } from "./components/ProxyAlert";
import { InputBar } from "./components/InputBar";
import { SummaryCard } from "./components/SummaryCard";
import { DnsRecordsCard } from "./components/DnsRecordsCard";
import { DiagnosticTrace } from "./components/DiagnosticTrace";
import { RecommendedActions } from "./components/RecommendedActions";
import { useDiagnostic } from "./hooks/useDiagnostic";

function App() {
  const { isRunning, result, error, start } = useDiagnostic();
  const [copyToast, setCopyToast] = useState(false);

  const handleCopy = async () => {
    if (!result) return;
    await writeText(JSON.stringify(result, null, 2));
    setCopyToast(true);
    setTimeout(() => setCopyToast(false), 2000);
  };

  return (
    <div className="app">
      <Header />

      {result && <ProxyAlert proxy={result.system.details.proxy} />}

      <InputBar
        onStart={start}
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

          <RecommendedActions actions={result.recommended_actions} />
        </>
      )}
    </div>
  );
}

export default App;
