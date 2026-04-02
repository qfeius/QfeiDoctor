import { open } from "@tauri-apps/plugin-shell";

interface ProxyAlertProps {
  proxyEnabled: boolean;
  proxyAddress?: string;
}

export function ProxyAlert({ proxyEnabled, proxyAddress }: ProxyAlertProps) {
  if (!proxyEnabled) return null;

  return (
    <div className="proxy-alert">
      <span className="proxy-alert__text">
        检测到系统代理已开启
        {proxyAddress ? `（${proxyAddress}）` : ""}
        ，这可能会影响网络诊断结果。建议暂时关闭代理设置后重新测试。
      </span>
      <button
        className="btn btn--outline proxy-alert__action"
        onClick={() => open("ms-settings:network-proxy")}
      >
        打开代理设置
      </button>
    </div>
  );
}
