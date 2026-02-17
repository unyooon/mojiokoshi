import { create } from "zustand";

export interface SearchMatch {
  entryId: string;
  startIndex: number;
  endIndex: number;
}

interface SearchState {
  query: string;
  isOpen: boolean;
  matches: SearchMatch[];
  currentMatchIndex: number;
}

interface SearchActions {
  setQuery: (query: string) => void;
  open: () => void;
  close: () => void;
  setMatches: (matches: SearchMatch[]) => void;
  nextMatch: () => void;
  prevMatch: () => void;
}

export const useSearchStore = create<SearchState & SearchActions>((set) => ({
  query: "",
  isOpen: false,
  matches: [],
  currentMatchIndex: 0,

  setQuery: (query) => {
    set({ query, currentMatchIndex: 0 });
  },

  open: () => {
    set({ isOpen: true });
  },

  close: () => {
    set({ isOpen: false, query: "", matches: [], currentMatchIndex: 0 });
  },

  setMatches: (matches) => {
    set((state) => ({
      matches,
      currentMatchIndex:
        matches.length > 0 ? Math.min(state.currentMatchIndex, matches.length - 1) : 0,
    }));
  },

  nextMatch: () => {
    set((state) => {
      if (state.matches.length === 0) return state;
      return { currentMatchIndex: (state.currentMatchIndex + 1) % state.matches.length };
    });
  },

  prevMatch: () => {
    set((state) => {
      if (state.matches.length === 0) return state;
      return {
        currentMatchIndex:
          (state.currentMatchIndex - 1 + state.matches.length) % state.matches.length,
      };
    });
  },
}));
