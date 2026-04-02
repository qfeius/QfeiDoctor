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

  const handleCopy = async () => {
    if (!result) return;
    await writeText(JSON.stringify(result, null, 2));
  };

  // Derive proxy state from system phase
  const systemPhase = result?.phases.find((p) => p.name === "system");
  const proxyEnabled = systemPhase?.details?.proxy_enabled === true;
  const proxyAddress = systemPhase?.details?.proxy_address as
    | string
    | undefined;

  // DNS phase for records card
  const dnsPhase = result?.phases.find((p) => p.name === "dns");

  return (
    <div className="app">
      <Header />

      {result && (
        <ProxyAlert proxyEnabled={proxyEnabled} proxyAddress={proxyAddress} />
      )}

      <InputBar
        onStart={start}
        onCopy={handleCopy}
        isRunning={isRunning}
        hasResult={!!result}
      />

      {error && <div className="error-banner">{error}</div>}

      {isRunning && <div className="loading">正在诊断中，请稍候...</div>}

      {result && (
        <>
          <div className="result-layout">
            <div>
              <SummaryCard report={result} />
              {dnsPhase && <DnsRecordsCard dnsPhase={dnsPhase} />}
            </div>
            <DiagnosticTrace phases={result.phases} />
          </div>

          <RecommendedActions suggestions={result.suggestions} />
        </>
      )}
    </div>
  );
}

export default App;
