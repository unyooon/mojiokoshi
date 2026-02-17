import { useState, useCallback, useRef, useEffect } from "react";

interface MinutesExportDialogProps {
  open: boolean;
  minutes: string;
  onClose: () => void;
}

export function MinutesExportDialog({ open, minutes, onClose }: MinutesExportDialogProps) {
  const dialogRef = useRef<HTMLDialogElement>(null);
  const [copied, setCopied] = useState(false);

  useEffect(() => {
    const dialog = dialogRef.current;
    if (!dialog) return;
    if (open && !dialog.open) {
      dialog.showModal();
    } else if (!open && dialog.open) {
      dialog.close();
    }
  }, [open]);

  const handleCopy = useCallback(async () => {
    await navigator.clipboard.writeText(minutes);
    setCopied(true);
    setTimeout(() => {
      setCopied(false);
    }, 2000);
  }, [minutes]);

  const handleSave = useCallback(() => {
    const blob = new Blob([minutes], { type: "text/markdown;charset=utf-8" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = `minutes-${new Date().toISOString().slice(0, 10)}.md`;
    a.click();
    URL.revokeObjectURL(url);
  }, [minutes]);

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
      className="backdrop:bg-black/50 rounded-lg p-0 w-full max-w-2xl max-h-[80vh] bg-background text-foreground border border-border shadow-lg"
    >
      <div className="flex flex-col h-full max-h-[80vh]">
        <div className="flex items-center justify-between px-4 py-3 border-b border-border">
          <h3 className="text-sm font-semibold">議事録エクスポート</h3>
          <button
            type="button"
            onClick={onClose}
            className="text-muted-foreground hover:text-foreground text-lg leading-none"
          >
            &times;
          </button>
        </div>
        <div className="flex-1 overflow-y-auto p-4">
          <pre className="text-sm leading-relaxed whitespace-pre-wrap">{minutes}</pre>
        </div>
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
            onClick={handleSave}
            className="rounded-md bg-primary px-3 py-1.5 text-sm font-medium text-primary-foreground hover:bg-primary/90 transition-colors"
          >
            ファイルに保存
          </button>
        </div>
      </div>
    </dialog>
  );
}
