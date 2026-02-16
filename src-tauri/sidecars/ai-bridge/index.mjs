import { createInterface } from "node:readline";

const IS_STUB = process.env.MOJIOKOSHI_AI_STUB === "1";

/**
 * Send a JSON-line response to stdout.
 * @param {object} response
 */
function respond(response) {
  process.stdout.write(JSON.stringify(response) + "\n");
}

/**
 * Return stub/mock data for testing without calling the API.
 * @param {object} command
 * @returns {object}
 */
function stubResponse(command) {
  const { id, type } = command;

  if (type === "analyze_batch") {
    return {
      id,
      type: "response",
      payload: {
        keywords: [],
        summary: {
          text: "Stub summary for testing.",
          updated_at: Date.now(),
          covering_from_ms: 0,
          covering_to_ms: 0,
        },
        action_items: [],
        decisions: [],
      },
    };
  }

  if (type === "investigate") {
    return {
      id,
      type: "response",
      payload: {
        id: `inv-${id}`,
        query: command.payload?.query ?? "",
        summary: "Stub investigation result.",
        details: "No real investigation performed in stub mode.",
        sources: [],
        created_at: Date.now(),
      },
    };
  }

  if (type === "generate_minutes") {
    return {
      id,
      type: "response",
      payload: {
        markdown: "# Meeting Minutes (Stub)\n\nNo content generated in stub mode.",
      },
    };
  }

  return {
    id,
    type: "error",
    payload: { message: `Unknown command type: ${type}` },
  };
}

/**
 * Build the prompt for analyze_batch.
 * @param {object} payload
 * @returns {string}
 */
function buildAnalyzeBatchPrompt(payload) {
  const { transcript_text, meeting_topic, existing_keywords } = payload;
  const topicLine = meeting_topic
    ? `Meeting topic: ${meeting_topic}\n`
    : "";
  const existingLine =
    existing_keywords && existing_keywords.length > 0
      ? `Already-known keywords (do not re-extract): ${existing_keywords.join(", ")}\n`
      : "";

  return `You are an AI assistant analyzing a meeting transcript.
${topicLine}${existingLine}
Analyze the following transcript text and extract:
1. **Keywords**: technical terms, proper nouns, acronyms, and jargon with definitions
2. **Summary**: a concise summary of the discussion
3. **Action items**: tasks assigned to people with deadlines and priority
4. **Decisions**: decisions made during the meeting with context

Transcript:
---
${transcript_text}
---

Return a JSON object with this exact structure:
{
  "keywords": [{"id": "string", "term": "string", "keyword_type": "TechTerm"|"ProperNoun"|"Acronym"|"Jargon", "definition": "string|null", "web_search_result": "string|null", "source_url": "string|null", "first_seen_at": 0, "occurrences": 1}],
  "summary": {"text": "string", "updated_at": ${Date.now()}, "covering_from_ms": 0, "covering_to_ms": 0},
  "action_items": [{"id": "string", "text": "string", "assignee": "string|null", "deadline": "string|null", "priority": "High"|"Medium"|"Low", "completed": false, "detected_at": 0}],
  "decisions": [{"id": "string", "text": "string", "context": "string", "participants": ["string"], "detected_at": 0}]
}`;
}

/**
 * Build the prompt for investigate.
 * @param {object} payload
 * @returns {string}
 */
function buildInvestigatePrompt(payload) {
  const { query, context } = payload;
  return `You are a research assistant. A user is in a meeting and wants to investigate a topic.

Context from the meeting:
${context}

Research query: ${query}

Search the web for relevant information and provide:
1. A concise summary
2. Detailed findings
3. Sources with titles and URLs

Return a JSON object:
{
  "summary": "string",
  "details": "string",
  "sources": [{"title": "string", "url": "string"}]
}`;
}

/**
 * Build the prompt for generate_minutes.
 * @param {object} payload
 * @returns {string}
 */
function buildMinutesPrompt(payload) {
  const { transcript_text, keywords, action_items, decisions, summary } =
    payload;
  return `You are a meeting minutes generator. Create well-structured meeting minutes in Markdown.

Transcript:
${transcript_text}

${summary ? `Summary so far: ${summary}` : ""}
${keywords && keywords.length > 0 ? `Key terms discussed: ${keywords.join(", ")}` : ""}
${action_items && action_items.length > 0 ? `Action items: ${JSON.stringify(action_items)}` : ""}
${decisions && decisions.length > 0 ? `Decisions: ${JSON.stringify(decisions)}` : ""}

Generate comprehensive meeting minutes in Markdown format.
Return a JSON object: { "markdown": "string" }`;
}

/**
 * Handle a single command by calling the Claude Agent SDK.
 * @param {object} command
 */
async function handleCommand(command) {
  const { id, type, payload } = command;

  if (IS_STUB) {
    respond(stubResponse(command));
    return;
  }

  let prompt;
  let allowedTools;

  if (type === "analyze_batch") {
    prompt = buildAnalyzeBatchPrompt(payload);
    allowedTools = ["WebSearch", "WebFetch"];
  } else if (type === "investigate") {
    prompt = buildInvestigatePrompt(payload);
    allowedTools = ["WebSearch", "WebFetch"];
  } else if (type === "generate_minutes") {
    prompt = buildMinutesPrompt(payload);
    allowedTools = [];
  } else {
    respond({
      id,
      type: "error",
      payload: { message: `Unknown command type: ${type}` },
    });
    return;
  }

  try {
    const { query } = await import("@anthropic-ai/claude-agent-sdk");
    const result = await query({
      prompt,
      allowedTools,
      permissionMode: "bypassPermissions",
      maxTurns: 3,
      options: {
        maxThinkingTokens: 1024,
      },
    });

    // Extract JSON from the result text
    const text = result.result ?? "";
    let parsed;
    try {
      // Try to parse the entire text as JSON
      parsed = JSON.parse(text);
    } catch {
      // Try to extract JSON from markdown code block
      const jsonMatch = text.match(/```(?:json)?\s*\n?([\s\S]*?)\n?```/);
      if (jsonMatch) {
        parsed = JSON.parse(jsonMatch[1].trim());
      } else {
        parsed = { raw_text: text };
      }
    }

    respond({ id, type: "response", payload: parsed });
  } catch (err) {
    respond({
      id,
      type: "error",
      payload: { message: err.message ?? String(err) },
    });
  }
}

// Main: read JSON-lines from stdin
const rl = createInterface({ input: process.stdin });

rl.on("line", (line) => {
  const trimmed = line.trim();
  if (!trimmed) return;

  let command;
  try {
    command = JSON.parse(trimmed);
  } catch (err) {
    respond({
      id: "unknown",
      type: "error",
      payload: { message: `Invalid JSON: ${err.message}` },
    });
    return;
  }

  handleCommand(command).catch((err) => {
    respond({
      id: command.id ?? "unknown",
      type: "error",
      payload: { message: `Unhandled error: ${err.message ?? String(err)}` },
    });
  });
});

rl.on("close", () => {
  process.exit(0);
});
