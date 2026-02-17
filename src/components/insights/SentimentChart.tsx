import { useEffect } from "react";
import { useSentimentStore } from "@/stores/sentimentStore";

const emotionColors: Record<string, string> = {
  positive: "bg-green-500",
  negative: "bg-red-500",
  neutral: "bg-yellow-400",
  excited: "bg-blue-500",
  concerned: "bg-orange-500",
  confused: "bg-gray-400",
};

const emotionLabels: Record<string, string> = {
  positive: "ポジティブ",
  negative: "ネガティブ",
  neutral: "ニュートラル",
  excited: "興奮",
  concerned: "懸念",
  confused: "困惑",
};

function getEmotionColor(emotion: string): string {
  return emotionColors[emotion] ?? "bg-gray-400";
}

function getEmotionLabel(emotion: string): string {
  return emotionLabels[emotion] ?? emotion;
}

interface SentimentChartProps {
  sessionId?: string | null;
}

export function SentimentChart({ sessionId = null }: SentimentChartProps) {
  const sentiments = useSentimentStore((s) => s.sentiments);
  const isAnalyzing = useSentimentStore((s) => s.isAnalyzing);
  const analyzeSentiment = useSentimentStore((s) => s.analyzeSentiment);
  const loadSentiments = useSentimentStore((s) => s.loadSentiments);

  useEffect(() => {
    if (sessionId) {
      void loadSentiments(sessionId);
    }
  }, [sessionId, loadSentiments]);

  if (isAnalyzing) {
    return (
      <div className="flex flex-col items-center justify-center py-12 gap-3">
        <div className="h-5 w-5 animate-spin rounded-full border-2 border-primary border-t-transparent" />
        <span className="text-sm text-muted-foreground">感情分析中...</span>
      </div>
    );
  }

  if (sentiments.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center py-12 gap-4">
        <p className="text-sm text-muted-foreground">感情分析データがありません</p>
        <button
          type="button"
          onClick={() => {
            if (sessionId) void analyzeSentiment(sessionId);
          }}
          disabled={!sessionId}
          className="rounded-md bg-primary px-4 py-1.5 text-sm font-medium text-primary-foreground hover:bg-primary/90 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
        >
          感情分析を実行
        </button>
      </div>
    );
  }

  return (
    <div className="p-4 space-y-4">
      <div className="flex items-center justify-between">
        <h3 className="text-sm font-medium">感情分析結果</h3>
        <button
          type="button"
          onClick={() => {
            if (sessionId) void analyzeSentiment(sessionId);
          }}
          disabled={!sessionId}
          className="rounded-md bg-secondary px-3 py-1 text-xs font-medium text-secondary-foreground hover:bg-secondary/80 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
        >
          再分析
        </button>
      </div>
      <div className="space-y-2">
        {sentiments.map((entry) => (
          <div key={entry.id} className="flex items-center gap-3">
            <span className="text-xs text-muted-foreground w-16 shrink-0 text-right tabular-nums">
              {Math.floor(entry.timestamp / 60)}:
              {String(Math.floor(entry.timestamp % 60)).padStart(2, "0")}
            </span>
            <div className="flex-1 flex items-center gap-2">
              <div className="flex-1 h-5 bg-muted rounded-full overflow-hidden">
                <div
                  className={`h-full rounded-full ${getEmotionColor(entry.emotion)}`}
                  style={{ width: `${Math.abs(entry.score) * 100}%` }}
                />
              </div>
              <span className="text-xs font-medium w-20 shrink-0">
                {getEmotionLabel(entry.emotion)}
              </span>
              <span className="text-xs text-muted-foreground w-10 shrink-0 text-right tabular-nums">
                {Math.round(entry.confidence * 100)}%
              </span>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
