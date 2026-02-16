import { useState, useRef, useEffect, useCallback, type ReactNode } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { AiKeyword, AiKeywordType, InvestigationResult } from "@/types";
import { useTranscriptStore } from "@/stores/transcriptStore";
import { useInsightsStore } from "@/stores/insightsStore";

interface KeywordPopoverProps {
  keyword: AiKeyword;
  children: ReactNode;
}

const typeBadge: Record<AiKeywordType, { label: string; className: string }> = {
  tech_term: { label: "技術用語", className: "bg-blue-100 text-blue-800" },
  proper_noun: { label: "固有名詞", className: "bg-green-100 text-green-800" },
  acronym: { label: "略語", className: "bg-purple-100 text-purple-800" },
  jargon: { label: "専門用語", className: "bg-orange-100 text-orange-800" },
};

export function KeywordPopover({ keyword, children }: KeywordPopoverProps) {
  const [open, setOpen] = useState(false);
  const [investigating, setInvestigating] = useState(false);
  const ref = useRef<HTMLSpanElement>(null);

  useEffect(() => {
    if (!open) return;
    function handleClickOutside(e: MouseEvent) {
      if (ref.current && !ref.current.contains(e.target as Node)) {
        setOpen(false);
      }
    }
    document.addEventListener("mousedown", handleClickOutside);
    return () => {
      document.removeEventListener("mousedown", handleClickOutside);
    };
  }, [open]);

  const handleInvestigate = useCallback(() => {
    if (investigating) return;
    setInvestigating(true);
    const entries = useTranscriptStore.getState().entries;
    const context = entries
      .slice(-10)
      .map((e) => `[${e.speakerName}] ${e.text}`)
      .join("\n");
    void invoke<InvestigationResult>("investigate", { query: keyword.term, context })
      .then((result) => {
        useInsightsStore.getState().addInvestigation(result);
        setOpen(false);
      })
      .catch(() => undefined)
      .finally(() => {
        setInvestigating(false);
      });
  }, [keyword.term, investigating]);

  const badge = typeBadge[keyword.type];

  return (
    <span ref={ref} className="relative inline">
      <span
        role="button"
        tabIndex={0}
        onClick={() => {
          setOpen((prev) => !prev);
        }}
        onKeyDown={(e) => {
          if (e.key === "Enter" || e.key === " ") setOpen((prev) => !prev);
        }}
      >
        {children}
      </span>
      {open && (
        <div className="absolute z-20 bottom-full left-0 mb-1 w-72 p-3 rounded-lg border border-border bg-popover text-popover-foreground shadow-lg text-xs space-y-2">
          <div className="flex items-center gap-2">
            <span className="font-semibold text-sm">{keyword.term}</span>
            <span className={`px-1.5 py-0.5 rounded text-[10px] font-medium ${badge.className}`}>
              {badge.label}
            </span>
          </div>
          {keyword.definition && <p>{keyword.definition}</p>}
          {keyword.webSearchResult && (
            <p className="text-muted-foreground">{keyword.webSearchResult}</p>
          )}
          {keyword.sourceUrl && (
            <a
              href={keyword.sourceUrl}
              target="_blank"
              rel="noopener noreferrer"
              className="text-primary underline block"
            >
              ソースを見る
            </a>
          )}
          <button
            type="button"
            onClick={handleInvestigate}
            disabled={investigating}
            className="w-full mt-1 px-2 py-1.5 rounded bg-primary text-primary-foreground text-xs font-medium hover:opacity-90 transition-opacity disabled:opacity-50"
          >
            {investigating ? "調査中..." : "Claudeで詳しく調査"}
          </button>
        </div>
      )}
    </span>
  );
}
