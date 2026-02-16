import { useState, useCallback, useEffect, type ReactNode } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useTranscriptStore } from "@/stores/transcriptStore";
import { useInsightsStore } from "@/stores/insightsStore";
import type { InvestigationResult } from "@/types";

function getSelectionData(): { query: string; context: string } | null {
  const sel = window.getSelection();
  if (!sel || sel.isCollapsed) return null;
  const query = sel.toString().trim();
  if (!query) return null;
  const entries = useTranscriptStore.getState().entries;
  const el = sel.anchorNode?.parentElement;
  const idx = parseInt(el?.closest("[data-index]")?.getAttribute("data-index") ?? "0", 10);
  const context = entries
    .slice(Math.max(0, idx - 5), Math.min(entries.length, idx + 6))
    .map((e) => `[${e.speakerName}] ${e.text}`)
    .join("\n");
  return { query, context };
}

function triggerInvestigation() {
  const data = getSelectionData();
  if (!data) return;
  void invoke<InvestigationResult>("investigate", data).then((r) => {
    useInsightsStore.getState().addInvestigation(r);
  });
}

export function TranscriptContextMenu({ children }: { children: ReactNode }) {
  const [menu, setMenu] = useState<{ x: number; y: number } | null>(null);

  const onCtxMenu = useCallback((e: React.MouseEvent) => {
    const sel = window.getSelection();
    if (sel && !sel.isCollapsed) {
      e.preventDefault();
      setMenu({ x: e.clientX, y: e.clientY });
    }
  }, []);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.metaKey && e.key === "i") {
        e.preventDefault();
        triggerInvestigation();
      }
    };
    const onClick = () => {
      setMenu(null);
    };
    document.addEventListener("keydown", onKey);
    document.addEventListener("click", onClick);
    return () => {
      document.removeEventListener("keydown", onKey);
      document.removeEventListener("click", onClick);
    };
  }, []);

  return (
    <div onContextMenu={onCtxMenu} className="contents">
      {children}
      {menu && (
        <div
          className="fixed z-50 min-w-[180px] py-1 rounded-md border border-border bg-popover text-popover-foreground shadow-md"
          style={{ left: menu.x, top: menu.y }}
        >
          <button
            type="button"
            onClick={() => {
              setMenu(null);
              triggerInvestigation();
            }}
            className="w-full px-3 py-1.5 text-left text-sm hover:bg-accent transition-colors"
          >
            Claudeで調査させる
          </button>
        </div>
      )}
    </div>
  );
}
