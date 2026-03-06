import { useRef, useEffect, useMemo } from "react";
import { useInsightsStore } from "@/stores/insightsStore";
import { KeywordHighlight } from "./KeywordHighlight";

/**
 * タイムスタンプ文字列 [HH:MM] にマッチする正規表現
 */
const TIMESTAMP_REGEX = /(\[(?:\d{1,2}:)?\d{2}:\d{2}\])/g;

interface TimestampPart {
  kind: "timestamp";
  value: string;
}

interface PlainTextPart {
  kind: "text";
  value: string;
}

type TextPart = TimestampPart | PlainTextPart;

/**
 * @description テキストをタイムスタンプと通常テキストに分割する
 * @param text - 分割対象テキスト
 * @returns テキストパーツの配列
 */
function splitTimestamps(text: string): TextPart[] {
  const parts: TextPart[] = [];
  let lastIndex = 0;
  for (const match of text.matchAll(TIMESTAMP_REGEX)) {
    if (match.index > lastIndex) {
      parts.push({ kind: "text", value: text.slice(lastIndex, match.index) });
    }
    parts.push({ kind: "timestamp", value: match[0] });
    lastIndex = match.index + match[0].length;
  }
  if (lastIndex < text.length) {
    parts.push({ kind: "text", value: text.slice(lastIndex) });
  }
  return parts;
}

/**
 * 話者名パターン: 行頭の「名前:」または「名前：」
 */
const SPEAKER_REGEX = /^([^：:]+[：:])\s*/;

interface ParagraphLineProps {
  /** 表示するテキスト行 */
  line: string;
}

/**
 * @description 1行のテキストを話者名ハイライトとタイムスタンプを含めてレンダリングする
 * @param props - コンポーネントプロパティ
 * @returns レンダリング済みの行要素
 */
function ParagraphLine({ line }: ParagraphLineProps) {
  const keywords = useInsightsStore((s) => s.keywords);
  const speakerMatch = SPEAKER_REGEX.exec(line);

  if (speakerMatch) {
    const speakerLabel = speakerMatch[1];
    const rest = line.slice(speakerMatch[0].length);
    const parts = splitTimestamps(rest);
    return (
      <p className="text-sm leading-relaxed mb-1">
        <span className="font-semibold text-foreground">{speakerLabel} </span>
        {parts.map((part, i) => {
          if (part.kind === "timestamp") {
            return (
              <span
                key={i}
                className="text-xs font-mono text-primary/70 bg-primary/10 rounded px-0.5 mx-0.5"
              >
                {part.value}
              </span>
            );
          }
          return <KeywordHighlight key={i} text={part.value} keywords={keywords} />;
        })}
      </p>
    );
  }

  const parts = splitTimestamps(line);
  return (
    <p className="text-sm leading-relaxed mb-1">
      {parts.map((part, i) => {
        if (part.kind === "timestamp") {
          return (
            <span
              key={i}
              className="text-xs font-mono text-primary/70 bg-primary/10 rounded px-0.5 mx-0.5"
            >
              {part.value}
            </span>
          );
        }
        return <KeywordHighlight key={i} text={part.value} keywords={keywords} />;
      })}
    </p>
  );
}

/**
 * @description フォーマット済みトランスクリプトを議事録スタイルで表示するコンポーネント。
 * テキストを段落ごとに分割し、タイムスタンプと話者名をスタイリングして表示する。
 * 新しいテキストが追加されると自動スクロールする。
 */
export function FormattedView() {
  const formattedTranscript = useInsightsStore((s) => s.formattedTranscript);
  const bottomRef = useRef<HTMLDivElement>(null);

  const paragraphs = useMemo(() => {
    if (!formattedTranscript) return [];
    return formattedTranscript
      .split(/\n\n+/)
      .map((block) => block.split("\n").filter((line) => line.trim().length > 0))
      .filter((lines) => lines.length > 0);
  }, [formattedTranscript]);

  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [paragraphs.length]);

  if (!formattedTranscript) {
    return (
      <div className="flex items-center justify-center h-full text-muted-foreground text-sm">
        AI整形中...
      </div>
    );
  }

  return (
    <div className="h-full overflow-y-auto p-4">
      {paragraphs.map((lines, pi) => (
        <div key={pi} className="mb-4">
          {lines.map((line, li) => (
            <ParagraphLine key={li} line={line} />
          ))}
        </div>
      ))}
      <div ref={bottomRef} />
    </div>
  );
}
