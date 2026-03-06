import { useState, useCallback } from "react";
import { useSettingsStore } from "@/stores/settingsStore";
import { commands } from "@/bindings";

type ConnectionStatus = "unknown" | "checking" | "connected" | "disconnected" | "error";

const STATUS_INFO: Record<
  ConnectionStatus,
  { label: string; dotClass: string; textClass: string }
> = {
  unknown: {
    label: "未確認",
    dotClass: "bg-muted-foreground",
    textClass: "text-muted-foreground",
  },
  checking: {
    label: "確認中...",
    dotClass: "bg-yellow-400 animate-pulse",
    textClass: "text-muted-foreground",
  },
  connected: { label: "接続済み", dotClass: "bg-green-500", textClass: "text-green-500" },
  disconnected: {
    label: "未接続",
    dotClass: "bg-yellow-500",
    textClass: "text-yellow-500",
  },
  error: { label: "エラー", dotClass: "bg-red-500", textClass: "text-red-500" },
};

/**
 * @description Claude AI接続状態の表示とAI分析の設定を管理するタブ。
 * サイドカープロセスの接続状態確認、AI分析の有効/無効切替、バッチ間隔を設定する。
 * @returns Claude設定のUI要素
 */
export function ClaudeSettingsTab() {
  const settings = useSettingsStore((s) => s.settings);
  const update = useSettingsStore((s) => s.updateSetting);
  const [connectionStatus, setConnectionStatus] = useState<ConnectionStatus>("unknown");
  const [aiEnabled, setAiEnabled] = useState(true);

  const handleCheckConnection = useCallback(() => {
    setConnectionStatus("checking");
    commands
      .healthCheck()
      .then((result) => {
        if (result.status === "ok") {
          setConnectionStatus("connected");
        } else {
          setConnectionStatus("disconnected");
        }
      })
      .catch(() => {
        setConnectionStatus("error");
      });
  }, []);

  const status = STATUS_INFO[connectionStatus];

  return (
    <div className="space-y-5">
      <div className="space-y-2">
        <h4 className="text-sm font-medium">Claude接続状態</h4>
        <div className="flex items-center justify-between rounded-md border border-border px-3 py-2">
          <div className="flex items-center gap-2">
            <span className={`inline-block h-2 w-2 rounded-full ${status.dotClass}`} />
            <span className={`text-sm ${status.textClass}`}>{status.label}</span>
          </div>
          <button
            type="button"
            onClick={handleCheckConnection}
            disabled={connectionStatus === "checking"}
            className="text-xs text-muted-foreground hover:text-foreground transition-colors disabled:opacity-50"
          >
            {connectionStatus === "unknown" ? "確認する" : "再確認"}
          </button>
        </div>
      </div>

      <div className="flex items-center justify-between">
        <div className="space-y-0.5">
          <span className="text-sm font-medium">AI分析</span>
          <p className="text-xs text-muted-foreground">会議中のリアルタイムAI分析を有効にする</p>
        </div>
        <button
          type="button"
          role="switch"
          aria-checked={aiEnabled}
          onClick={() => {
            setAiEnabled((prev) => !prev);
          }}
          className={`relative inline-flex h-5 w-9 items-center rounded-full transition-colors ${
            aiEnabled ? "bg-primary" : "bg-muted"
          }`}
        >
          <span
            className={`inline-block h-3.5 w-3.5 transform rounded-full bg-white shadow transition-transform ${
              aiEnabled ? "translate-x-4.5" : "translate-x-0.5"
            }`}
          />
        </button>
      </div>

      <label className="block space-y-1">
        <span className="text-sm font-medium">
          バッチ間隔: {settings.analysisIntervalMinutes} 分
        </span>
        <p className="text-xs text-muted-foreground">AI分析を実行する間隔（デフォルト: 3分）</p>
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
        <div className="flex justify-between text-xs text-muted-foreground">
          <span>1分</span>
          <span>10分</span>
        </div>
      </label>
    </div>
  );
}
