import { describe, it, expect, beforeEach } from "vitest";
import { useTranslationStore } from "../translationStore";

describe("translationStore", () => {
  beforeEach(() => {
    useTranslationStore.getState().clearTranslations();
  });

  it("initial state has empty translations and targetLang 'en'", () => {
    const state = useTranslationStore.getState();
    expect(state.translations).toEqual([]);
    expect(state.targetLang).toBe("en");
    expect(state.isTranslating).toBe(false);
  });

  it("setTargetLang updates targetLang", () => {
    useTranslationStore.getState().setTargetLang("ja");
    expect(useTranslationStore.getState().targetLang).toBe("ja");

    useTranslationStore.getState().setTargetLang("fr");
    expect(useTranslationStore.getState().targetLang).toBe("fr");
  });

  it("clearTranslations resets to initial state", () => {
    // Modify state via synchronous actions
    useTranslationStore.getState().setTargetLang("de");

    // Clear should reset translations and isTranslating but not targetLang
    useTranslationStore.getState().clearTranslations();
    const state = useTranslationStore.getState();
    expect(state.translations).toEqual([]);
    expect(state.isTranslating).toBe(false);
  });
});
