import { useState, useCallback } from "react";

interface MinutesPanelProps {
  sessionId: string | null;
  minutes?: string | null;
  isGenerating?: boolean;
  onGenerate?: () => void;
}

export function MinutesPanel({
  sessionId,
  minutes = null,
  isGenerating = false,
  onGenerate,
}: MinutesPanelProps) {
  const [copied, setCopied] = useState(false);

  const handleCopy = useCallback(async () => {
    if (!minutes) return;
    await navigator.clipboard.writeText(minutes);
    setCopied(true);
    setTimeout(() => {
      setCopied(false);
    }, 2000);
  }, [minutes]);

  if (isGenerating) {
    return (
      <div className="flex flex-col items-center justify-center py-12 gap-3">
        <div className="h-5 w-5 animate-spin rounded-full border-2 border-primary border-t-transparent" />
        <span className="text-sm text-muted-foreground">議事録を生成中...</span>
      </div>
    );
  }

  if (!minutes) {
    return (
      <div className="flex flex-col items-center justify-center py-12 gap-4">
        <p className="text-sm text-muted-foreground">録音終了後に議事録を生成できます</p>
        <button
          type="button"
          onClick={onGenerate}
          disabled={!sessionId || !onGenerate}
          className="rounded-md bg-primary px-4 py-1.5 text-sm font-medium text-primary-foreground hover:bg-primary/90 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
        >
          議事録を生成
        </button>
      </div>
    );
  }

  return (
    <div className="flex flex-col h-full">
      <div className="flex items-center justify-end px-4 py-2 border-b border-border">
        <button
          type="button"
          onClick={() => void handleCopy()}
          className="rounded-md bg-secondary px-3 py-1 text-xs font-medium text-secondary-foreground hover:bg-secondary/80 transition-colors"
        >
          {copied ? "コピーしました" : "クリップボードにコピー"}
        </button>
      </div>
      <div className="flex-1 overflow-y-auto p-4">
        <pre className="text-sm leading-relaxed whitespace-pre-wrap">{minutes}</pre>
      </div>
    </div>
  );
}
