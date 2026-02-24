import { useMemo } from "react";
import { useInsightsStore } from "@/stores/insightsStore";
import { FeatureStatusBanner } from "./FeatureStatusBanner";
import type { Topic } from "@/types";

function formatTime(timestamp: number): string {
  const date = new Date(timestamp);
  return date.toLocaleTimeString("ja-JP", { hour: "2-digit", minute: "2-digit" });
}

function TopicEntry({ topic }: { topic: Topic }) {
  return (
    <div className="flex items-start gap-3">
      <div className="flex flex-col items-center">
        <span className="text-xs font-mono text-muted-foreground whitespace-nowrap">
          {formatTime(topic.timestamp)}
        </span>
        <div className="w-px flex-1 bg-border mt-1" />
      </div>
      <div className="pb-6 min-w-0">
        <p className="text-sm font-medium text-foreground">{topic.title}</p>
        {topic.keywords.length > 0 && (
          <div className="flex flex-wrap gap-1 mt-1.5">
            {topic.keywords.map((kw) => (
              <span
                key={kw}
                className="inline-block rounded-full bg-secondary px-2 py-0.5 text-xs text-secondary-foreground"
              >
                {kw}
              </span>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

export function TopicTimeline() {
  const topics = useInsightsStore((s) => s.topics);

  const sorted = useMemo(() => [...topics].sort((a, b) => a.timestamp - b.timestamp), [topics]);

  if (sorted.length === 0) {
    return (
      <div>
        <FeatureStatusBanner status="stub" />
        <div className="flex items-center justify-center py-12">
          <p className="text-sm text-muted-foreground">トピックは分析完了後に表示されます</p>
        </div>
      </div>
    );
  }

  return (
    <div className="p-4">
      {sorted.map((topic) => (
        <TopicEntry key={topic.id} topic={topic} />
      ))}
    </div>
  );
}
