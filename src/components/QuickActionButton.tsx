import { open } from "@tauri-apps/plugin-shell";
import type { QuickAction } from "../types/diagnostic";

interface QuickActionButtonProps {
  action: QuickAction;
}

export function QuickActionButton({ action }: QuickActionButtonProps) {
  return (
    <button className="btn btn--outline" onClick={() => open(action.target)}>
      {action.label}
    </button>
  );
}
