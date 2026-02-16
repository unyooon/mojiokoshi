import { describe, it, expect, beforeEach } from "vitest";
import { useInsightsStore } from "../insightsStore";
import type { AiKeyword, AiSummary, AiActionItem, Decision, InvestigationResult } from "@/types";

function makeKeyword(overrides: Partial<AiKeyword> = {}): AiKeyword {
  return {
    id: "kw-1",
    term: "WebRTC",
    type: "tech_term",
    firstSeenAt: 1000,
    occurrences: 1,
    ...overrides,
  };
}

function makeActionItem(overrides: Partial<AiActionItem> = {}): AiActionItem {
  return {
    id: "ai-1",
    text: "Review the deployment plan",
    priority: "high",
    completed: false,
    detectedAt: 2000,
    ...overrides,
  };
}

function makeDecision(overrides: Partial<Decision> = {}): Decision {
  return {
    id: "d-1",
    text: "Use Rust for the backend",
    context: "Performance requirements",
    participants: ["Alice", "Bob"],
    detectedAt: 3000,
    ...overrides,
  };
}

function makeInvestigation(overrides: Partial<InvestigationResult> = {}): InvestigationResult {
  return {
    id: "inv-1",
    query: "What is WebRTC?",
    summary: "A real-time communication protocol",
    details: "WebRTC enables peer-to-peer communication.",
    sources: [],
    createdAt: 4000,
    ...overrides,
  };
}

