import { create } from "zustand";
import type {
  AiKeyword,
  AiSummary,
  AiActionItem,
  AiTopic,
  Decision,
  InvestigationResult,
  QuestionSuggestion,
  Topic,
} from "@/types";

interface InsightsState {
  /** AIキーワード一覧 */
  keywords: AiKeyword[];
  /** サマリー情報 */
  summary: AiSummary | null;
  /** アクションアイテム一覧 */
  actionItems: AiActionItem[];
  /** 決定事項一覧 */
  decisions: Decision[];
  /** 調査結果一覧 */
  investigations: InvestigationResult[];
  /** タイムライン用トピック一覧（既存） */
  topics: Topic[];
  /** AI検出トピック一覧 */
  aiTopics: AiTopic[];
  /** フォーマット済みトランスクリプトテキスト */
  formattedTranscript: string;
  /** 質問候補一覧 */
  questions: QuestionSuggestion[];
  /** AI分析実行中フラグ */
  isAnalyzing: boolean;
}

interface InsightsActions {
  /** キーワードを追加または更新する */
  addKeywords: (keywords: AiKeyword[]) => void;
  /** サマリーを更新する */
  updateSummary: (summary: AiSummary) => void;
  /** アクションアイテムを追加する */
  addActionItems: (items: AiActionItem[]) => void;
  /** 決定事項を追加する */
  addDecisions: (decisions: Decision[]) => void;
  /** 調査結果を追加する */
  addInvestigation: (result: InvestigationResult) => void;
  /** タイムライン用トピックを追加する（既存） */
  addTopics: (topics: Topic[]) => void;
  /** AI検出トピックを追加する */
  addAiTopics: (topics: AiTopic[]) => void;
  /** フォーマット済みトランスクリプトをセットする */
  setFormattedTranscript: (text: string) => void;
  /** 質問候補をセットする */
  setQuestions: (questions: QuestionSuggestion[]) => void;
  /** アクションアイテムの完了状態をトグルする */
  toggleActionItemComplete: (id: string) => void;
  /** AI分析中状態をセットする */
  setAnalyzing: (analyzing: boolean) => void;
  /** 全状態をリセットする */
  clearAll: () => void;
}

const initialState: InsightsState = {
  keywords: [],
  summary: null,
  actionItems: [],
  decisions: [],
  investigations: [],
  topics: [],
  aiTopics: [],
  formattedTranscript: "",
  questions: [],
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

  addAiTopics: (newTopics) => {
    set((state) => ({
      aiTopics: [
        ...state.aiTopics,
        ...newTopics.filter((t) => !state.aiTopics.some((existing) => existing.id === t.id)),
      ],
    }));
  },

  setFormattedTranscript: (text) => {
    set({ formattedTranscript: text });
  },

  setQuestions: (questions) => {
    set({ questions });
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
