import { create } from "zustand";
import type {
  AiKeyword,
  AiSummary,
  AiActionItem,
  Decision,
  InvestigationResult,
  Topic,
} from "@/types";

interface InsightsState {
  keywords: AiKeyword[];
  summary: AiSummary | null;
  actionItems: AiActionItem[];
  decisions: Decision[];
  investigations: InvestigationResult[];
  topics: Topic[];
  isAnalyzing: boolean;
}

interface InsightsActions {
  addKeywords: (keywords: AiKeyword[]) => void;
  updateSummary: (summary: AiSummary) => void;
  addActionItems: (items: AiActionItem[]) => void;
  addDecisions: (decisions: Decision[]) => void;
  addInvestigation: (result: InvestigationResult) => void;
  addTopics: (topics: Topic[]) => void;
  toggleActionItemComplete: (id: string) => void;
  setAnalyzing: (analyzing: boolean) => void;
  clearAll: () => void;
}

const initialState: InsightsState = {
  keywords: [],
  summary: null,
  actionItems: [],
  decisions: [],
  investigations: [],
  topics: [],
  isAnalyzing: false,
};

export const useInsightsStore = create<InsightsState & InsightsActions>((set) => ({
  ...initialState,

  addKeywords: (newKeywords) => {
    set((state) => {
      const existing = new Map(state.keywords.map((k) => [k.term, k]));
      for (const kw of newKeywords) {
        const prev = existing.get(kw.term);
        if (prev) {
          existing.set(kw.term, {
            ...prev,
            occurrences: prev.occurrences + kw.occurrences,
          });
        } else {
          existing.set(kw.term, kw);
        }
      }
      return { keywords: Array.from(existing.values()) };
    });
  },

  updateSummary: (summary) => {
    set({ summary });
  },

  addActionItems: (items) => {
    set((state) => ({
      actionItems: [
        ...state.actionItems,
        ...items.filter((item) => !state.actionItems.some((existing) => existing.id === item.id)),
      ],
    }));
  },

  addDecisions: (newDecisions) => {
    set((state) => ({
      decisions: [
        ...state.decisions,
        ...newDecisions.filter((d) => !state.decisions.some((existing) => existing.id === d.id)),
      ],
    }));
  },

  addInvestigation: (result) => {
    set((state) => ({
      investigations: [...state.investigations, result],
    }));
  },

  addTopics: (newTopics) => {
    set((state) => ({
      topics: [
        ...state.topics,
        ...newTopics.filter((t) => !state.topics.some((existing) => existing.id === t.id)),
      ],
    }));
  },

  toggleActionItemComplete: (id) => {
    set((state) => ({
      actionItems: state.actionItems.map((item) =>
        item.id === id ? { ...item, completed: !item.completed } : item,
      ),
    }));
  },

  setAnalyzing: (isAnalyzing) => {
    set({ isAnalyzing });
  },

  clearAll: () => {
    set(initialState);
  },
}));