describe("insightsStore", () => {
  beforeEach(() => {
    useInsightsStore.getState().clearAll();
  });

  it("starts with empty initial state", () => {
    const state = useInsightsStore.getState();
    expect(state.keywords).toEqual([]);
    expect(state.summary).toBeNull();
    expect(state.actionItems).toEqual([]);
    expect(state.decisions).toEqual([]);
    expect(state.investigations).toEqual([]);
    expect(state.isAnalyzing).toBe(false);
  });

  describe("addKeywords", () => {
    it("adds new keywords", () => {
      useInsightsStore.getState().addKeywords([makeKeyword()]);
      expect(useInsightsStore.getState().keywords).toHaveLength(1);
      expect(useInsightsStore.getState().keywords[0].term).toBe("WebRTC");
    });

    it("deduplicates by term and sums occurrences", () => {
      useInsightsStore.getState().addKeywords([makeKeyword({ occurrences: 2 })]);
      useInsightsStore.getState().addKeywords([makeKeyword({ id: "kw-2", occurrences: 3 })]);
      const keywords = useInsightsStore.getState().keywords;
      expect(keywords).toHaveLength(1);
      expect(keywords[0].occurrences).toBe(5);
    });

    it("keeps distinct terms separate", () => {
      useInsightsStore
        .getState()
        .addKeywords([
          makeKeyword({ id: "kw-1", term: "WebRTC" }),
          makeKeyword({ id: "kw-2", term: "Rust" }),
        ]);
      expect(useInsightsStore.getState().keywords).toHaveLength(2);
    });
  });

  describe("updateSummary", () => {
    it("sets the summary", () => {
      const summary: AiSummary = {
        text: "Meeting about infrastructure",
        updatedAt: 5000,
        coveringFromMs: 0,
        coveringToMs: 10000,
      };
      useInsightsStore.getState().updateSummary(summary);
      expect(useInsightsStore.getState().summary).toEqual(summary);
    });

    it("replaces existing summary", () => {
      const first: AiSummary = {
        text: "First",
        updatedAt: 1000,
        coveringFromMs: 0,
        coveringToMs: 5000,
      };
      const second: AiSummary = {
        text: "Second",
        updatedAt: 2000,
        coveringFromMs: 0,
        coveringToMs: 10000,
      };
      useInsightsStore.getState().updateSummary(first);
      useInsightsStore.getState().updateSummary(second);
      expect(useInsightsStore.getState().summary?.text).toBe("Second");
    });
  });

  describe("addActionItems", () => {
    it("adds action items", () => {
      useInsightsStore.getState().addActionItems([makeActionItem()]);
      expect(useInsightsStore.getState().actionItems).toHaveLength(1);
    });

    it("deduplicates by id", () => {
      useInsightsStore.getState().addActionItems([makeActionItem()]);
      useInsightsStore.getState().addActionItems([makeActionItem({ text: "Updated text" })]);
      expect(useInsightsStore.getState().actionItems).toHaveLength(1);
      // Keeps the first version
      expect(useInsightsStore.getState().actionItems[0].text).toBe("Review the deployment plan");
    });
  });

  describe("addDecisions", () => {
    it("adds decisions", () => {
      useInsightsStore.getState().addDecisions([makeDecision()]);
      expect(useInsightsStore.getState().decisions).toHaveLength(1);
    });

    it("deduplicates by id", () => {
      useInsightsStore.getState().addDecisions([makeDecision()]);
      useInsightsStore.getState().addDecisions([makeDecision({ text: "Changed decision" })]);
      expect(useInsightsStore.getState().decisions).toHaveLength(1);
    });
  });

  describe("addInvestigation", () => {
    it("appends investigation results", () => {
      useInsightsStore.getState().addInvestigation(makeInvestigation());
      useInsightsStore
        .getState()
        .addInvestigation(makeInvestigation({ id: "inv-2", query: "What is Rust?" }));
      expect(useInsightsStore.getState().investigations).toHaveLength(2);
    });
  });

  describe("toggleActionItemComplete", () => {
    it("toggles completed state", () => {
      useInsightsStore.getState().addActionItems([makeActionItem()]);
      expect(useInsightsStore.getState().actionItems[0].completed).toBe(false);
      useInsightsStore.getState().toggleActionItemComplete("ai-1");
      expect(useInsightsStore.getState().actionItems[0].completed).toBe(true);
      useInsightsStore.getState().toggleActionItemComplete("ai-1");
      expect(useInsightsStore.getState().actionItems[0].completed).toBe(false);
    });

    it("does not affect other items", () => {
      useInsightsStore
        .getState()
        .addActionItems([
          makeActionItem({ id: "ai-1" }),
          makeActionItem({ id: "ai-2", text: "Other task" }),
        ]);
      useInsightsStore.getState().toggleActionItemComplete("ai-1");
      expect(useInsightsStore.getState().actionItems[0].completed).toBe(true);
      expect(useInsightsStore.getState().actionItems[1].completed).toBe(false);
    });
  });

  describe("setAnalyzing", () => {
    it("sets analyzing flag", () => {
      useInsightsStore.getState().setAnalyzing(true);
      expect(useInsightsStore.getState().isAnalyzing).toBe(true);
      useInsightsStore.getState().setAnalyzing(false);
      expect(useInsightsStore.getState().isAnalyzing).toBe(false);
    });
  });

  describe("clearAll", () => {
    it("resets to initial state", () => {
      // Populate store with data
      useInsightsStore.getState().addKeywords([makeKeyword()]);
      useInsightsStore.getState().updateSummary({
        text: "Summary",
        updatedAt: 1000,
        coveringFromMs: 0,
        coveringToMs: 5000,
      });
      useInsightsStore.getState().addActionItems([makeActionItem()]);
      useInsightsStore.getState().addDecisions([makeDecision()]);
      useInsightsStore.getState().addInvestigation(makeInvestigation());
      useInsightsStore.getState().setAnalyzing(true);

      // Clear
      useInsightsStore.getState().clearAll();

      const state = useInsightsStore.getState();
      expect(state.keywords).toEqual([]);
      expect(state.summary).toBeNull();
      expect(state.actionItems).toEqual([]);
      expect(state.decisions).toEqual([]);
      expect(state.investigations).toEqual([]);
      expect(state.isAnalyzing).toBe(false);
    });
  });
});
