import { useInsightsStore } from "@/stores/insightsStore";

function formatTime(ms: number): string {
  return new Date(ms).toLocaleTimeString("ja-JP", {
    hour: "2-digit",
    minute: "2-digit",
  });
}

export function DecisionList() {
  const decisions = useInsightsStore((s) => s.decisions);

  if (decisions.length === 0) {
    return (
      <div className="flex items-center justify-center py-12 text-muted-foreground text-sm">
        決定事項はまだありません
      </div>
    );
  }

  return (
    <ul className="divide-y divide-border">
      {decisions.map((d) => (
        <li key={d.id} className="px-4 py-3 space-y-1">
          <p className="text-sm font-medium">{d.text}</p>
          <p className="text-xs text-muted-foreground">{d.context}</p>
          <p className="text-xs text-muted-foreground">{formatTime(d.detectedAt)}</p>
        </li>
      ))}
    </ul>
  );
}
