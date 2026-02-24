import { useInsightsStore } from "@/stores/insightsStore";
import { FeatureStatusBanner } from "./FeatureStatusBanner";
import type { AiPriority } from "@/types";

const priorityColors: Record<AiPriority, string> = {
  high: "bg-red-500",
  medium: "bg-yellow-500",
  low: "bg-gray-400",
};

export function ActionItemList() {
  const actionItems = useInsightsStore((s) => s.actionItems);
  const toggleComplete = useInsightsStore((s) => s.toggleActionItemComplete);

  if (actionItems.length === 0) {
    return (
      <div>
        <FeatureStatusBanner status="stub" />
        <div className="flex items-center justify-center py-12 text-muted-foreground text-sm">
          アクションアイテムはまだありません
        </div>
      </div>
    );
  }

  return (
    <ul className="divide-y divide-border">
      {actionItems.map((item) => (
        <li key={item.id} className="flex items-start gap-3 px-4 py-3">
          <input
            type="checkbox"
            checked={item.completed}
            onChange={() => {
              toggleComplete(item.id);
            }}
            className="mt-0.5 h-4 w-4 rounded border-input accent-primary shrink-0"
          />
          <div className="flex-1 min-w-0">
            <p
              className={`text-sm leading-relaxed ${item.completed ? "line-through text-muted-foreground" : ""}`}
            >
              {item.text}
            </p>
            <div className="flex items-center gap-2 mt-1">
              <span
                className={`inline-block h-2 w-2 rounded-full ${priorityColors[item.priority]}`}
                title={item.priority}
              />
              {item.assignee && (
                <span className="text-xs bg-secondary text-secondary-foreground px-1.5 py-0.5 rounded">
                  {item.assignee}
                </span>
              )}
            </div>
          </div>
        </li>
      ))}
    </ul>
  );
}
