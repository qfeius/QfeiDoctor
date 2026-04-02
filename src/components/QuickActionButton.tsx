import { open } from "@tauri-apps/plugin-shell";

const ACTION_LABELS: Record<string, string> = {
  "ms-settings:network-proxy": "打开代理设置",
  "ms-settings:network-status": "打开网络状态",
  "ms-settings:network-ethernet": "打开网络设置",
  "certmgr.msc": "打开证书管理器",
  "ncpa.cpl": "打开网络连接",
};

interface QuickActionButtonProps {
  uri: string;
}

export function QuickActionButton({ uri }: QuickActionButtonProps) {
  const label = ACTION_LABELS[uri] ?? uri;

  return (
    <button className="btn btn--outline" onClick={() => open(uri)}>
      {label}
    </button>
  );
}
