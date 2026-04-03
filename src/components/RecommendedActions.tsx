import type { RecommendedActions as RecommendedActionsType } from "../types/diagnostic";
import { QuickActionButton } from "./QuickActionButton";

interface RecommendedActionsProps {
  actions: RecommendedActionsType;
}

export function RecommendedActions({ actions }: RecommendedActionsProps) {
  const hasActions =
    actions.manual_actions.length > 0 || actions.quick_actions.length > 0;

  if (!hasActions) return null;

  return (
    <div className="actions">
      <div className="actions__title">建议操作</div>

      {actions.manual_actions.map((text, i) => (
        <div className="action-item" key={i}>
          <span className="action-item__suggestion">{text}</span>
        </div>
      ))}

      {actions.quick_actions.length > 0 && (
        <div style={{ display: "flex", gap: 8, marginTop: 8 }}>
          {actions.quick_actions.map((qa) => (
            <QuickActionButton key={qa.id} action={qa} />
          ))}
        </div>
      )}
    </div>
  );
}
