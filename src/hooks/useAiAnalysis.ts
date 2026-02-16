import { useEffect, useRef } from "react";
import { useInsightsStore } from "@/stores/insightsStore";
import type { AiKeyword, AiSummary, AiActionItem, Decision, InvestigationResult } from "@/types";

type UnlistenFn = () => void;
interface TauriEvent<T> {
  payload: T;
}
async function invokeCommand(cmd: string, args?: Record<string, unknown>): Promise<void> {
  try {
    const { invoke } = await import("@tauri-apps/api/core");
    await invoke(cmd, args);
  } catch {
    // Tauri API not available (browser dev mode)
  }
}

export async function triggerInvestigation(query: string, context: string): Promise<void> {
  await invokeCommand("investigate", { query, context });
}

const AI_BATCH_INTERVAL_MS = 180_000;

export function useAiAnalysis(sessionId: string | null, isRecording: boolean) {
  const {
    addKeywords,
    updateSummary,
    addActionItems,
    addDecisions,
    addInvestigation,
    setAnalyzing,
    clearAll,
  } = useInsightsStore();

  // Start/stop AI analysis when recording state changes
  useEffect(() => {
    if (!sessionId || !isRecording) return;
    void invokeCommand("start_ai_analysis", { sessionId });
    return () => {
      void invokeCommand("stop_ai_analysis", { sessionId });
    };
  }, [sessionId, isRecording]);

  // 3-minute batch timer
  useEffect(() => {
    if (!sessionId || !isRecording) return;
    const interval = setInterval(() => {
      setAnalyzing(true);
      void invokeCommand("run_ai_batch", { sessionId });
    }, AI_BATCH_INTERVAL_MS);
    return () => {
      clearInterval(interval);
    };
  }, [sessionId, isRecording, setAnalyzing]);

  const mountedRef = useRef(true);
  const unlistenersRef = useRef<UnlistenFn[]>([]);

  useEffect(() => {
    mountedRef.current = true;
    const done = () => {
      setAnalyzing(false);
    };

    async function setupListeners() {
      try {
        const { listen } = await import("@tauri-apps/api/event");
        if (!mountedRef.current) return;

        const fns: UnlistenFn[] = [
          await listen<AiKeyword[]>("ai:keywords", (e: TauriEvent<AiKeyword[]>) => {
            addKeywords(e.payload);
            done();
          }),
          await listen<AiSummary>("ai:summary", (e: TauriEvent<AiSummary>) => {
            updateSummary(e.payload);
            done();
          }),
          await listen<AiActionItem[]>("ai:actions", (e: TauriEvent<AiActionItem[]>) => {
            addActionItems(e.payload);
            done();
          }),
          await listen<Decision[]>("ai:decisions", (e: TauriEvent<Decision[]>) => {
            addDecisions(e.payload);
            done();
          }),
          await listen<InvestigationResult>(
            "ai:investigation",
            (e: TauriEvent<InvestigationResult>) => {
              addInvestigation(e.payload);
              done();
            },
          ),
        ];

        // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition
        if (mountedRef.current) {
          unlistenersRef.current = fns;
        } else {
          for (const fn of fns) fn();
        }
      } catch {
        // Tauri API not available (browser dev mode)
      }
    }

    void setupListeners();
    return () => {
      mountedRef.current = false;
      for (const fn of unlistenersRef.current) fn();
      unlistenersRef.current = [];
    };
  }, [addKeywords, updateSummary, addActionItems, addDecisions, addInvestigation, setAnalyzing]);

  // Clear insights when session changes
  useEffect(() => {
    clearAll();
  }, [sessionId, clearAll]);
}
