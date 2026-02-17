import { describe, it, expect } from "vitest";
import { useKeywordDictionaryStore } from "../keywordDictionaryStore";

describe("keywordDictionaryStore", () => {
  it("initial state has empty keywords and isLoading false", () => {
    const state = useKeywordDictionaryStore.getState();
    expect(state.keywords).toEqual([]);
    expect(state.isLoading).toBe(false);
  });
});
