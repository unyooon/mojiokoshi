import { useRef, useEffect } from "react";
import { useVirtualizer } from "@tanstack/react-virtual";
import { useTranscriptStore } from "@/stores/transcriptStore";
import { useInsightsStore } from "@/stores/insightsStore";
import { SegmentLine } from "./SegmentLine";
import { TranscriptContextMenu } from "./TranscriptContextMenu";

export function TranscriptPanel() {
  const entries = useTranscriptStore((s) => s.entries);
  const partialEntry = useTranscriptStore((s) => s.partialEntry);
  const autoScroll = useTranscriptStore((s) => s.autoScroll);
  const keywords = useInsightsStore((s) => s.keywords);

  const allEntries = partialEntry ? [...entries, partialEntry] : entries;
  const parentRef = useRef<HTMLDivElement>(null);

  const virtualizer = useVirtualizer({
    count: allEntries.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => 60,
    overscan: 5,
  });

  useEffect(() => {
    if (autoScroll && allEntries.length > 0) {
      virtualizer.scrollToIndex(allEntries.length - 1, { align: "end" });
    }
  }, [allEntries.length, autoScroll, virtualizer]);

  return (
    <TranscriptContextMenu>
      <div className="flex flex-col h-full">
        <div className="px-4 py-2 border-b border-border">
          <h2 className="text-sm font-semibold">Transcript</h2>
        </div>
        <div ref={parentRef} className="flex-1 overflow-auto">
          <div
            style={{
              height: `${virtualizer.getTotalSize()}px`,
              width: "100%",
              position: "relative",
            }}
          >
            {virtualizer.getVirtualItems().map((virtualRow) => {
              const entry = allEntries[virtualRow.index];
              return (
                <div
                  key={virtualRow.key}
                  style={{
                    position: "absolute",
                    top: 0,
                    left: 0,
                    width: "100%",
                    transform: `translateY(${virtualRow.start}px)`,
                  }}
                  data-index={virtualRow.index}
                  ref={virtualizer.measureElement}
                >
                  <SegmentLine entry={entry} keywords={keywords} />
                </div>
              );
            })}
          </div>
          {allEntries.length === 0 && (
            <div className="flex items-center justify-center h-full text-muted-foreground text-sm">
              Start recording to see transcription
            </div>
          )}
        </div>
      </div>
    </TranscriptContextMenu>
  );
}
