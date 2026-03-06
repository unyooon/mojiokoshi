import { useEffect, useRef } from "react";
import { useInsightsStore } from "@/stores/insightsStore";
import type {
  AiKeyword,
  AiSummary,
  AiActionItem,
  AiTopic,
  Decision,
  FormattedTranscript,
  InvestigationResult,
} from "@/types";

type UnlistenFn = () => void;
interface TauriEvent<T> {
  payload: T;
}

/**
 * @description Tauriコマンドを呼び出すヘルパー。エラー時はログ出力して再throwする。
 * @param cmd - Tauriコマンド名
 * @param args - コマンド引数
 * @throws Tauriコマンドのエラー
 */
async function invokeCommand(cmd: string, args?: Record<string, unknown>): Promise<void> {
  const { invoke } = await import("@tauri-apps/api/core");
  await invoke(cmd, args);
}

/**
 * @description Tauriコマンドを呼び出すヘルパー（エラーを無視）。ブラウザdevモード対応。
 * @param cmd - Tauriコマンド名
 * @param args - コマンド引数
 */
async function invokeCommandSilent(cmd: string, args?: Record<string, unknown>): Promise<void> {
  try {
    await invokeCommand(cmd, args);
  } catch {
    // Tauri API not available (browser dev mode)
  }
}

export async function triggerInvestigation(query: string, context: string): Promise<void> {
  await invokeCommandSilent("investigate", { query, context });
}

/** バッチ分析間隔: 3分 */
const AI_BATCH_INTERVAL_MS = 180_000;
/** トランスクリプトフォーマット間隔: 30秒 */
const FORMAT_INTERVAL_MS = 30_000;

export function useAiAnalysis(sessionId: string | null, isRecording: boolean) {
  const {
    addKeywords,
    updateSummary,
    addActionItems,
    addDecisions,
    addInvestigation,
    addAiTopics,
    setFormattedTranscript,
    setAnalyzing,
    clearAll,
  } = useInsightsStore();

  // Start/stop AI analysis when recording state changes
  useEffect(() => {
    if (!sessionId || !isRecording) return;
    void invokeCommandSilent("start_ai_analysis", { sessionId });
    return () => {
      void invokeCommandSilent("stop_ai_analysis", { sessionId });
    };
  }, [sessionId, isRecording]);

  // 3-minute batch timer
  useEffect(() => {
    if (!sessionId || !isRecording) return;
    const runBatch = async () => {
      setAnalyzing(true);
      try {
        await invokeCommand("run_ai_batch", { sessionId });
      } catch {
        setAnalyzing(false);
      }
    };
    const interval = setInterval(() => {
      void runBatch();
    }, AI_BATCH_INTERVAL_MS);
    return () => {
      clearInterval(interval);
    };
  }, [sessionId, isRecording, setAnalyzing]);

  // 30-second format timer
  useEffect(() => {
    if (!sessionId || !isRecording) return;
    const interval = setInterval(() => {
      void invokeCommandSilent("format_transcript", { sessionId });
    }, FORMAT_INTERVAL_MS);
    return () => {
      clearInterval(interval);
    };
  }, [sessionId, isRecording]);

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
          await listen<AiTopic[]>("ai:topics", (e: TauriEvent<AiTopic[]>) => {
            addAiTopics(e.payload);
            done();
          }),
          await listen<FormattedTranscript>(
            "ai:formatted-transcript",
            (e: TauriEvent<FormattedTranscript>) => {
              setFormattedTranscript(e.payload.formatted_text);
              done();
            },
          ),
          await listen("ai:batch-done", () => {
            done();
          }),
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
  }, [
    addKeywords,
    updateSummary,
    addActionItems,
    addDecisions,
    addInvestigation,
    addAiTopics,
    setFormattedTranscript,
    setAnalyzing,
  ]);

  // Clear insights when session changes
  useEffect(() => {
    clearAll();
  }, [sessionId, clearAll]);
}
