import { create } from "zustand";
import type { TranscriptEntry } from "@/types";

interface TranscriptState {
  entries: TranscriptEntry[];
  partialEntry: TranscriptEntry | null;
  autoScroll: boolean;

  addEntry: (entry: TranscriptEntry) => void;
  updatePartial: (entry: TranscriptEntry | null) => void;
  clearEntries: () => void;
  setAutoScroll: (enabled: boolean) => void;
}

export const useTranscriptStore = create<TranscriptState>((set) => ({
  entries: [],
  partialEntry: null,
  autoScroll: true,

  addEntry: (entry) => {
    set((state) => ({
      entries: [...state.entries, entry],
      partialEntry: null,
    }));
  },

  updatePartial: (entry) => {
    set({ partialEntry: entry });
  },

  clearEntries: () => {
    set({ entries: [], partialEntry: null });
  },

  setAutoScroll: (enabled) => {
    set({ autoScroll: enabled });
  },
}));
