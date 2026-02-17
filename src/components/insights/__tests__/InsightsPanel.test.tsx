import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import { InsightsPanel } from "../InsightsPanel";

// Mock all child components to avoid deep rendering issues with Tauri APIs
vi.mock("@/components/insights/SummaryCard", () => ({
  SummaryCard: () => <div data-testid="summary-card" />,
}));

vi.mock("@/components/insights/KeywordList", () => ({
  KeywordList: () => <div data-testid="keyword-list" />,
}));

vi.mock("@/components/insights/ActionItemList", () => ({
  ActionItemList: () => <div data-testid="action-item-list" />,
}));

vi.mock("@/components/insights/DecisionList", () => ({
  DecisionList: () => <div data-testid="decision-list" />,
}));

vi.mock("@/components/insights/SpeakerPanel", () => ({
  SpeakerPanel: () => <div data-testid="speaker-panel" />,
}));

vi.mock("@/components/insights/InvestigationPanel", () => ({
  InvestigationPanel: () => <div data-testid="investigation-panel" />,
}));

vi.mock("@/components/insights/TopicTimeline", () => ({
  TopicTimeline: () => <div data-testid="topic-timeline" />,
}));

vi.mock("@/components/insights/MinutesPanel", () => ({
  MinutesPanel: ({ sessionId }: { sessionId: string | null }) => (
    <div data-testid="minutes-panel" data-session-id={sessionId} />
  ),
}));

vi.mock("@/components/insights/TranslationPanel", () => ({
  TranslationPanel: ({ sessionId }: { sessionId: string | null }) => (
    <div data-testid="translation-panel" data-session-id={sessionId} />
  ),
}));

vi.mock("@/components/insights/SentimentChart", () => ({
  SentimentChart: ({ sessionId }: { sessionId: string | null }) => (
    <div data-testid="sentiment-chart" data-session-id={sessionId} />
  ),
}));

vi.mock("@/components/insights/MeetingLinksPanel", () => ({
  MeetingLinksPanel: ({ sessionId }: { sessionId: string | null }) => (
    <div data-testid="meeting-links-panel" data-session-id={sessionId} />
  ),
}));

vi.mock("@/components/insights/KeywordDictionaryPanel", () => ({
  KeywordDictionaryPanel: () => <div data-testid="keyword-dictionary-panel" />,
}));

const TAB_LABELS = [
  "サマリー",
  "キーワード",
  "アクション",
  "決定事項",
  "話者",
  "調査",
  "タイムライン",
  "議事録",
  "翻訳",
  "感情",
  "関連会議",
  "辞書",
] as const;

describe("InsightsPanel", () => {
  it("renders all 12 tabs", () => {
    render(<InsightsPanel />);

    for (const label of TAB_LABELS) {
      expect(screen.getByText(label)).toBeInTheDocument();
    }
  });
});
