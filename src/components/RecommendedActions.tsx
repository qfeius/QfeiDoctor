import type { Suggestion } from "../types/diagnostic";
import { QuickActionButton } from "./QuickActionButton";

interface RecommendedActionsProps {
  suggestions: Suggestion[];
}

export function RecommendedActions({ suggestions }: RecommendedActionsProps) {
  if (suggestions.length === 0) return null;

  return (
    <div className="actions">
      <div className="actions__title">Recommended Actions</div>
      {suggestions.map((s, i) => (
        <div className="action-item" key={i}>
          <span className="action-item__cause">{s.cause}</span>
          <span className="action-item__suggestion">{s.action}</span>
          {s.quick_action && (
            <span className="action-item__quick">
              <QuickActionButton uri={s.quick_action} />
            </span>
          )}
        </div>
      ))}
    </div>
  );
}
