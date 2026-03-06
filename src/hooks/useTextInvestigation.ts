import { useState, useCallback, useEffect } from "react";
import { useTranscriptStore } from "@/stores/transcriptStore";
import { useInsightsStore } from "@/stores/insightsStore";
import type { InvestigationResult } from "@/types";

/**
 * テキスト選択から取得した調査用データ
 */
interface SelectionData {
  /** 調査対象クエリ（選択テキスト） */
  query: string;
  /** 選択箇所周辺のトランスクリプトコンテキスト */
  context: string;
}

/**
 * 選択テキストをウィンドウから取得し、周辺のトランスクリプトをコンテキストとして付与する
 * @returns 選択データ、または選択がない場合は null
 */
function getSelectionData(): SelectionData | null {
  const sel = window.getSelection();
  if (!sel || sel.isCollapsed) return null;
  const query = sel.toString().trim();
  if (!query) return null;

  const entries = useTranscriptStore.getState().entries;
  const el = sel.anchorNode?.parentElement;
  const idxAttr = el?.closest("[data-index]")?.getAttribute("data-index") ?? "0";
  const idx = parseInt(idxAttr, 10);
  const context = entries
    .slice(Math.max(0, idx - 5), Math.min(entries.length, idx + 6))
    .map((e) => `[${e.speakerName}] ${e.text}`)
    .join("\n");

  return { query, context };
}

/**
 * useTextInvestigation が返す値
 */
interface UseTextInvestigationReturn {
  /** 現在調査中かどうか */
  isInvestigating: boolean;
  /** 選択テキストが存在し調査可能な状態かどうか */
  hasSelection: boolean;
  /** 選択テキストを調査する。選択がなければ何もしない */
  investigate: () => void;
}

/**
 * @description
 * テキスト選択に基づく Claude 調査機能を提供するフック。
 * window.getSelection() で選択テキストを取得し、Tauri の investigate コマンドを
 * 呼び出して結果を insightsStore に追加する。
 *
 * @returns 調査状態と実行関数
 */
export function useTextInvestigation(): UseTextInvestigationReturn {
  const [isInvestigating, setIsInvestigating] = useState(false);
  const [hasSelection, setHasSelection] = useState(false);
  const addInvestigation = useInsightsStore((s) => s.addInvestigation);
  const setAnalyzing = useInsightsStore((s) => s.setAnalyzing);

  // selectionchange イベントで hasSelection を更新
  useEffect(() => {
    const handleSelectionChange = () => {
      const sel = window.getSelection();
      setHasSelection(!!sel && !sel.isCollapsed && sel.toString().trim().length > 0);
    };
    document.addEventListener("selectionchange", handleSelectionChange);
    return () => {
      document.removeEventListener("selectionchange", handleSelectionChange);
    };
  }, []);

  /**
   * 現在の選択テキストを調査する。
   * Tauri の investigate コマンドを呼び、結果を insightsStore に追加する。
   */
  const investigate = useCallback(() => {
    if (isInvestigating) return;
    const data = getSelectionData();
    if (!data) return;

    const { query, context } = data;

    setIsInvestigating(true);
    setAnalyzing(true);

    async function run() {
      try {
        const { invoke } = await import("@tauri-apps/api/core");
        const result = await invoke<InvestigationResult>("investigate", { query, context });
        addInvestigation(result);
      } catch {
        // Tauri API not available (browser dev mode) or invoke error
      } finally {
        setIsInvestigating(false);
        setAnalyzing(false);
      }
    }

    void run();
  }, [isInvestigating, addInvestigation, setAnalyzing]);

  return { isInvestigating, hasSelection, investigate };
}
