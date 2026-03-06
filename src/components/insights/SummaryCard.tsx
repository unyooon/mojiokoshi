import { useInsightsStore } from "@/stores/insightsStore";

function formatTime(ms: number): string {
  return new Date(ms).toLocaleTimeString("ja-JP", {
    hour: "2-digit",
    minute: "2-digit",
  });
}

export function SummaryCard() {
  const summary = useInsightsStore((s) => s.summary);
  const isAnalyzing = useInsightsStore((s) => s.isAnalyzing);

  if (isAnalyzing && !summary) {
    return (
      <div className="space-y-3 p-4 animate-pulse">
        <div className="h-4 bg-muted rounded w-3/4" />
        <div className="h-4 bg-muted rounded w-full" />
        <div className="h-4 bg-muted rounded w-5/6" />
        <div className="h-3 bg-muted rounded w-1/3 mt-4" />
      </div>
    );
  }

  if (!summary) {
    return (
      <div className="flex items-center justify-center py-12 text-muted-foreground text-sm">
        AI分析待ち...
      </div>
    );
  }

  return (
    <div className="p-4 space-y-3">
      <p className="text-sm leading-relaxed whitespace-pre-wrap">{summary.text}</p>
      <p className="text-xs text-muted-foreground">最終更新: {formatTime(summary.updatedAt)}</p>
    </div>
  );
}
