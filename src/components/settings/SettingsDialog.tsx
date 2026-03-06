import { useState, useRef, useEffect, useCallback } from "react";
import { useSettingsStore } from "@/stores/settingsStore";
import { AudioSettingsTab } from "./AudioSettingsTab";
import { WhisperSettingsTab } from "./WhisperSettingsTab";
import { ClaudeSettingsTab } from "./ClaudeSettingsTab";
import { ExportSettingsTab } from "./ExportSettingsTab";

interface SettingsDialogProps {
  /** ダイアログの表示状態 */
  open: boolean;
  /** ダイアログを閉じるコールバック */
  onClose: () => void;
}

type SettingsTab = "general" | "audio" | "whisper" | "claude" | "export";

const TABS: { key: SettingsTab; label: string }[] = [
  { key: "general", label: "一般" },
  { key: "audio", label: "音声" },
  { key: "whisper", label: "Whisper" },
  { key: "claude", label: "Claude" },
  { key: "export", label: "エクスポート" },
];

const LANGUAGES = [
  { code: "ja", label: "Japanese" },
  { code: "en", label: "English" },
  { code: "zh", label: "Chinese" },
  { code: "ko", label: "Korean" },
  { code: "auto", label: "Auto-detect" },
];

/**
 * @description アプリケーション設定ダイアログ。
 * 一般 / 音声 / Whisper / Claude / エクスポートの5タブで構成される。
 * @param props.open - ダイアログの表示状態
 * @param props.onClose - ダイアログを閉じるコールバック
 * @returns 設定ダイアログ要素
 */
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
      className="m-auto backdrop:bg-black/50 rounded-lg p-0 w-full max-w-lg max-h-[80vh] bg-background text-foreground border border-border shadow-lg"
    >
      <div className="flex flex-col min-h-[50vh] max-h-[80vh]">
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

        <div className="flex border-b border-border px-4 gap-1 overflow-x-auto">
          {TABS.map((t) => (
            <button
              key={t.key}
              type="button"
              onClick={() => {
                setTab(t.key);
              }}
              className={`px-3 py-2 text-sm font-medium border-b-2 transition-colors whitespace-nowrap ${
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

          {tab === "audio" && <AudioSettingsTab settings={settings} onUpdate={update} />}

          {tab === "whisper" && <WhisperSettingsTab />}

          {tab === "claude" && <ClaudeSettingsTab />}

          {tab === "export" && <ExportSettingsTab />}
        </div>
      </div>
    </dialog>
  );
}
