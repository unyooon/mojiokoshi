import { create } from "zustand";
import { persist } from "zustand/middleware";
import type { AppSettings } from "@/types";

interface SettingsState extends AppSettings {
  updateSettings: (partial: Partial<AppSettings>) => void;
  resetSettings: () => void;
}

const defaultSettings: AppSettings = {
  whisperModel: "large-v3-turbo",
  language: "ja",
  vadSensitivity: 0.5,
  analysisIntervalMinutes: 3,
  theme: "system",
  fontSize: 14,
};

export const useSettingsStore = create<SettingsState>()(
  persist(
    (set) => ({
      ...defaultSettings,

      updateSettings: (partial) => set((state) => ({ ...state, ...partial })),

      resetSettings: () => set(defaultSettings),
    }),
    {
      name: "mojiokoshi-settings",
    },
  ),
);
