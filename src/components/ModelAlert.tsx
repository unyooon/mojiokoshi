import { useRef, useEffect, useCallback } from "react";

interface ModelAlertProps {
  open: boolean;
  onOpenSettings: () => void;
  onDismiss: () => void;
}

export function ModelAlert({ open, onOpenSettings, onDismiss }: ModelAlertProps) {
  const dialogRef = useRef<HTMLDialogElement>(null);

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
      if (e.target === dialogRef.current) onDismiss();
    },
    [onDismiss],
  );

  return (
    <dialog
      ref={dialogRef}
      onClose={onDismiss}
      onClick={handleBackdropClick}
      className="fixed inset-0 m-auto backdrop:bg-black/50 rounded-lg p-0 w-full max-w-md bg-background text-foreground border border-border shadow-lg"
    >
      <div className="p-6">
        <h2 className="text-lg font-semibold">Whisper モデルが必要です</h2>
        <p className="mt-2 text-sm text-muted-foreground">
          音声文字起こしにはWhisperモデルのダウンロードが必要です。設定画面からダウンロードしてください。
        </p>
        <p className="mt-1 text-xs text-muted-foreground">
          モデルサイズ目安: base (~150MB), small (~500MB), medium (~1.5GB)
        </p>
        <div className="mt-4 flex justify-end gap-2">
          <button
            type="button"
            onClick={onDismiss}
            className="rounded-md px-3 py-1.5 text-sm text-muted-foreground hover:bg-secondary transition-colors"
          >
            後で
          </button>
          <button
            type="button"
            onClick={onOpenSettings}
            className="rounded-md bg-primary px-3 py-1.5 text-sm font-medium text-primary-foreground hover:bg-primary/90 transition-colors"
          >
            設定を開く
          </button>
        </div>
      </div>
    </dialog>
  );
}
