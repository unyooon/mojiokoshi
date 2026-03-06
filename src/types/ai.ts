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

export interface Topic {
  id: string;
  title: string;
  timestamp: number;
  keywords: string[];
}

export interface DictionaryKeyword {
  id: number;
  term: string;
  reading: string | null;
  definition: string | null;
  category: string;
  created_at: string;
  updated_at: string;
}

export interface TranslationEntry {
  id: number;
  session_id: string;
  segment_id: number;
  source_lang: string;
  target_lang: string;
  source_text: string;
  translated_text: string;
  created_at: string;
}

export interface SentimentEntry {
  id: number;
  session_id: string;
  segment_id: number;
  score: number;
  emotion: string;
  confidence: number;
  timestamp: number;
  created_at: string;
}

export interface MeetingLink {
  id: number;
  session_id: string;
  related_session_id: string;
  related_title: string;
  similarity_score: number;
  shared_keywords: string[];
  created_at: string;
}

/** 議論トピック */
export interface AiTopic {
  /** トピックの一意識別子 */
  id: string;
  /** トピックのタイトル */
  title: string;
  /** トピックの要約テキスト */
  summary: string;
  /** トピック開始位置（ミリ秒） */
  start_ms: number;
  /** トピック終了位置（ミリ秒） */
  end_ms: number;
}

/** フォーマット済みトランスクリプト */
export interface FormattedTranscript {
  /** 整形済みのテキスト */
  formatted_text: string;
  /** 最後にフォーマットされたセグメントの終了位置（ミリ秒） */
  last_segment_end_ms: number;
}

/** 質問候補 */
export interface QuestionSuggestion {
  /** 質問の一意識別子 */
  id: string;
  /** 質問テキスト */
  text: string;
  /** この質問を提案する理由 */
  reason: string;
}
