import { useMemo } from "react";
import { useInsightsStore } from "@/stores/insightsStore";
import type { AiTopic, Topic } from "@/types";

/**
 * @description ミリ秒を mm:ss 形式にフォーマットする
 * @param ms - ミリ秒
 * @returns mm:ss 形式の文字列
 */
function formatMs(ms: number): string {
  const totalSeconds = Math.floor(ms / 1000);
  const minutes = Math.floor(totalSeconds / 60);
  const seconds = totalSeconds % 60;
  return `${String(minutes).padStart(2, "0")}:${String(seconds).padStart(2, "0")}`;
}

/**
 * @description タイムスタンプをローカル時刻 HH:MM にフォーマットする
 * @param timestamp - Unixタイムスタンプ（ミリ秒）
 * @returns HH:MM 形式の文字列
 */
function formatTime(timestamp: number): string {
  const date = new Date(timestamp);
  return date.toLocaleTimeString("ja-JP", { hour: "2-digit", minute: "2-digit" });
}

interface AiTopicCardProps {
  /** 表示するAIトピック */
  topic: AiTopic;
}

/**
 * @description AIトピックをカード形式で表示するコンポーネント
 * @param props - コンポーネントプロパティ
 * @returns トピックカード要素
 */
function AiTopicCard({ topic }: AiTopicCardProps) {
  return (
    <div className="rounded-lg border border-border bg-card p-3 mb-3">
      <div className="flex items-center justify-between mb-1.5">
        <p className="text-sm font-semibold text-foreground">{topic.title}</p>
        <span className="text-xs font-mono text-muted-foreground whitespace-nowrap ml-2">
          {formatMs(topic.start_ms)} - {formatMs(topic.end_ms)}
        </span>
      </div>
      {topic.summary && (
        <p className="text-xs text-muted-foreground leading-relaxed">{topic.summary}</p>
      )}
    </div>
  );
}

interface LegacyTopicEntryProps {
  /** 表示するレガシートピック */
  topic: Topic;
}

/**
 * @description レガシートピックをタイムライン形式で表示するコンポーネント
 * @param props - コンポーネントプロパティ
 * @returns タイムラインエントリー要素
 */
function LegacyTopicEntry({ topic }: LegacyTopicEntryProps) {
  return (
    <div className="flex items-start gap-3">
      <div className="flex flex-col items-center">
        <span className="text-xs font-mono text-muted-foreground whitespace-nowrap">
          {formatTime(topic.timestamp)}
        </span>
        <div className="w-px flex-1 bg-border mt-1" />
      </div>
      <div className="pb-6 min-w-0">
        <p className="text-sm font-medium text-foreground">{topic.title}</p>
        {topic.keywords.length > 0 && (
          <div className="flex flex-wrap gap-1 mt-1.5">
            {topic.keywords.map((kw) => (
              <span
                key={kw}
                className="inline-block rounded-full bg-secondary px-2 py-0.5 text-xs text-secondary-foreground"
              >
                {kw}
              </span>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

/**
 * @description トピックタイムラインを表示するコンポーネント。
 * AI検出トピック（AiTopic）とレガシートピック（Topic）の両方を表示する。
 * どちらも空の場合は検出待ちメッセージを表示する。
 */
export function TopicTimeline() {
  const aiTopics = useInsightsStore((s) => s.aiTopics);
  const topics = useInsightsStore((s) => s.topics);

  const sortedAiTopics = useMemo(
    () => [...aiTopics].sort((a, b) => a.start_ms - b.start_ms),
    [aiTopics],
  );
  const sortedTopics = useMemo(
    () => [...topics].sort((a, b) => a.timestamp - b.timestamp),
    [topics],
  );

  const hasAny = sortedAiTopics.length > 0 || sortedTopics.length > 0;

  if (!hasAny) {
    return (
      <div className="flex items-center justify-center py-12">
        <p className="text-sm text-muted-foreground">トピック検出待ち...</p>
      </div>
    );
  }

  return (
    <div className="p-4">
      {sortedAiTopics.length > 0 && (
        <div className="mb-2">
          {sortedAiTopics.map((topic) => (
            <AiTopicCard key={topic.id} topic={topic} />
          ))}
        </div>
      )}
      {sortedTopics.length > 0 && (
        <div>
          {sortedTopics.map((topic) => (
            <LegacyTopicEntry key={topic.id} topic={topic} />
          ))}
        </div>
      )}
    </div>
  );
}
