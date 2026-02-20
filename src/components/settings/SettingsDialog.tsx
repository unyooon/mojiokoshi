import { useState, useRef, useEffect, useCallback } from "react";
import { useSettingsStore } from "@/stores/settingsStore";

interface SettingsDialogProps {
  open: boolean;
  onClose: () => void;
}

type SettingsTab = "general" | "audio" | "ai" | "export";

const TABS: { key: SettingsTab; label: string }[] = [
  { key: "general", label: "General" },
  { key: "audio", label: "Audio" },
  { key: "ai", label: "AI" },
  { key: "export", label: "Export" },
];

const WHISPER_MODELS = ["tiny", "base", "small", "medium", "large-v3", "large-v3-turbo"];

const LANGUAGES = [
  { code: "ja", label: "Japanese" },
  { code: "en", label: "English" },
  { code: "zh", label: "Chinese" },
  { code: "ko", label: "Korean" },
  { code: "auto", label: "Auto-detect" },
];

export function SettingsDialog({ open, onClose }: SettingsDialogProps) {
  const dialogRef = useRef<HTMLDialogElement>(null);
  const [tab, setTab] = useState<SettingsTab>("general");
  const settings = useSettingsStore((s) => s.settings);
  const update = useSettingsStore((s) => s.updateSetting);

  useEffect(() => {
    const dialog = dialogRef.current;
    if (!dialog) return;
    if (open && !dialog.open) {
      dialog.showModal();
    } else if (!open && dialog.open) {
      dialog.close();
    }
  }, [open]);

  const handleBackdropClick = useCallback(
    (e: React.MouseEvent<HTMLDialogElement>) => {
      if (e.target === dialogRef.current) onClose();
    },
    [onClose],
  );

  return (
    <dialog
      ref={dialogRef}
      onClose={onClose}
      onClick={handleBackdropClick}
      className="fixed inset-0 m-auto backdrop:bg-black/50 rounded-lg p-0 w-full max-w-lg max-h-[80vh] bg-background text-foreground border border-border shadow-lg"
    >
      <div className="flex flex-col h-full max-h-[80vh]">
        <div className="flex items-center justify-between px-4 py-3 border-b border-border">
          <h3 className="text-sm font-semibold">Settings</h3>
          <button
            type="button"
            onClick={onClose}
            className="text-muted-foreground hover:text-foreground text-lg leading-none"
          >
            &times;
          </button>
        </div>

        <div className="flex border-b border-border px-4 gap-1">
          {TABS.map((t) => (
            <button
              key={t.key}
              type="button"
              onClick={() => {
                setTab(t.key);
              }}
              className={`px-3 py-2 text-sm font-medium border-b-2 transition-colors ${
                tab === t.key
                  ? "border-primary text-foreground"
                  : "border-transparent text-muted-foreground hover:text-foreground"
              }`}
            >
              {t.label}
            </button>
          ))}
        </div>

        <div className="flex-1 overflow-y-auto p-4 space-y-4">
          {tab === "general" && (
            <>
              <fieldset className="space-y-2">
                <legend className="text-sm font-medium">Theme</legend>
                <div className="flex gap-3">
                  {(["light", "dark", "system"] as const).map((v) => (
                    <label key={v} className="flex items-center gap-1.5 text-sm">
                      <input
                        type="radio"
                        name="theme"
                        value={v}
                        checked={settings.theme === v}
                        onChange={() => {
                          update("theme", v);
                        }}
                        className="accent-primary"
                      />
                      {v.charAt(0).toUpperCase() + v.slice(1)}
                    </label>
                  ))}
                </div>
              </fieldset>

              <label className="block space-y-1">
                <span className="text-sm font-medium">Language</span>
                <select
                  value={settings.language}
                  onChange={(e) => {
                    update("language", e.target.value);
                  }}
                  className="block w-full rounded-md border border-input bg-background px-3 py-1.5 text-sm"
                >
                  {LANGUAGES.map((l) => (
                    <option key={l.code} value={l.code}>
                      {l.label}
                    </option>
                  ))}
                </select>
              </label>

              <label className="block space-y-1">
                <span className="text-sm font-medium">Font Size: {settings.fontSize}px</span>
                <input
                  type="range"
                  min={10}
                  max={24}
                  step={1}
                  value={settings.fontSize}
                  onChange={(e) => {
                    update("fontSize", Number(e.target.value));
                  }}
                  className="w-full"
                />
              </label>
            </>
          )}

          {tab === "audio" && (
            <>
              <label className="block space-y-1">
                <span className="text-sm font-medium">Whisper Model</span>
                <select
                  value={settings.whisperModel}
                  onChange={(e) => {
                    update("whisperModel", e.target.value);
                  }}
                  className="block w-full rounded-md border border-input bg-background px-3 py-1.5 text-sm"
                >
                  {WHISPER_MODELS.map((m) => (
                    <option key={m} value={m}>
                      {m}
                    </option>
                  ))}
                </select>
              </label>

              <label className="block space-y-1">
                <span className="text-sm font-medium">
                  VAD Sensitivity: {settings.vadSensitivity.toFixed(2)}
                </span>
                <input
                  type="range"
                  min={0}
                  max={1}
                  step={0.05}
                  value={settings.vadSensitivity}
                  onChange={(e) => {
                    update("vadSensitivity", Number(e.target.value));
                  }}
                  className="w-full"
                />
              </label>
            </>
          )}

          {tab === "ai" && (
            <label className="block space-y-1">
              <span className="text-sm font-medium">
                Analysis Interval: {settings.analysisIntervalMinutes} min
              </span>
              <input
                type="range"
                min={1}
                max={10}
                step={1}
                value={settings.analysisIntervalMinutes}
                onChange={(e) => {
                  update("analysisIntervalMinutes", Number(e.target.value));
                }}
                className="w-full"
              />
            </label>
          )}

          {tab === "export" && (
            <div className="text-sm text-muted-foreground">
              <p>Default export format: Markdown</p>
              <p className="mt-2">
                Additional export options will be available in a future update.
              </p>
            </div>
          )}
        </div>
      </div>
    </dialog>
  );
}
