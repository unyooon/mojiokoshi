import { useState } from "react";
import { useInsightsStore } from "@/stores/insightsStore";
import type { FeatureStatus } from "./FeatureStatusBanner";
import { SummaryCard } from "./SummaryCard";
import { KeywordList } from "./KeywordList";
import { ActionItemList } from "./ActionItemList";
import { DecisionList } from "./DecisionList";
import { SpeakerPanel } from "./SpeakerPanel";
import { InvestigationPanel } from "./InvestigationPanel";
import { MinutesPanel } from "./MinutesPanel";
import { TopicTimeline } from "./TopicTimeline";
import { TranslationPanel } from "./TranslationPanel";
import { SentimentChart } from "./SentimentChart";
import { MeetingLinksPanel } from "./MeetingLinksPanel";
import { KeywordDictionaryPanel } from "./KeywordDictionaryPanel";

type Section =
  | "summary"
  | "keywords"
  | "actions"
  | "decisions"
  | "speakers"
  | "investigation"
  | "timeline"
  | "minutes"
  | "translation"
  | "sentiment"
  | "links"
  | "dictionary";

const sections: { key: Section; label: string; status: FeatureStatus }[] = [
  { key: "summary", label: "サマリー", status: "stub" },
  { key: "keywords", label: "キーワード", status: "stub" },
  { key: "actions", label: "アクション", status: "stub" },
  { key: "decisions", label: "決定事項", status: "stub" },
  { key: "speakers", label: "話者", status: "not-implemented" },
  { key: "investigation", label: "調査", status: "stub" },
  { key: "timeline", label: "タイムライン", status: "stub" },
  { key: "minutes", label: "議事録", status: "stub" },
  { key: "translation", label: "翻訳", status: "stub" },
  { key: "sentiment", label: "感情", status: "stub" },
  { key: "links", label: "関連会議", status: "not-implemented" },
  { key: "dictionary", label: "辞書", status: "ready" },
];

const statusDot: Record<FeatureStatus, string> = {
  ready: "",
  stub: "text-yellow-500",
  "not-implemented": "text-red-500",
};

const sectionComponents: Record<string, React.ComponentType> = {
  summary: SummaryCard,
  keywords: KeywordList,
  actions: ActionItemList,
  decisions: DecisionList,
  speakers: SpeakerPanel,
  investigation: InvestigationPanel,
  timeline: TopicTimeline,
  dictionary: KeywordDictionaryPanel,
};

const sessionIdSections = new Set(["minutes", "translation", "sentiment", "links"]);

type SessionIdComponent = React.ComponentType<{ sessionId: string | null }>;

const sessionIdComponents: Record<string, SessionIdComponent> = {
  minutes: MinutesPanel,
  translation: TranslationPanel,
  sentiment: SentimentChart,
  links: MeetingLinksPanel,
};

interface InsightsPanelProps {
  sessionId?: string | null;
}

export function InsightsPanel({ sessionId = null }: InsightsPanelProps) {
  const [active, setActive] = useState<Section>("summary");
  const isAnalyzing = useInsightsStore((s) => s.isAnalyzing);

  return (
    <div className="flex flex-col h-full">
      <div className="px-4 py-2 border-b border-border flex items-center justify-between">
        <h2 className="text-sm font-semibold">AI Insights</h2>
        {isAnalyzing && (
          <span className="text-xs text-muted-foreground animate-pulse">AI分析中...</span>
        )}
      </div>
      <div className="flex flex-wrap border-b border-border">
        {sections.map((s) => (
          <button
            key={s.key}
            type="button"
            onClick={() => {
              setActive(s.key);
            }}
            className={`px-2 py-1.5 text-xs font-medium transition-colors flex items-center gap-1 ${
              active === s.key
                ? "text-primary border-b-2 border-primary"
                : "text-muted-foreground hover:text-foreground"
            }`}
          >
            {s.label}
            {s.status !== "ready" && (
              <span className={`inline-block w-1.5 h-1.5 rounded-full bg-current ${statusDot[s.status]}`} />
            )}
          </button>
        ))}
      </div>
      <div className="flex-1 overflow-y-auto">
        {sessionIdSections.has(active)
          ? (() => {
              const Comp = sessionIdComponents[active];
              return <Comp sessionId={sessionId} />;
            })()
          : (() => {
              const Comp = sectionComponents[active];
              return <Comp />;
            })()}
      </div>
    </div>
  );
}
