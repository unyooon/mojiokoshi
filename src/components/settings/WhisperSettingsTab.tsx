import { useState, useCallback } from "react";
import { useSettingsStore } from "@/stores/settingsStore";

const WHISPER_MODELS = [
  { id: "tiny", label: "Tiny (~75MB)" },
  { id: "base", label: "Base (~142MB)" },
  { id: "small", label: "Small (~466MB)" },
  { id: "medium", label: "Medium (~1.5GB)" },
  { id: "large-v3-turbo", label: "Large v3 Turbo (~809MB) [推奨]" },
];

type DownloadStatus = "idle" | "downloading" | "done" | "error";

/**
 * @description Whisperモデルの選択とダウンロード状態を管理する設定タブ。
 * @returns WhisperモデルのUI要素
 */
export function WhisperSettingsTab() {
  const settings = useSettingsStore((s) => s.settings);
  const update = useSettingsStore((s) => s.updateSetting);
  const [downloadStatus, setDownloadStatus] = useState<DownloadStatus>("idle");
  const [downloadingModel, setDownloadingModel] = useState<string | null>(null);

  const handleDownload = useCallback(async (modelId: string) => {
    setDownloadingModel(modelId);
    setDownloadStatus("downloading");
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      await invoke("download_whisper_model", { model: modelId });
      setDownloadStatus("done");
    } catch {
      setDownloadStatus("error");
    }
  }, []);

  const handleSelectModel = useCallback(
    (modelId: string) => {
      update("whisperModel", modelId);
    },
    [update],
  );

  return (
    <div className="space-y-4">
      <div>
        <h4 className="text-sm font-medium mb-2">Whisperモデル選択</h4>
        <p className="text-xs text-muted-foreground mb-3">
          精度とパフォーマンスのバランスに合わせてモデルを選択してください。
        </p>
      </div>

      <div className="space-y-2">
        {WHISPER_MODELS.map((model) => (
          <div
            key={model.id}
            className={`flex items-center justify-between rounded-md border px-3 py-2 transition-colors ${
              settings.whisperModel === model.id
                ? "border-primary bg-primary/5"
                : "border-border hover:border-muted-foreground"
            }`}
          >
            <label className="flex items-center gap-2 flex-1 cursor-pointer">
              <input
                type="radio"
                name="whisperModel"
                value={model.id}
                checked={settings.whisperModel === model.id}
                onChange={() => {
                  handleSelectModel(model.id);
                }}
                className="accent-primary"
              />
              <span className="text-sm">{model.label}</span>
            </label>

            {downloadingModel === model.id && (
              <span className="text-xs text-muted-foreground ml-2">
                {downloadStatus === "downloading" && "ダウンロード中..."}
                {downloadStatus === "done" && "完了"}
                {downloadStatus === "error" && "エラー"}
              </span>
            )}

            {downloadingModel !== model.id && (
              <button
                type="button"
                onClick={() => void handleDownload(model.id)}
                className="ml-2 text-xs text-muted-foreground hover:text-foreground transition-colors"
              >
                ダウンロード
              </button>
            )}
          </div>
        ))}
      </div>
    </div>
  );
}
