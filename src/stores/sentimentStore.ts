import { create } from "zustand";
import type { SentimentEntry } from "@/types";

interface SentimentState {
  sentiments: SentimentEntry[];
  isAnalyzing: boolean;
}

interface SentimentActions {
  analyzeSentiment: (sessionId: string) => Promise<void>;
  loadSentiments: (sessionId: string) => Promise<void>;
  clearSentiments: () => void;
}

const initialState: SentimentState = {
  sentiments: [],
  isAnalyzing: false,
};

export const useSentimentStore = create<SentimentState & SentimentActions>((set) => ({
  ...initialState,

  analyzeSentiment: async (sessionId: string) => {
    set({ isAnalyzing: true });
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      const result = await invoke<SentimentEntry[]>("analyze_sentiment", { sessionId });
      set({ sentiments: result });
    } catch {
      // Tauri API not available (browser dev mode)
    } finally {
      set({ isAnalyzing: false });
    }
  },

  loadSentiments: async (sessionId: string) => {
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      const result = await invoke<SentimentEntry[]>("get_sentiments", { sessionId });
      set({ sentiments: result });
    } catch {
      // Tauri API not available (browser dev mode)
    }
  },

  clearSentiments: () => {
    set(initialState);
  },
}));
