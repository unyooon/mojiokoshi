import { useState, useRef, useEffect, useCallback, useMemo } from "react";
import { useVirtualizer } from "@tanstack/react-virtual";
import { useTranscriptStore } from "@/stores/transcriptStore";
import { useInsightsStore } from "@/stores/insightsStore";
import { useSpeakerStore } from "@/stores/speakerStore";
import { useSearchStore, type SearchMatch } from "@/stores/searchStore";
import { useTextInvestigation } from "@/hooks/useTextInvestigation";
import { SegmentLine } from "./SegmentLine";
import { SearchBar } from "./SearchBar";
import { TranscriptContextMenu } from "./TranscriptContextMenu";
import { FormattedView } from "./FormattedView";

/** トランスクリプト表示モード */
type ViewMode = "timeline" | "minutes";

function buildSearchMatches(entries: { id: string; text: string }[], query: string): SearchMatch[] {
  if (!query) return [];
  const matches: SearchMatch[] = [];
  const lowerQuery = query.toLowerCase();
  for (const entry of entries) {
    const lowerText = entry.text.toLowerCase();
    let pos = 0;
    while (pos < lowerText.length) {
      const idx = lowerText.indexOf(lowerQuery, pos);
      if (idx === -1) break;
      matches.push({ entryId: entry.id, startIndex: idx, endIndex: idx + query.length });
      pos = idx + 1;
    }
  }
  return matches;
}

export function TranscriptPanel() {
  const [viewMode, setViewMode] = useState<ViewMode>("timeline");
  const { investigate, isInvestigating } = useTextInvestigation();
  const entries = useTranscriptStore((s) => s.entries);
  const partialEntry = useTranscriptStore((s) => s.partialEntry);
  const autoScroll = useTranscriptStore((s) => s.autoScroll);
  const keywords = useInsightsStore((s) => s.keywords);
  const speakers = useSpeakerStore((s) => s.speakers);

  const searchIsOpen = useSearchStore((s) => s.isOpen);
  const searchQuery = useSearchStore((s) => s.query);
  const currentMatchIndex = useSearchStore((s) => s.currentMatchIndex);
  const searchOpen = useSearchStore((s) => s.open);
  const setMatches = useSearchStore((s) => s.setMatches);

  const allEntries = useMemo(
    () => (partialEntry ? [...entries, partialEntry] : entries),
    [entries, partialEntry],
  );
  const parentRef = useRef<HTMLDivElement>(null);

  const virtualizer = useVirtualizer({
    count: allEntries.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => 60,
    overscan: 5,
  });

  // Compute and sync search matches whenever query or entries change
  const matches = useMemo(
    () => buildSearchMatches(allEntries, searchQuery),
    [allEntries, searchQuery],
  );

  useEffect(() => {
    setMatches(matches);
  }, [matches, setMatches]);

  // Auto-scroll to current match
  const currentMatch = matches[currentMatchIndex] as SearchMatch | undefined;
  useEffect(() => {
    if (!currentMatch) return;
    const entryIndex = allEntries.findIndex((e) => e.id === currentMatch.entryId);
    if (entryIndex >= 0) {
      virtualizer.scrollToIndex(entryIndex, { align: "center" });
    }
  }, [currentMatch, allEntries, virtualizer]);

  // Auto-scroll for new entries
  useEffect(() => {
    if (autoScroll && allEntries.length > 0 && !searchIsOpen) {
      virtualizer.scrollToIndex(allEntries.length - 1, { align: "end" });
    }
  }, [allEntries.length, autoScroll, searchIsOpen, virtualizer]);

  // Cmd+F keyboard shortcut
  const handleKeyDown = useCallback(
    (e: KeyboardEvent) => {
      if (e.key === "f" && e.metaKey && !e.shiftKey) {
        e.preventDefault();
        searchOpen();
      }
    },
    [searchOpen],
  );

  useEffect(() => {
    document.addEventListener("keydown", handleKeyDown);
    return () => {
      document.removeEventListener("keydown", handleKeyDown);
    };
  }, [handleKeyDown]);

  return (
    <TranscriptContextMenu onInvestigate={investigate} isInvestigating={isInvestigating}>
      <div className="flex flex-col h-full">
        <div className="px-4 py-2 border-b border-border flex items-center justify-between">
          <h2 className="text-sm font-semibold">Transcript</h2>
          <div className="flex items-center gap-0.5 bg-muted rounded-md p-0.5">
            <button
              type="button"
              onClick={() => {
                setViewMode("timeline");
              }}
              className={`px-2 py-0.5 text-xs font-medium rounded transition-colors ${
                viewMode === "timeline"
                  ? "bg-background text-foreground shadow-sm"
                  : "text-muted-foreground hover:text-foreground"
              }`}
            >
              タイムライン
            </button>
            <button
              type="button"
              onClick={() => {
                setViewMode("minutes");
              }}
              className={`px-2 py-0.5 text-xs font-medium rounded transition-colors ${
                viewMode === "minutes"
                  ? "bg-background text-foreground shadow-sm"
                  : "text-muted-foreground hover:text-foreground"
              }`}
            >
              議事録
            </button>
          </div>
        </div>
        {viewMode === "minutes" ? (
          <div className="flex-1 overflow-hidden">
            <FormattedView />
          </div>
        ) : (
          <div className="relative flex-1 overflow-hidden">
            {searchIsOpen && <SearchBar />}
            <div ref={parentRef} className="h-full overflow-auto">
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
                      <SegmentLine
                        entry={entry}
                        keywords={keywords}
                        speaker={entry.speakerId ? speakers.get(entry.speakerId) : undefined}
                        searchActive={searchIsOpen && searchQuery.length > 0}
                      />
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
        )}
      </div>
    </TranscriptContextMenu>
  );
}
