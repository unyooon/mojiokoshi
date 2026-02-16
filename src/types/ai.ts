// AI analysis domain types (distinct from transcript-level Keyword/ActionItem in index.ts)

export interface AiKeyword {
  id: string;
  term: string;
  type: AiKeywordType;
  definition?: string;
  webSearchResult?: string;
  sourceUrl?: string;
  firstSeenAt: number;
  occurrences: number;
}

export type AiKeywordType = "tech_term" | "proper_noun" | "acronym" | "jargon";

export interface AiSummary {
  text: string;
  updatedAt: number;
  coveringFromMs: number;
  coveringToMs: number;
}

export interface AiActionItem {
  id: string;
  text: string;
  assignee?: string;
  deadline?: string;
  priority: AiPriority;
  completed: boolean;
  detectedAt: number;
}

export type AiPriority = "high" | "medium" | "low";

export interface Decision {
  id: string;
  text: string;
  context: string;
  participants: string[];
  detectedAt: number;
}

export interface InvestigationResult {
  id: string;
  query: string;
  summary: string;
  details: string;
  sources: Source[];
  createdAt: number;
}

export interface Source {
  title: string;
  url: string;
}
