import { useState, useCallback, useRef, useEffect } from "react";
import { ExportOptionsForm } from "./ExportOptionsForm";
import type { ExportOptions } from "./ExportOptionsForm";
import { commands } from "@/bindings";
import type { ExportOptions as BackendExportOptions } from "@/bindings";

interface ExportDialogProps {
  open: boolean;
  onClose: () => void;
  sessionId: string | null;
}

function downloadAsFile(content: string) {
  const blob = new Blob([content], { type: "text/markdown;charset=utf-8" });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = `export-${new Date().toISOString().slice(0, 10)}.md`;
  a.click();
  URL.revokeObjectURL(url);
}

export function ExportDialog({ open, onClose, sessionId }: ExportDialogProps) {
  const dialogRef = useRef<HTMLDialogElement>(null);
  const [options, setOptions] = useState<ExportOptions>({
    include_summary: false,
    include_actions: false,
    include_keywords: false,
    include_transcript: true,
  });
  const [markdown, setMarkdown] = useState<string | null>(null);
  const [filename, setFilename] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [copied, setCopied] = useState(false);

  useEffect(() => {
    const dialog = dialogRef.current;
    if (!dialog) return;
    if (open && !dialog.open) dialog.showModal();
    else if (!open && dialog.open) dialog.close();
  }, [open]);

  useEffect(() => {
    if (!open) {
      setMarkdown(null);
      setFilename(null);
      setCopied(false);
    }
  }, [open]);

  const handleBackdropClick = useCallback(
    (e: React.MouseEvent<HTMLDialogElement>) => {
      if (e.target === dialogRef.current) onClose();
    },
    [onClose],
  );

  const toggleOption = useCallback((key: keyof ExportOptions) => {
    setOptions((prev) => ({ ...prev, [key]: !prev[key] }));
  }, []);

  const handleExport = useCallback(async () => {
    if (!sessionId || isLoading) return;
    setIsLoading(true);
    try {
      const backendOptions: BackendExportOptions = {
        session_id: sessionId,
        ...options,
      };
      const result = await commands.exportMarkdown(backendOptions);
      if (result.status === "ok") {
        setMarkdown(result.data.content);
        setFilename(result.data.filename);
      }
    } catch {
      /* Tauri API not available */
    } finally {
      setIsLoading(false);
    }
  }, [sessionId, isLoading, options]);

  const handleCopy = useCallback(async () => {
    if (!markdown) return;
    await navigator.clipboard.writeText(markdown);
    setCopied(true);
    setTimeout(() => {
      setCopied(false);
    }, 2000);
  }, [markdown]);

  const handleSave = useCallback(async () => {
    if (!markdown || !filename) return;
    try {
      await commands.saveExportFile(markdown, filename);
    } catch {
      downloadAsFile(markdown);
    }
  }, [markdown, filename]);

  return (
    <dialog
      ref={dialogRef}
      onClose={onClose}
      onClick={handleBackdropClick}
      className="m-auto backdrop:bg-black/50 rounded-lg p-0 w-full max-w-2xl max-h-[80vh] bg-background text-foreground border border-border shadow-lg"
    >
      <div className="flex flex-col min-h-[50vh] max-h-[80vh]">
        <div className="flex items-center justify-between px-4 py-3 border-b border-border">
          <h3 className="text-sm font-semibold">エクスポート</h3>
          <button
            type="button"
            onClick={onClose}
            className="text-muted-foreground hover:text-foreground text-lg leading-none"
          >
            &times;
          </button>
        </div>
        <div className="flex-1 overflow-y-auto p-4 space-y-4">
          {!markdown ? (
            <ExportOptionsForm
              options={options}
              onToggle={toggleOption}
              onExport={() => void handleExport()}
              isLoading={isLoading}
              disabled={!sessionId}
            />
          ) : (
            <pre className="text-sm leading-relaxed whitespace-pre-wrap">{markdown}</pre>
          )}
        </div>
        {markdown && (
          <div className="flex items-center justify-end gap-2 px-4 py-3 border-t border-border">
            <button
              type="button"
              onClick={() => void handleCopy()}
              className="rounded-md bg-secondary px-3 py-1.5 text-sm font-medium text-secondary-foreground hover:bg-secondary/80 transition-colors"
            >
              {copied ? "コピーしました" : "クリップボードにコピー"}
            </button>
            <button
              type="button"
              onClick={() => void handleSave()}
              className="rounded-md bg-primary px-3 py-1.5 text-sm font-medium text-primary-foreground hover:bg-primary/90 transition-colors"
            >
              ファイルに保存
            </button>
          </div>
        )}
      </div>
    </dialog>
  );
}
