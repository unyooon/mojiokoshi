// Meeting state
export type MeetingState = "idle" | "recording" | "paused" | "processing" | "review";

// Speaker
export interface Speaker {
  id: string;
  name: string;
  color: SpeakerColor;
}

export interface SpeakerColor {
  bg: string;
  text: string;
  border: string;
}

// Transcript
export interface TranscriptEntry {
  id: string;
  timestamp: number;
  speakerId: string;
  speakerName: string;
  speakerColor: SpeakerColor;
  text: string;
  keywords: Keyword[];
  isPartial: boolean;
  confidence: number;
}

// Keywords
export type KeywordType = "tech_term" | "proper_noun" | "acronym" | "industry_term";
export type KeywordPriority = "high" | "medium" | "low";

export interface Keyword {
  id: string;
  text: string;
  startIndex: number;
  endIndex: number;
  type: KeywordType;
  color: string;
  researchResult?: ResearchResult;
}

export interface ResearchResult {
  query: string;
  summary: string;
  relatedLinks: string[];
  fetchedAt: number;
}

// Meeting session
export interface MeetingSession {
  id: string;
  title: string;
  startedAt: number;
  endedAt?: number;
  targetApp?: string;
  whisperModel?: string;
  status: "active" | "completed" | "archived";
}

// AI Insights
export type InsightType = "summary" | "action_item" | "decision" | "question" | "investigation" | "topic";

export interface AiInsight {
  id: string;
  type: InsightType;
  content: string;
  assignee?: string;
  timeRangeStart?: number;
  timeRangeEnd?: number;
  createdAt: number;
}

export interface ActionItem {
  id: string;
  text: string;
  assignee?: string;
  completed: boolean;
  createdAt: number;
}

// Settings
export interface AppSettings {
  whisperModel: string;
  language: string;
  vadSensitivity: number;
  analysisIntervalMinutes: number;
  theme: "light" | "dark" | "system";
  fontSize: number;
}

// Bookmarks
export interface Bookmark {
  id: string;
  sessionId: string;
  segmentId?: number;
  note?: string;
  createdAt: number;
}
