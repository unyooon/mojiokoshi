import { describe, it, expect, beforeEach } from "vitest";
import { useSpeakerStore } from "../speakerStore";

describe("speakerStore", () => {
  beforeEach(() => {
    useSpeakerStore.getState().clearSpeakers();
  });

  it("addSpeaker creates new speaker with auto label", () => {
    const speaker = useSpeakerStore.getState().addSpeaker("s1");
    expect(speaker.id).toBe("s1");
    expect(speaker.label).toBe("Speaker 1");
    expect(speaker.isSelf).toBe(false);
  });

  it("addSpeaker with custom label", () => {
    const speaker = useSpeakerStore.getState().addSpeaker("s1", "Alice");
    expect(speaker.label).toBe("Alice");
  });

  it("addSpeaker returns existing for duplicate id", () => {
    const s1 = useSpeakerStore.getState().addSpeaker("s1", "Alice");
    const s2 = useSpeakerStore.getState().addSpeaker("s1", "Bob");
    expect(s1).toBe(s2); // same reference
    expect(s2.label).toBe("Alice"); // original label preserved
  });

  it("updateSpeakerLabel changes label", () => {
    useSpeakerStore.getState().addSpeaker("s1", "Alice");
    useSpeakerStore.getState().updateSpeakerLabel("s1", "Bob");
    expect(useSpeakerStore.getState().speakers.get("s1")?.label).toBe("Bob");
  });

  it("setSelfSpeaker marks one speaker as self", () => {
    useSpeakerStore.getState().addSpeaker("s1");
    useSpeakerStore.getState().addSpeaker("s2");
    useSpeakerStore.getState().setSelfSpeaker("s1");
    expect(useSpeakerStore.getState().speakers.get("s1")?.isSelf).toBe(true);
    expect(useSpeakerStore.getState().speakers.get("s2")?.isSelf).toBe(false);
  });

  it("setSelfSpeaker updates when changed", () => {
    useSpeakerStore.getState().addSpeaker("s1");
    useSpeakerStore.getState().addSpeaker("s2");
    useSpeakerStore.getState().setSelfSpeaker("s1");
    useSpeakerStore.getState().setSelfSpeaker("s2");
    expect(useSpeakerStore.getState().speakers.get("s1")?.isSelf).toBe(false);
    expect(useSpeakerStore.getState().speakers.get("s2")?.isSelf).toBe(true);
  });

  it("setActiveSpeaker sets active id", () => {
    useSpeakerStore.getState().setActiveSpeaker("s1");
    expect(useSpeakerStore.getState().activeSpeakerId).toBe("s1");
  });

  it("clearSpeakers resets all state", () => {
    useSpeakerStore.getState().addSpeaker("s1");
    useSpeakerStore.getState().setActiveSpeaker("s1");
    useSpeakerStore.getState().clearSpeakers();
    expect(useSpeakerStore.getState().speakers.size).toBe(0);
    expect(useSpeakerStore.getState().activeSpeakerId).toBeNull();
  });

  it("auto-assigns colors in order", () => {
    const s1 = useSpeakerStore.getState().addSpeaker("s1");
    const s2 = useSpeakerStore.getState().addSpeaker("s2");
    expect(s1.color).toBe("blue");
    expect(s2.color).toBe("green");
  });
});
