import { useState, useCallback, useEffect } from "react";
import { getWhisperModelStatus, downloadWhisperModel } from "@/commands/whisperModel";
import type { WhisperModelStatus } from "@/commands/whisperModel";
import type { AppSettings } from "@/types";

const WHISPER_MODELS = ["tiny", "base", "small", "medium", "large-v3", "large-v3-turbo"];

interface AudioSettingsTabProps {
  settings: AppSettings;
  onUpdate: <K extends keyof AppSettings>(key: K, value: AppSettings[K]) => void;
}

export function AudioSettingsTab({ settings, onUpdate }: AudioSettingsTabProps) {
  const [modelStatus, setModelStatus] = useState<WhisperModelStatus | null>(null);
  const [isDownloading, setIsDownloading] = useState(false);
  const [downloadError, setDownloadError] = useState<string | null>(null);

  const checkStatus = useCallback((model: string) => {
    getWhisperModelStatus(model)
      .then((result) => {
        if (result.status === "ok") {
          setModelStatus(result.data);
        }
      })
      .catch(() => {
        // Tauri not available
      });
  }, []);

  useEffect(() => {
    checkStatus(settings.whisperModel);
  }, [settings.whisperModel, checkStatus]);

  const handleDownload = useCallback(() => {
    setIsDownloading(true);
    setDownloadError(null);
    downloadWhisperModel(settings.whisperModel)
      .then((result) => {
        if (result.status === "ok") {
          checkStatus(settings.whisperModel);
        } else {
          setDownloadError("ダウンロードに失敗しました");
        }
      })
      .catch(() => {
        setDownloadError("ダウンロードに失敗しました");
      })
      .finally(() => {
        setIsDownloading(false);
      });
  }, [settings.whisperModel, checkStatus]);

  const isReady = modelStatus !== null && typeof modelStatus === "object" && "Ready" in modelStatus;
  const isNotDownloaded = modelStatus === "NotDownloaded";

  return (
    <>
      <div className="space-y-2">
        <label className="block space-y-1">
          <span className="text-sm font-medium">Whisper Model</span>
          <div className="flex items-center gap-3">
            <select
              value={settings.whisperModel}
              onChange={(e) => {
                onUpdate("whisperModel", e.target.value);
              }}
              disabled={isDownloading}
              className="block flex-1 rounded-md border border-input bg-background px-3 py-1.5 text-sm"
            >
              {WHISPER_MODELS.map((m) => (
                <option key={m} value={m}>
                  {m}
                </option>
              ))}
            </select>
            {isReady && <span className="shrink-0 text-sm text-green-600">● 準備完了</span>}
            {isNotDownloaded && !isDownloading && (
              <>
                <span className="shrink-0 text-sm text-muted-foreground">○ 未ダウンロード</span>
                <button
                  type="button"
                  onClick={handleDownload}
                  className="shrink-0 rounded-md bg-primary px-3 py-1 text-sm font-medium text-primary-foreground transition-colors hover:bg-primary/90"
                >
                  ダウンロード
                </button>
              </>
            )}
            {isDownloading && (
              <span className="flex shrink-0 items-center gap-1.5 text-sm text-muted-foreground">
                <span className="inline-block h-3.5 w-3.5 animate-spin rounded-full border-2 border-primary border-t-transparent" />
                ダウンロード中...
              </span>
            )}
          </div>
        </label>
        {downloadError && <p className="text-sm text-red-500">{downloadError}</p>}
      </div>

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
            onUpdate("vadSensitivity", Number(e.target.value));
          }}
          className="w-full"
        />
      </label>
    </>
  );
}
