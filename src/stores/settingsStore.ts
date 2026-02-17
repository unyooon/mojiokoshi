import { create } from "zustand";
import type { AppSettings } from "@/types";

const DEFAULT_SETTINGS: AppSettings = {
  whisperModel: "large-v3-turbo",
  language: "ja",
  vadSensitivity: 0.5,
  analysisIntervalMinutes: 3,
  theme: "system",
  fontSize: 14,
};

interface SettingsState {
  settings: AppSettings;
  isLoading: boolean;
}

interface SettingsActions {
  loadSettings: () => Promise<void>;
  updateSetting: <K extends keyof AppSettings>(key: K, value: AppSettings[K]) => void;
  resetSettings: () => void;
}

export const useSettingsStore = create<SettingsState & SettingsActions>((set) => ({
  settings: DEFAULT_SETTINGS,
  isLoading: true,

  loadSettings: async () => {
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      const stored = await invoke<Record<string, string>>("get_all_settings");
      const merged = { ...DEFAULT_SETTINGS };
      if (stored.theme) merged.theme = stored.theme as AppSettings["theme"];
      if (stored.whisperModel) merged.whisperModel = stored.whisperModel;
      if (stored.language) merged.language = stored.language;
      if (stored.vadSensitivity) merged.vadSensitivity = Number(stored.vadSensitivity);
      if (stored.analysisIntervalMinutes)
        merged.analysisIntervalMinutes = Number(stored.analysisIntervalMinutes);
      if (stored.fontSize) merged.fontSize = Number(stored.fontSize);
      set({ settings: merged, isLoading: false });
    } catch {
      set({ isLoading: false });
    }
  },

  updateSetting: (key, value) => {
    set((state) => ({
      settings: { ...state.settings, [key]: value },
    }));
    // Fire-and-forget persistence
    void (async () => {
      try {
        const { invoke } = await import("@tauri-apps/api/core");
        await invoke("set_setting", { key, value: String(value) });
      } catch {
        /* not in Tauri env */
      }
    })();
  },

  resetSettings: () => {
    set({ settings: DEFAULT_SETTINGS });
  },
}));
