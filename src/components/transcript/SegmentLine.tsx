import type { TranscriptEntry, AiKeyword } from "@/types";
import { KeywordHighlight } from "./KeywordHighlight";

interface SegmentLineProps {
  entry: TranscriptEntry;
  keywords: AiKeyword[];
}

function formatTime(timestamp: number): string {
  const date = new Date(timestamp);
  return date.toLocaleTimeString("ja-JP", {
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
  });
}

export function SegmentLine({ entry, keywords }: SegmentLineProps) {
  return (
    <div className={`flex gap-3 px-4 py-2 ${entry.isPartial ? "opacity-60" : ""}`}>
      <span className="shrink-0 text-xs text-muted-foreground font-mono tabular-nums mt-0.5">
        {formatTime(entry.timestamp)}
      </span>
      <div className="flex-1 min-w-0">
        <span className="text-xs font-medium mr-2" style={{ color: entry.speakerColor.text }}>
          {entry.speakerName}
        </span>
        <span className="text-sm">
          {keywords.length > 0 ? (
            <KeywordHighlight text={entry.text} keywords={keywords} />
          ) : (
            entry.text
          )}
        </span>
      </div>
    </div>
  );
}
