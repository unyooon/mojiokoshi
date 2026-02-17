import type { TranscriptEntry, AiKeyword, Speaker } from "@/types";
import { SPEAKER_COLOR_CLASSES } from "@/types";
import { KeywordHighlight } from "./KeywordHighlight";
import { SearchHighlight } from "./SearchHighlight";

interface SegmentLineProps {
  entry: TranscriptEntry;
  keywords: AiKeyword[];
  speaker?: Speaker;
  searchActive?: boolean;
}

function formatTime(timestamp: number): string {
  const date = new Date(timestamp);
  return date.toLocaleTimeString("ja-JP", {
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
  });
}

export function SegmentLine({ entry, keywords, speaker, searchActive }: SegmentLineProps) {
  const renderText = () => {
    if (searchActive) {
      return <SearchHighlight text={entry.text} entryId={entry.id} />;
    }
    if (keywords.length > 0) {
      return <KeywordHighlight text={entry.text} keywords={keywords} />;
    }
    return entry.text;
  };

  return (
    <div className={`flex gap-3 px-4 py-2 ${entry.isPartial ? "opacity-60" : ""}`}>
      <span className="shrink-0 text-xs text-muted-foreground font-mono tabular-nums mt-0.5">
        {formatTime(entry.timestamp)}
      </span>
      <div className="flex-1 min-w-0">
        {speaker ? (
          <span className="text-xs font-medium mr-2">
            <span
              className={
                SPEAKER_COLOR_CLASSES[speaker.color] + " inline-block w-2 h-2 rounded-full mr-1"
              }
            />
            {speaker.label}
          </span>
        ) : (
          <span className="text-xs font-medium mr-2" style={{ color: entry.speakerColor.text }}>
            {entry.speakerName}
          </span>
        )}
        <span className="text-sm">{renderText()}</span>
      </div>
    </div>
  );
}
