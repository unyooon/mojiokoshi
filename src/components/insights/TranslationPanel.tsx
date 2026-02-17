import { useCallback } from "react";
import { useTranslationStore } from "@/stores/translationStore";

interface TranslationPanelProps {
  sessionId?: string | null;
}

export function TranslationPanel({ sessionId = null }: TranslationPanelProps) {
  const translations = useTranslationStore((s) => s.translations);
  const targetLang = useTranslationStore((s) => s.targetLang);
  const isTranslating = useTranslationStore((s) => s.isTranslating);
  const translateSession = useTranslationStore((s) => s.translateSession);
  const setTargetLang = useTranslationStore((s) => s.setTargetLang);

  const toggleLang = useCallback(() => {
    setTargetLang(targetLang === "en" ? "ja" : "en");
  }, [targetLang, setTargetLang]);

  const handleTranslate = useCallback(() => {
    if (!sessionId) return;
    void translateSession(sessionId);
  }, [sessionId, translateSession]);

  if (isTranslating) {
    return (
      <div className="flex flex-col items-center justify-center py-12 gap-3">
        <div className="h-5 w-5 animate-spin rounded-full border-2 border-primary border-t-transparent" />
        <span className="text-sm text-muted-foreground">翻訳中...</span>
      </div>
    );
  }

  return (
    <div className="flex flex-col h-full">
      <div className="flex items-center justify-between gap-2 px-4 py-2 border-b border-border">
        <button
          type="button"
          onClick={toggleLang}
          className="rounded-md bg-secondary px-3 py-1 text-xs font-medium text-secondary-foreground hover:bg-secondary/80 transition-colors"
        >
          {targetLang === "en" ? "EN" : "JA"} ← {targetLang === "en" ? "JA" : "EN"}
        </button>
        <button
          type="button"
          onClick={handleTranslate}
          disabled={!sessionId}
          className="rounded-md bg-primary px-4 py-1.5 text-sm font-medium text-primary-foreground hover:bg-primary/90 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
        >
          翻訳実行
        </button>
      </div>

      <div className="flex-1 overflow-y-auto">
        {translations.length === 0 ? (
          <div className="flex items-center justify-center py-12 text-muted-foreground text-sm">
            翻訳データがありません
          </div>
        ) : (
          <ul className="divide-y divide-border">
            {translations.map((entry) => (
              <li key={entry.id} className="px-4 py-3 space-y-1">
                <p className="text-xs text-muted-foreground">{entry.source_text}</p>
                <p className="text-sm text-foreground">{entry.translated_text}</p>
              </li>
            ))}
          </ul>
        )}
      </div>
    </div>
  );
}
