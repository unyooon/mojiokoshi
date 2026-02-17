import { create } from "zustand";
import type { Speaker, SpeakerColor } from "@/types";
import { SPEAKER_COLORS } from "@/types";

interface SpeakerState {
  speakers: Map<string, Speaker>;
  activeSpeakerId: string | null;
}

interface SpeakerActions {
  addSpeaker: (id: string, label?: string) => Speaker;
  updateSpeakerLabel: (id: string, label: string) => void;
  setSelfSpeaker: (id: string) => void;
  setActiveSpeaker: (id: string | null) => void;
  clearSpeakers: () => void;
}

function nextColor(speakers: Map<string, Speaker>): SpeakerColor {
  const usedCount = speakers.size;
  return SPEAKER_COLORS[usedCount % SPEAKER_COLORS.length];
}

export const useSpeakerStore = create<SpeakerState & SpeakerActions>((set, get) => ({
  speakers: new Map(),
  activeSpeakerId: null,

  addSpeaker: (id, label) => {
    const existing = get().speakers.get(id);
    if (existing) return existing;

    const speaker: Speaker = {
      id,
      label: label ?? `Speaker ${get().speakers.size + 1}`,
      color: nextColor(get().speakers),
      isSelf: false,
    };

    set((state) => {
      const next = new Map(state.speakers);
      next.set(id, speaker);
      return { speakers: next };
    });

    return speaker;
  },

  updateSpeakerLabel: (id, label) => {
    set((state) => {
      const speaker = state.speakers.get(id);
      if (!speaker) return state;
      const next = new Map(state.speakers);
      next.set(id, { ...speaker, label });
      return { speakers: next };
    });
  },

  setSelfSpeaker: (id) => {
    set((state) => {
      const next = new Map(state.speakers);
      for (const [key, speaker] of next) {
        next.set(key, { ...speaker, isSelf: key === id });
      }
      return { speakers: next };
    });
  },

  setActiveSpeaker: (id) => {
    set({ activeSpeakerId: id });
  },

  clearSpeakers: () => {
    set({ speakers: new Map(), activeSpeakerId: null });
  },
}));
