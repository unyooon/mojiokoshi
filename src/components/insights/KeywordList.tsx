import { useState } from "react";
import { useInsightsStore } from "@/stores/insightsStore";
import { FeatureStatusBanner } from "./FeatureStatusBanner";
import type { AiKeywordType } from "@/types";

const typeColors: Record<AiKeywordType, string> = {
  tech_term: "bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200",
  proper_noun: "bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200",
  acronym: "bg-purple-100 text-purple-800 dark:bg-purple-900 dark:text-purple-200",
  jargon: "bg-orange-100 text-orange-800 dark:bg-orange-900 dark:text-orange-200",
};

export function KeywordList() {
  const keywords = useInsightsStore((s) => s.keywords);
  const [expandedId, setExpandedId] = useState<string | null>(null);

  if (keywords.length === 0) {
    return (
      <div>
        <FeatureStatusBanner status="stub" />
        <div className="flex items-center justify-center py-12 text-muted-foreground text-sm">
          キーワード未検出
        </div>
      </div>
    );
  }

  return (
    <div className="p-4">
      <div className="flex flex-wrap gap-2">
        {keywords.map((kw) => {
          const isExpanded = expandedId === kw.id;
          return (
            <div key={kw.id} className="relative">
              <button
                type="button"
                onClick={() => {
                  setExpandedId(isExpanded ? null : kw.id);
                }}
                className={`inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-xs font-medium transition-colors ${typeColors[kw.type]}`}
              >
                {kw.term}
                <span className="inline-flex items-center justify-center h-4 min-w-4 px-1 rounded-full bg-black/10 text-[10px] leading-none">
                  {kw.occurrences}
                </span>
              </button>
              {isExpanded && (kw.definition ?? kw.webSearchResult) && (
                <div className="absolute z-10 top-full left-0 mt-1 w-64 p-3 rounded-lg border border-border bg-popover text-popover-foreground shadow-lg text-xs space-y-2">
                  {kw.definition && <p>{kw.definition}</p>}
                  {kw.webSearchResult && (
                    <p className="text-muted-foreground">{kw.webSearchResult}</p>
                  )}
                  {kw.sourceUrl && (
                    <a
                      href={kw.sourceUrl}
                      target="_blank"
                      rel="noopener noreferrer"
                      className="text-primary underline"
                    >
                      Source
                    </a>
                  )}
                </div>
              )}
            </div>
          );
        })}
      </div>
    </div>
  );
}
