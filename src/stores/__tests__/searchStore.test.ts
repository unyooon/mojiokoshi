import { describe, it, expect, beforeEach } from "vitest";
import { useSearchStore } from "../searchStore";
import type { SearchMatch } from "../searchStore";

function makeMatch(overrides: Partial<SearchMatch> = {}): SearchMatch {
  return {
    entryId: "entry-1",
    startIndex: 0,
    endIndex: 5,
    ...overrides,
  };
}

describe("searchStore", () => {
  beforeEach(() => {
    useSearchStore.getState().close();
  });

  it("setQuery updates query and resets match index", () => {
    // Set some matches and advance the index first
    useSearchStore.getState().setMatches([makeMatch(), makeMatch({ entryId: "entry-2" })]);
    useSearchStore.getState().nextMatch();
    expect(useSearchStore.getState().currentMatchIndex).toBe(1);

    // setQuery should reset currentMatchIndex to 0
    useSearchStore.getState().setQuery("hello");
    expect(useSearchStore.getState().query).toBe("hello");
    expect(useSearchStore.getState().currentMatchIndex).toBe(0);
  });

  it("open sets isOpen to true", () => {
    expect(useSearchStore.getState().isOpen).toBe(false);
    useSearchStore.getState().open();
    expect(useSearchStore.getState().isOpen).toBe(true);
  });

  it("close resets all state", () => {
    // Populate store
    useSearchStore.getState().open();
    useSearchStore.getState().setQuery("test");
    useSearchStore.getState().setMatches([makeMatch(), makeMatch({ entryId: "entry-2" })]);
    useSearchStore.getState().nextMatch();

    // Close should reset everything
    useSearchStore.getState().close();
    const state = useSearchStore.getState();
    expect(state.isOpen).toBe(false);
    expect(state.query).toBe("");
    expect(state.matches).toEqual([]);
    expect(state.currentMatchIndex).toBe(0);
  });

  it("setMatches updates matches array", () => {
    const matches = [
      makeMatch({ entryId: "e1", startIndex: 0, endIndex: 3 }),
      makeMatch({ entryId: "e2", startIndex: 5, endIndex: 10 }),
    ];
    useSearchStore.getState().setMatches(matches);
    expect(useSearchStore.getState().matches).toHaveLength(2);
    expect(useSearchStore.getState().matches[0].entryId).toBe("e1");
    expect(useSearchStore.getState().matches[1].entryId).toBe("e2");
  });

  it("nextMatch cycles through matches", () => {
    const matches = [
      makeMatch({ entryId: "e1" }),
      makeMatch({ entryId: "e2" }),
      makeMatch({ entryId: "e3" }),
    ];
    useSearchStore.getState().setMatches(matches);
    expect(useSearchStore.getState().currentMatchIndex).toBe(0);

    useSearchStore.getState().nextMatch();
    expect(useSearchStore.getState().currentMatchIndex).toBe(1);

    useSearchStore.getState().nextMatch();
    expect(useSearchStore.getState().currentMatchIndex).toBe(2);
  });

  it("prevMatch cycles backwards", () => {
    const matches = [
      makeMatch({ entryId: "e1" }),
      makeMatch({ entryId: "e2" }),
      makeMatch({ entryId: "e3" }),
    ];
    useSearchStore.getState().setMatches(matches);

    // Move forward to index 2
    useSearchStore.getState().nextMatch();
    useSearchStore.getState().nextMatch();
    expect(useSearchStore.getState().currentMatchIndex).toBe(2);

    // Move back
    useSearchStore.getState().prevMatch();
    expect(useSearchStore.getState().currentMatchIndex).toBe(1);

    useSearchStore.getState().prevMatch();
    expect(useSearchStore.getState().currentMatchIndex).toBe(0);
  });

  it("nextMatch wraps around to 0", () => {
    const matches = [makeMatch({ entryId: "e1" }), makeMatch({ entryId: "e2" })];
    useSearchStore.getState().setMatches(matches);

    // Advance past the end
    useSearchStore.getState().nextMatch(); // 1
    useSearchStore.getState().nextMatch(); // wraps to 0
    expect(useSearchStore.getState().currentMatchIndex).toBe(0);
  });

  it("prevMatch wraps to last", () => {
    const matches = [
      makeMatch({ entryId: "e1" }),
      makeMatch({ entryId: "e2" }),
      makeMatch({ entryId: "e3" }),
    ];
    useSearchStore.getState().setMatches(matches);

    // At index 0, going back should wrap to last index
    useSearchStore.getState().prevMatch();
    expect(useSearchStore.getState().currentMatchIndex).toBe(2);
  });
});
