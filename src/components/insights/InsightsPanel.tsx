import { useState } from "react";
import { useInsightsStore } from "@/stores/insightsStore";
import { SummaryCard } from "./SummaryCard";
import { KeywordList } from "./KeywordList";
import { ActionItemList } from "./ActionItemList";
import { DecisionList } from "./DecisionList";
import { InvestigationPanel } from "./InvestigationPanel";

type Section = "summary" | "keywords" | "actions" | "decisions" | "investigation";

const sections: { key: Section; label: string }[] = [
  { key: "summary", label: "サマリー" },
  { key: "keywords", label: "キーワード" },
  { key: "actions", label: "アクション" },
  { key: "decisions", label: "決定事項" },
  { key: "investigation", label: "調査" },
];

const sectionComponents: Record<Section, React.ComponentType> = {
  summary: SummaryCard,
  keywords: KeywordList,
  actions: ActionItemList,
  decisions: DecisionList,
  investigation: InvestigationPanel,
};

export function InsightsPanel() {
  const [active, setActive] = useState<Section>("summary");
  const isAnalyzing = useInsightsStore((s) => s.isAnalyzing);

  const ActiveComponent = sectionComponents[active];

  return (
    <div className="flex flex-col h-full">
      <div className="px-4 py-2 border-b border-border flex items-center justify-between">
        <h2 className="text-sm font-semibold">AI Insights</h2>
        {isAnalyzing && (
          <span className="text-xs text-muted-foreground animate-pulse">AI分析中...</span>
        )}
      </div>
      <div className="flex border-b border-border">
        {sections.map((s) => (
          <button
            key={s.key}
            type="button"
            onClick={() => {
              setActive(s.key);
            }}
            className={`flex-1 px-2 py-1.5 text-xs font-medium transition-colors ${
              active === s.key
                ? "text-primary border-b-2 border-primary"
                : "text-muted-foreground hover:text-foreground"
            }`}
          >
            {s.label}
          </button>
        ))}
      </div>
      <div className="flex-1 overflow-y-auto">
        <ActiveComponent />
      </div>
    </div>
  );
}
