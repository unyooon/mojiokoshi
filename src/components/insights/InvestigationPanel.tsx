import { useInsightsStore } from "@/stores/insightsStore";
import { FeatureStatusBanner } from "./FeatureStatusBanner";
import type { InvestigationResult, Source } from "@/types";

function formatTime(ms: number): string {
  return new Date(ms).toLocaleTimeString("ja-JP", {
    hour: "2-digit",
    minute: "2-digit",
  });
}

function SourceLink({ source }: { source: Source }) {
  return (
    <a
      href={source.url}
      target="_blank"
      rel="noopener noreferrer"
      className="text-primary underline text-xs block truncate hover:opacity-80"
    >
      {source.title}
    </a>
  );
}

function InvestigationCard({ result }: { result: InvestigationResult }) {
  return (
    <div className="p-3 border-b border-border space-y-2">
      <div className="flex items-start justify-between gap-2">
        <p className="text-sm font-bold leading-snug">{result.query}</p>
        <span className="shrink-0 text-[10px] text-muted-foreground">
          {formatTime(result.createdAt)}
        </span>
      </div>
      <p className="text-sm leading-relaxed">{result.summary}</p>
      {result.details && (
        <p className="text-xs text-muted-foreground leading-relaxed whitespace-pre-wrap">
          {result.details}
        </p>
      )}
      {result.sources.length > 0 && (
        <div className="space-y-0.5 pt-1">
          <p className="text-[10px] font-medium text-muted-foreground uppercase tracking-wide">
            Sources
          </p>
          {result.sources.map((source, i) => (
            <SourceLink key={i} source={source} />
          ))}
        </div>
      )}
    </div>
  );
}

export function InvestigationPanel() {
  const investigations = useInsightsStore((s) => s.investigations);
  const isAnalyzing = useInsightsStore((s) => s.isAnalyzing);

  if (isAnalyzing && investigations.length === 0) {
    return (
      <div className="flex items-center gap-2 justify-center py-12 text-muted-foreground text-sm">
        <span className="inline-block h-4 w-4 animate-spin rounded-full border-2 border-current border-t-transparent" />
        調査中...
      </div>
    );
  }

  if (investigations.length === 0) {
    return (
      <div>
        <FeatureStatusBanner status="stub" />
        <div className="flex items-center justify-center py-12 text-muted-foreground text-sm px-4 text-center">
          テキストを選択して Cmd+I で調査を開始
        </div>
      </div>
    );
  }

  const sorted = [...investigations].sort((a, b) => b.createdAt - a.createdAt);

  return (
    <div className="flex flex-col">
      {sorted.map((result) => (
        <InvestigationCard key={result.id} result={result} />
      ))}
    </div>
  );
}
