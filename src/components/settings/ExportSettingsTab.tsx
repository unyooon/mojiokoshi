import { useState, useCallback } from "react";

type ExportFormat = "markdown" | "pdf";

/**
 * @description エクスポートのデフォルト設定を管理するタブ。
 * エクスポート形式とエクスポート先ディレクトリを設定する。
 * @returns エクスポート設定のUI要素
 */
export function ExportSettingsTab() {
  const [defaultFormat, setDefaultFormat] = useState<ExportFormat>("markdown");
  const [exportDir, setExportDir] = useState<string>("");

  const handleSelectDirectory = useCallback(async () => {
    try {
      // Tauri の invoke でネイティブダイアログを呼び出す
      const { invoke } = await import("@tauri-apps/api/core");
      const selected: unknown = await invoke("open_directory_dialog");
      if (typeof selected === "string") {
        setExportDir(selected);
      }
    } catch {
      // Tauri環境外またはコマンド未実装の場合は無視
    }
  }, []);

  const formats: { id: ExportFormat; label: string; description: string }[] = [
    { id: "markdown", label: "Markdown", description: "汎用的なテキスト形式（.md）" },
    { id: "pdf", label: "PDF", description: "印刷・共有に適した形式（.pdf）" },
  ];

  return (
    <div className="space-y-5">
      <div className="space-y-2">
        <h4 className="text-sm font-medium">デフォルトエクスポート形式</h4>
        <div className="space-y-2">
          {formats.map((format) => (
            <label
              key={format.id}
              className={`flex items-start gap-3 rounded-md border px-3 py-2.5 cursor-pointer transition-colors ${
                defaultFormat === format.id
                  ? "border-primary bg-primary/5"
                  : "border-border hover:border-muted-foreground"
              }`}
            >
              <input
                type="radio"
                name="exportFormat"
                value={format.id}
                checked={defaultFormat === format.id}
                onChange={() => {
                  setDefaultFormat(format.id);
                }}
                className="accent-primary mt-0.5"
              />
              <div>
                <p className="text-sm font-medium">{format.label}</p>
                <p className="text-xs text-muted-foreground">{format.description}</p>
              </div>
            </label>
          ))}
        </div>
      </div>

      <div className="space-y-2">
        <h4 className="text-sm font-medium">エクスポート先ディレクトリ</h4>
        <p className="text-xs text-muted-foreground">
          未設定の場合はドキュメントフォルダに保存されます。
        </p>
        <div className="flex gap-2">
          <input
            type="text"
            value={exportDir}
            readOnly
            placeholder="デフォルト（ドキュメント）"
            className="flex-1 rounded-md border border-input bg-background px-3 py-1.5 text-sm text-muted-foreground"
          />
          <button
            type="button"
            onClick={() => void handleSelectDirectory()}
            className="rounded-md border border-border px-3 py-1.5 text-sm font-medium hover:bg-muted transition-colors"
          >
            選択
          </button>
          {exportDir && (
            <button
              type="button"
              onClick={() => {
                setExportDir("");
              }}
              className="rounded-md border border-border px-3 py-1.5 text-sm text-muted-foreground hover:text-foreground transition-colors"
            >
              クリア
            </button>
          )}
        </div>
      </div>
    </div>
  );
}
