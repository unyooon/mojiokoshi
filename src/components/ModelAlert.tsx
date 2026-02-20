interface ModelAlertProps {
  open: boolean;
  onOpenSettings: () => void;
  onDismiss: () => void;
}

export function ModelAlert({ open, onOpenSettings, onDismiss }: ModelAlertProps) {
  if (!open) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
      <div className="mx-4 max-w-md rounded-lg bg-background p-6 shadow-lg border border-border">
        <h2 className="text-lg font-semibold">Whisper モデルが必要です</h2>
        <p className="mt-2 text-sm text-muted-foreground">
          音声文字起こしにはWhisperモデルのダウンロードが必要です。設定画面からダウンロードしてください。
        </p>
        <p className="mt-1 text-xs text-muted-foreground">
          モデルサイズ目安: base (~150MB), small (~500MB), medium (~1.5GB)
        </p>
        <div className="mt-4 flex justify-end gap-2">
          <button
            onClick={onDismiss}
            className="rounded-md px-3 py-1.5 text-sm text-muted-foreground hover:bg-secondary transition-colors"
          >
            後で
          </button>
          <button
            onClick={onOpenSettings}
            className="rounded-md bg-primary px-3 py-1.5 text-sm font-medium text-primary-foreground hover:bg-primary/90 transition-colors"
          >
            設定を開く
          </button>
        </div>
      </div>
    </div>
  );
}
