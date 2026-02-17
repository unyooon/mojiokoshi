import { create } from "zustand";
import type { MeetingLink } from "@/types";

interface MeetingLinkState {
  links: MeetingLink[];
  isSearching: boolean;
}

interface MeetingLinkActions {
  findRelated: (sessionId: string) => Promise<void>;
  loadLinks: (sessionId: string) => Promise<void>;
  clearLinks: () => void;
}

const initialState: MeetingLinkState = {
  links: [],
  isSearching: false,
};

export const useMeetingLinkStore = create<MeetingLinkState & MeetingLinkActions>((set) => ({
  ...initialState,

  findRelated: async (sessionId: string) => {
    set({ isSearching: true });
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      const result = await invoke<MeetingLink[]>("find_related_meetings", { sessionId });
      set({ links: result });
    } catch {
      // Tauri API not available (browser dev mode)
    } finally {
      set({ isSearching: false });
    }
  },

  loadLinks: async (sessionId: string) => {
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      const result = await invoke<MeetingLink[]>("get_meeting_links", { sessionId });
      set({ links: result });
    } catch {
      // Tauri API not available (browser dev mode)
    }
  },

  clearLinks: () => {
    set(initialState);
  },
}));
