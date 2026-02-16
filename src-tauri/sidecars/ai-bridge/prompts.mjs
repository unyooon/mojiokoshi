/** JSON Schema for the unified analysis response. */
export const ANALYSIS_SCHEMA = {
  type: "object",
  required: ["keywords", "summary", "action_items", "decisions"],
  properties: {
    keywords: {
      type: "array",
      items: {
        type: "object",
        required: ["id", "term", "keyword_type", "first_seen_at", "occurrences"],
        properties: {
          id: { type: "string" },
          term: { type: "string" },
          keyword_type: { type: "string", enum: ["TechTerm", "ProperNoun", "Acronym", "Jargon"] },
          definition: { type: ["string", "null"] },
          web_search_result: { type: ["string", "null"] },
          source_url: { type: ["string", "null"] },
          first_seen_at: { type: "number" },
          occurrences: { type: "integer", minimum: 1 },
        },
      },
    },
    summary: {
      type: "object",
      required: ["text", "updated_at", "covering_from_ms", "covering_to_ms"],
      properties: {
        text: { type: "string" },
        updated_at: { type: "number" },
        covering_from_ms: { type: "number" },
        covering_to_ms: { type: "number" },
      },
    },
    action_items: {
      type: "array",
      items: {
        type: "object",
        required: ["id", "text", "priority", "completed", "detected_at"],
        properties: {
          id: { type: "string" },
          text: { type: "string" },
          assignee: { type: ["string", "null"] },
          deadline: { type: ["string", "null"] },
          priority: { type: "string", enum: ["High", "Medium", "Low"] },
          completed: { type: "boolean" },
          detected_at: { type: "number" },
        },
      },
    },
    decisions: {
      type: "array",
      items: {
        type: "object",
        required: ["id", "text", "context", "participants", "detected_at"],
        properties: {
          id: { type: "string" },
          text: { type: "string" },
          context: { type: "string" },
          participants: { type: "array", items: { type: "string" } },
          detected_at: { type: "number" },
        },
      },
    },
  },
};

/** System prompt for unified meeting analysis. */
const SYSTEM_PROMPT = `You are a meeting analysis AI. You receive transcript text and extract structured insights in a single pass.

Rules:
- Extract ONLY what is explicitly stated or clearly implied in the transcript.
- For keywords: identify technical terms, proper nouns, acronyms, and jargon. Use WebSearch to look up definitions for unfamiliar terms if possible.
- For summary: write a concise, informative summary of the discussion content.
- For action items: extract tasks with assignees, deadlines, and priority (High/Medium/Low). Only mark items explicitly assigned or volunteered.
- For decisions: extract decisions with the context that led to them and who participated.
- Generate unique IDs using the format: kw-<index>, ai-<index>, d-<index>.
- All timestamps should use the segment times from the transcript.
- Return valid JSON matching the required schema exactly.`;

/**
 * Build the unified analysis prompt.
 * @param {object} payload - { transcript_text, meeting_topic?, existing_keywords?, context_summary? }
 * @param {number} fromMs - Start time of the batch window
 * @param {number} toMs - End time of the batch window
 * @returns {{ system: string, user: string }}
 */
export function buildAnalysisBatchPrompt(payload, fromMs = 0, toMs = 0) {
  const { transcript_text, meeting_topic, existing_keywords, context_summary } = payload;

  const parts = [];

  if (meeting_topic) {
    parts.push(`Meeting topic: ${meeting_topic}`);
  }
  if (context_summary) {
    parts.push(`Previous context (compressed summary of earlier discussion):\n${context_summary}`);
  }
  if (existing_keywords && existing_keywords.length > 0) {
    parts.push(`Already-extracted keywords (increment occurrences if seen again, do not create new entries): ${existing_keywords.join(", ")}`);
  }

  parts.push(`Analysis window: ${fromMs}ms to ${toMs}ms`);
  parts.push(`\nTranscript:\n---\n${transcript_text}\n---`);
  parts.push(`\nExtract keywords, summary, action items, and decisions from this transcript segment. Return JSON only.`);

  return { system: SYSTEM_PROMPT, user: parts.join("\n") };
}

/**
 * Build the investigation prompt.
 * @param {object} payload - { query, context }
 * @returns {{ system: string, user: string }}
 */
export function buildInvestigatePrompt(payload) {
  const { query, context } = payload;
  const system = `You are a research assistant helping during a live meeting. Search the web for relevant information and provide concise, actionable results.`;
  const user = `Context from the meeting:\n${context}\n\nResearch query: ${query}\n\nSearch the web and provide:\n1. A concise summary (2-3 sentences)\n2. Detailed findings\n3. Sources with titles and URLs\n\nReturn JSON: { "summary": "...", "details": "...", "sources": [{"title": "...", "url": "..."}] }`;
  return { system, user };
}

/**
 * Build the minutes generation prompt.
 * @param {object} payload
 * @returns {{ system: string, user: string }}
 */
export function buildMinutesPrompt(payload) {
  const { transcript_text, keywords, action_items, decisions, summary } = payload;
  const system = `You are a meeting minutes generator. Create well-structured, professional meeting minutes in Markdown format.`;
  const parts = [`Transcript:\n${transcript_text}`];
  if (summary) parts.push(`Summary: ${summary}`);
  if (keywords?.length > 0) parts.push(`Key terms: ${keywords.join(", ")}`);
  if (action_items?.length > 0) parts.push(`Action items: ${JSON.stringify(action_items)}`);
  if (decisions?.length > 0) parts.push(`Decisions: ${JSON.stringify(decisions)}`);
  parts.push(`\nGenerate comprehensive meeting minutes in Markdown. Return JSON: { "markdown": "..." }`);
  return { system, user: parts.join("\n\n") };
}
