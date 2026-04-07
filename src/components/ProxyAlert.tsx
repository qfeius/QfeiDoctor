import { open } from "@tauri-apps/plugin-shell";
import type { ProxyInfo } from "../types/diagnostic";

interface ProxyAlertProps {
  proxy: ProxyInfo;
}

export function ProxyAlert({ proxy }: ProxyAlertProps) {
  if (!proxy.enabled) return null;

  return (
    <div className="proxy-alert">
      <span className="proxy-alert__text">
        检测到系统代理已开启
        {proxy.address ? `（${proxy.address}）` : ""}
        ，这可能会影响网络诊断结果。建议暂时关闭代理设置后重新测试。
      </span>
      {proxy.settings_uri && (
        <button
          className="btn btn--outline proxy-alert__action"
          onClick={() => open(proxy.settings_uri!)}
        >
          打开代理设置
        </button>
      )}
    </div>
  );
}
