import { createInterface } from "node:readline";
import {
  ANALYSIS_SCHEMA,
  buildAnalysisBatchPrompt,
  buildFormatTranscriptPrompt,
  buildInvestigatePrompt,
  buildMinutesPrompt,
  buildSuggestQuestionsPrompt,
  buildTranslatePrompt,
} from "./prompts.mjs";

const IS_STUB = process.env.MOJIOKOSHI_AI_STUB === "1";

/** Send a JSON-line response to stdout. */
function respond(response) {
  process.stdout.write(JSON.stringify(response) + "\n");
}

/** Return stub/mock data without calling the API. */
function stubResponse(command) {
  const { id, type } = command;
  if (type === "analyze_batch") {
    return {
      id, type: "response",
      payload: {
        keywords: [],
        summary: { text: "スタブサマリー（テスト用）", updated_at: Date.now(), covering_from_ms: 0, covering_to_ms: 0 },
        action_items: [],
        decisions: [],
        topics: [],
      },
    };
  }
  if (type === "investigate") {
    return {
      id, type: "response",
      payload: {
        id: `inv-${id}`, query: command.payload?.query ?? "",
        summary: "スタブ調査結果（テスト用）",
        details: "スタブモードのため実際の調査は行われていません。",
        sources: [], created_at: Date.now(),
      },
    };
  }
  if (type === "generate_minutes") {
    return {
      id, type: "response",
      payload: { markdown: "# 議事録（スタブ）\n\nスタブモードのためコンテンツは生成されていません。" },
    };
  }
  if (type === "format_transcript") {
    const segments = command.payload?.segments ?? [];
    const joined = segments.map(seg => seg.text ?? "").join(" ");
    const lastEndMs = segments.length > 0
      ? (segments[segments.length - 1].end_time ?? 0)
      : 0;
    const previous = command.payload?.previous_formatted ?? "";
    const formatted_text = previous ? `${previous}\n${joined}` : joined;
    return {
      id, type: "response",
      payload: { formatted_text, last_segment_end_ms: lastEndMs },
    };
  }
  if (type === "suggest_questions") {
    return {
      id, type: "response",
      payload: {
        questions: [
          { id: "q-1", text: "この決定事項の実施担当者は誰ですか？", reason: "スタブモードのダミー質問です。" },
          { id: "q-2", text: "次回のマイルストーンはいつを想定していますか？", reason: "スタブモードのダミー質問です。" },
          { id: "q-3", text: "リスクや懸念事項はありますか？", reason: "スタブモードのダミー質問です。" },
        ],
      },
    };
  }
  if (type === "translate") {
    return {
      id, type: "response",
      payload: { translated_text: `[スタブ翻訳] ${command.payload?.text ?? ""}` },
    };
  }
  return { id, type: "error", payload: { message: `Unknown command type: ${type}` } };
}

/** Handle a single command by calling the Claude Agent SDK. */
async function handleCommand(command) {
  const { id, type, payload } = command;

  if (IS_STUB) {
    respond(stubResponse(command));
    return;
  }

  let promptParts;
  let allowedTools;
  let outputFormat;

  if (type === "analyze_batch") {
    promptParts = buildAnalysisBatchPrompt(
      payload,
      payload.from_ms ?? 0,
      payload.to_ms ?? 0,
    );
    allowedTools = ["WebSearch", "WebFetch"];
    outputFormat = { type: "json_schema", schema: ANALYSIS_SCHEMA };
  } else if (type === "investigate") {
    promptParts = buildInvestigatePrompt(payload);
    allowedTools = ["WebSearch", "WebFetch"];
  } else if (type === "generate_minutes") {
    promptParts = buildMinutesPrompt(payload);
    allowedTools = [];
  } else if (type === "format_transcript") {
    promptParts = buildFormatTranscriptPrompt(payload);
    allowedTools = [];
  } else if (type === "suggest_questions") {
    promptParts = buildSuggestQuestionsPrompt(payload);
    allowedTools = [];
  } else if (type === "translate") {
    promptParts = buildTranslatePrompt(payload);
    allowedTools = [];
  } else {
    respond({ id, type: "error", payload: { message: `Unknown command type: ${type}` } });
    return;
  }

  try {
    const { query } = await import("@anthropic-ai/claude-agent-sdk");
    const options = {
      prompt: promptParts.user,
      systemPrompt: promptParts.system,
      allowedTools,
      permissionMode: "bypassPermissions",
      maxTurns: 3,
    };
    if (outputFormat) {
      options.outputFormat = outputFormat;
    }
    const result = await query(options);

    const text = result.result ?? "";
    let parsed;
    try {
      parsed = JSON.parse(text);
    } catch {
      const jsonMatch = text.match(/```(?:json)?\s*\n?([\s\S]*?)\n?```/);
      if (jsonMatch) {
        parsed = JSON.parse(jsonMatch[1].trim());
      } else {
        parsed = { raw_text: text };
      }
    }
    respond({ id, type: "response", payload: parsed });
  } catch (err) {
    respond({ id, type: "error", payload: { message: err.message ?? String(err) } });
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
    respond({ id: "unknown", type: "error", payload: { message: `Invalid JSON: ${err.message}` } });
    return;
  }
  handleCommand(command).catch((err) => {
    respond({
      id: command.id ?? "unknown", type: "error",
      payload: { message: `Unhandled error: ${err.message ?? String(err)}` },
    });
  });
});

rl.on("close", () => {
  process.exit(0);
});
