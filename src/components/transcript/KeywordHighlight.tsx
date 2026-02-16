import { useMemo } from "react";
import type { AiKeyword, AiKeywordType } from "@/types";
import { KeywordPopover } from "./KeywordPopover";

interface KeywordHighlightProps {
  text: string;
  keywords: AiKeyword[];
}

const typeColors: Record<AiKeywordType, string> = {
  tech_term: "bg-blue-100 text-blue-800 dark:bg-blue-900/40 dark:text-blue-200",
  proper_noun: "bg-green-100 text-green-800 dark:bg-green-900/40 dark:text-green-200",
  acronym: "bg-purple-100 text-purple-800 dark:bg-purple-900/40 dark:text-purple-200",
  jargon: "bg-orange-100 text-orange-800 dark:bg-orange-900/40 dark:text-orange-200",
};

interface TextSegment {
  text: string;
  keyword: AiKeyword | null;
}

function buildSegments(text: string, keywords: AiKeyword[]): TextSegment[] {
  if (keywords.length === 0) return [{ text, keyword: null }];

  const terms = keywords
    .map((kw) => ({ pattern: kw.term.replace(/[.*+?^${}()|[\]\\]/g, "\\$&"), keyword: kw }))
    .sort((a, b) => b.pattern.length - a.pattern.length);

  const regex = new RegExp(`(${terms.map((t) => t.pattern).join("|")})`, "gi");
  const segments: TextSegment[] = [];
  let lastIndex = 0;

  for (const match of text.matchAll(regex)) {
    const matchIndex = match.index;
    if (matchIndex > lastIndex) {
      segments.push({ text: text.slice(lastIndex, matchIndex), keyword: null });
    }
    const matched = match[0];
    const found = terms.find((t) => t.keyword.term.toLowerCase() === matched.toLowerCase());
    segments.push({ text: matched, keyword: found?.keyword ?? null });
    lastIndex = matchIndex + matched.length;
  }

  if (lastIndex < text.length) {
    segments.push({ text: text.slice(lastIndex), keyword: null });
  }

  return segments;
}

export function KeywordHighlight({ text, keywords }: KeywordHighlightProps) {
  const segments = useMemo(() => buildSegments(text, keywords), [text, keywords]);

  return (
    <span>
      {segments.map((seg, i) => {
        if (!seg.keyword) {
          return <span key={i}>{seg.text}</span>;
        }
        return (
          <KeywordPopover key={i} keyword={seg.keyword}>
            <span className={`rounded px-0.5 cursor-pointer ${typeColors[seg.keyword.type]}`}>
              {seg.text}
            </span>
          </KeywordPopover>
        );
      })}
    </span>
  );
}
