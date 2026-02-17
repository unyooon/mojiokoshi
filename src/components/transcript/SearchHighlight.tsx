import { useMemo } from "react";
import { useSearchStore, type SearchMatch } from "@/stores/searchStore";

interface SearchHighlightProps {
  text: string;
  entryId: string;
}

interface HighlightSegment {
  text: string;
  isCurrent: boolean;
  isMatch: boolean;
}

function buildHighlightSegments(
  text: string,
  entryId: string,
  matches: SearchMatch[],
  currentMatchIndex: number,
): HighlightSegment[] {
  const entryMatches = matches
    .map((m, globalIdx) => ({ ...m, globalIdx }))
    .filter((m) => m.entryId === entryId);

  if (entryMatches.length === 0) {
    return [{ text, isCurrent: false, isMatch: false }];
  }

  const segments: HighlightSegment[] = [];
  let lastIndex = 0;

  for (const match of entryMatches) {
    if (match.startIndex > lastIndex) {
      segments.push({
        text: text.slice(lastIndex, match.startIndex),
        isCurrent: false,
        isMatch: false,
      });
    }
    segments.push({
      text: text.slice(match.startIndex, match.endIndex),
      isCurrent: match.globalIdx === currentMatchIndex,
      isMatch: true,
    });
    lastIndex = match.endIndex;
  }

  if (lastIndex < text.length) {
    segments.push({ text: text.slice(lastIndex), isCurrent: false, isMatch: false });
  }

  return segments;
}

export function SearchHighlight({ text, entryId }: SearchHighlightProps) {
  const matches = useSearchStore((s) => s.matches);
  const currentMatchIndex = useSearchStore((s) => s.currentMatchIndex);

  const segments = useMemo(
    () => buildHighlightSegments(text, entryId, matches, currentMatchIndex),
    [text, entryId, matches, currentMatchIndex],
  );

  return (
    <span>
      {segments.map((seg, i) => {
        if (!seg.isMatch) {
          return <span key={i}>{seg.text}</span>;
        }
        return (
          <mark
            key={i}
            className={
              seg.isCurrent
                ? "bg-orange-400 dark:bg-orange-500 text-foreground rounded-sm px-px"
                : "bg-yellow-200 dark:bg-yellow-700 text-foreground rounded-sm px-px"
            }
            data-search-current={seg.isCurrent || undefined}
          >
            {seg.text}
          </mark>
        );
      })}
    </span>
  );
}
