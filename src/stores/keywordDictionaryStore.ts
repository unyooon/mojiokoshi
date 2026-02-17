import { create } from "zustand";
import type { DictionaryKeyword } from "@/types";

interface KeywordDictionaryState {
  keywords: DictionaryKeyword[];
  isLoading: boolean;
}

interface KeywordDictionaryActions {
  loadKeywords: () => Promise<void>;
  addKeyword: (
    term: string,
    reading: string | null,
    definition: string | null,
    category: string,
  ) => Promise<void>;
  updateKeyword: (
    id: number,
    term: string,
    reading: string | null,
    definition: string | null,
    category: string,
  ) => Promise<void>;
  deleteKeyword: (id: number) => Promise<void>;
}

const initialState: KeywordDictionaryState = {
  keywords: [],
  isLoading: false,
};

export const useKeywordDictionaryStore = create<KeywordDictionaryState & KeywordDictionaryActions>(
  (set) => ({
    ...initialState,

    loadKeywords: async () => {
      set({ isLoading: true });
      try {
        const { invoke } = await import("@tauri-apps/api/core");
        const result = await invoke<DictionaryKeyword[]>("get_all_dictionary_keywords");
        set({ keywords: result });
      } catch {
        // Tauri API not available (browser dev mode)
      } finally {
        set({ isLoading: false });
      }
    },

    addKeyword: async (
      term: string,
      reading: string | null,
      definition: string | null,
      category: string,
    ) => {
      try {
        const { invoke } = await import("@tauri-apps/api/core");
        await invoke<number>("add_dictionary_keyword", {
          term,
          reading,
          definition,
          category,
        });
        const result = await invoke<DictionaryKeyword[]>("get_all_dictionary_keywords");
        set({ keywords: result });
      } catch {
        // Tauri API not available (browser dev mode)
      }
    },

    updateKeyword: async (
      id: number,
      term: string,
      reading: string | null,
      definition: string | null,
      category: string,
    ) => {
      try {
        const { invoke } = await import("@tauri-apps/api/core");
        await invoke("update_dictionary_keyword", {
          id,
          term,
          reading,
          definition,
          category,
        });
        const result = await invoke<DictionaryKeyword[]>("get_all_dictionary_keywords");
        set({ keywords: result });
      } catch {
        // Tauri API not available (browser dev mode)
      }
    },

    deleteKeyword: async (id: number) => {
      try {
        const { invoke } = await import("@tauri-apps/api/core");
        await invoke("delete_dictionary_keyword", { id });
        const result = await invoke<DictionaryKeyword[]>("get_all_dictionary_keywords");
        set({ keywords: result });
      } catch {
        // Tauri API not available (browser dev mode)
      }
    },
  }),
);
