import { create } from "zustand";
import type { TranslationEntry } from "@/types";

interface TranslationState {
  translations: TranslationEntry[];
  targetLang: string;
  isTranslating: boolean;
}

interface TranslationActions {
  translateSession: (sessionId: string) => Promise<void>;
  loadTranslations: (sessionId: string) => Promise<void>;
  setTargetLang: (lang: string) => void;
  clearTranslations: () => void;
}

const initialState: TranslationState = {
  translations: [],
  targetLang: "en",
  isTranslating: false,
};

export const useTranslationStore = create<TranslationState & TranslationActions>((set, get) => ({
  ...initialState,

  translateSession: async (sessionId: string) => {
    if (get().isTranslating) return;
    set({ isTranslating: true });
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      const result = await invoke<TranslationEntry[]>("translate_segments", {
        sessionId,
        targetLang: get().targetLang,
      });
      set({ translations: result });
    } catch {
      // Tauri API not available (browser dev mode)
    } finally {
      set({ isTranslating: false });
    }
  },

  loadTranslations: async (sessionId: string) => {
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      const result = await invoke<TranslationEntry[]>("get_translations", {
        sessionId,
        targetLang: get().targetLang,
      });
      set({ translations: result });
    } catch {
      // Tauri API not available (browser dev mode)
    }
  },

  setTargetLang: (lang: string) => {
    set({ targetLang: lang });
  },

  clearTranslations: () => {
    set({ translations: [], isTranslating: false });
  },
}));
