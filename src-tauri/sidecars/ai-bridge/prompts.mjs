/** JSON Schema for the unified analysis response. */
export const ANALYSIS_SCHEMA = {
  type: "object",
  required: ["keywords", "summary", "action_items", "decisions", "topics"],
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
          /** 担当者名（日本語の氏名も対応） */
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
          /** 決定に至った経緯・背景（稟議・合意形成等の日本的会議スタイルも考慮） */
          context: { type: "string" },
          participants: { type: "array", items: { type: "string" } },
          detected_at: { type: "number" },
        },
      },
    },
    topics: {
      type: "array",
      items: {
        type: "object",
        required: ["id", "title", "start_ms", "end_ms"],
        properties: {
          id: { type: "string" },
          title: { type: "string" },
          summary: { type: "string" },
          start_ms: { type: "number" },
          end_ms: { type: "number" },
        },
      },
    },
  },
};

/** System prompt for unified meeting analysis (Japanese meeting optimized). */
const SYSTEM_PROMPT = `あなたは日本語会議の分析AIです。文字起こしテキストを受け取り、構造化されたインサイトを一度に抽出してJSONで返します。

## 基本ルール
- トランスクリプトに明示的に述べられているか、明らかに示唆されていることのみ抽出する
- 推測や補完は行わない
- 有効なJSONをスキーマ通りに返す

## 日本語処理
- 敬語表現（です・ます調、だ・である調の混在）を正しく理解する
- 句読点が少ない・文境界が曖昧な日本語トランスクリプトの特性に対応する
- カタカナ語・英語混じりの専門用語はそのまま正確に保持する
- 話し言葉特有の省略・倒置・言い直しを適切に解釈する

## キーワード抽出
- 技術用語、固有名詞、略語、業界用語を抽出する
- 不明な用語はWebSearchで日本語検索して定義を調べる（検索クエリも日本語で）
- IDは kw-<インデックス> 形式で生成する

## サマリー
- 議論内容を簡潔かつ情報量豊かにまとめる
- 日本語で記述する

## アクションアイテム
- 明示的に割り当てられた・自発的に引き受けたタスクのみ抽出する
- 担当者名は日本語の氏名もそのまま使用する
- 優先度はHigh/Medium/Lowで分類する
- IDは ai-<インデックス> 形式で生成する

## 決定事項
- 会議内で決まった事項を抽出する
- contextには稟議・合意形成・多数決など日本的会議スタイルの決定経緯も含める
- IDは d-<インデックス> 形式で生成する

## トピック
- 会議全体を意味のある話題・議題単位に分割する
- 各トピックのタイトルは日本語で簡潔に記述する
- 時間範囲はトランスクリプトのセグメントタイムスタンプを使用する
- IDは t-<インデックス> 形式で生成する`;

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
    parts.push(`会議テーマ: ${meeting_topic}`);
  }
  if (context_summary) {
    parts.push(`前回までの要約（以前の議論の圧縮サマリー）:\n${context_summary}`);
  }
  if (existing_keywords && existing_keywords.length > 0) {
    parts.push(
      `既出キーワード（再出現時はoccurrencesを加算し、新規エントリは作成しない）: ${existing_keywords.join(", ")}`
    );
  }

  parts.push(`分析ウィンドウ: ${fromMs}ms ～ ${toMs}ms`);
  parts.push(`\nトランスクリプト:\n---\n${transcript_text}\n---`);
  parts.push(
    `\nこのトランスクリプトセグメントからキーワード、サマリー、アクションアイテム、決定事項、トピックを抽出してください。JSONのみ返してください。`
  );

  return { system: SYSTEM_PROMPT, user: parts.join("\n") };
}

/**
 * Build the investigation prompt.
 * @param {object} payload - { query, context }
 * @returns {{ system: string, user: string }}
 */
export function buildInvestigatePrompt(payload) {
  const { query, context } = payload;
  const system = `あなたは会議中にリアルタイムでサポートするリサーチアシスタントです。Webを検索して関連情報を調べ、簡潔で実用的な結果を提供してください。検索クエリは日本語でも積極的に使用してください。`;
  const user = `会議のコンテキスト:\n${context}\n\nリサーチクエリ: ${query}\n\nWebを検索して以下を提供してください:\n1. 簡潔なサマリー（2〜3文）\n2. 詳細な調査結果\n3. 参照元（タイトルとURL）\n\nJSON形式で返してください: { "summary": "...", "details": "...", "sources": [{"title": "...", "url": "..."}] }`;
  return { system, user };
}

/**
 * Build the minutes generation prompt.
 * @param {object} payload - { transcript_text, keywords?, action_items?, decisions?, summary? }
 * @returns {{ system: string, user: string }}
 */
export function buildMinutesPrompt(payload) {
  const { transcript_text, keywords, action_items, decisions, summary } = payload;
  const system = `あなたは議事録作成AIです。日本語の会議議事録として、日時・出席者・議題・議事内容・決定事項・次回アクションの形式に沿ったMarkdownを生成してください。敬体（です・ます調）で記述し、専門用語はそのまま保持してください。`;
  const parts = [`トランスクリプト:\n${transcript_text}`];
  if (summary) parts.push(`サマリー: ${summary}`);
  if (keywords?.length > 0) parts.push(`キーワード: ${keywords.join(", ")}`);
  if (action_items?.length > 0) parts.push(`アクションアイテム: ${JSON.stringify(action_items)}`);
  if (decisions?.length > 0) parts.push(`決定事項: ${JSON.stringify(decisions)}`);
  parts.push(
    `\n上記をもとに日本語の議事録をMarkdownで作成してください。JSON形式で返してください: { "markdown": "..." }`
  );
  return { system, user: parts.join("\n\n") };
}

/**
 * Build the transcript formatting prompt.
 * @param {object} payload - { segments: Array<{speaker, text, start_time, end_time}>, previous_formatted?: string }
 * @returns {{ system: string, user: string }}
 */
export function buildFormatTranscriptPrompt(payload) {
  const { segments, previous_formatted } = payload;

  const system = `あなたは会議の文字起こしを読みやすい議事録テキストにフォーマットするAIです。

## フォーマット規則
- 断片的な発話を自然な文章に繋げる
- フィラー語（えーと、あの、まあ、その、なんか等）を適切に除去する
- 話者ごとに段落を分ける
- 各段落の先頭にタイムスタンプを [HH:MM] 形式で付与する
- 繰り返しや言い直しを整理して簡潔にする
- 専門用語・固有名詞・カタカナ語はそのまま保持する
- 日本語メインだが英語混じりの発話にも対応する
- 発話の意図や内容を変えない

## 出力形式
JSON形式で返してください: { "formatted_text": "...", "last_segment_end_ms": 1234.5 }
- formatted_text: フォーマット済みテキスト（previous_formattedがある場合はそれに追記）
- last_segment_end_ms: 処理した最後のセグメントのend_time（ミリ秒）`;

  const segmentsText = segments
    .map(seg => {
      const startSec = Math.floor((seg.start_time ?? 0) / 1000);
      const mm = Math.floor(startSec / 60).toString().padStart(2, "0");
      const ss = (startSec % 60).toString().padStart(2, "0");
      const speaker = seg.speaker ? `[${seg.speaker}]` : "";
      return `[${mm}:${ss}] ${speaker} ${seg.text}`.trim();
    })
    .join("\n");

  const parts = [];
  if (previous_formatted) {
    parts.push(`前回のフォーマット済みテキスト（このテキストに続けて追記してください）:\n---\n${previous_formatted}\n---`);
  }
  parts.push(`新しいセグメント:\n---\n${segmentsText}\n---`);
  parts.push(`\n上記のセグメントをフォーマットしてJSONで返してください。`);

  return { system, user: parts.join("\n\n") };
}

/**
 * Build the suggest questions prompt.
 * @param {object} payload - { transcript_text, summary?, topics? }
 * @returns {{ system: string, user: string }}
 */
export function buildSuggestQuestionsPrompt(payload) {
  const { transcript_text, summary, topics } = payload;

  const system = `あなたは会議をより実りあるものにするためのAIアシスタントです。会議の文脈を理解し、参加者が「聞いておくべき質問」を3〜5個提案してください。

## 質問提案の基準
- 議論が不明瞭なまま進んでいる点を明確にする質問
- 決定前に確認すべきリスクや前提条件に関する質問
- 見落とされている重要な観点や関係者への質問
- アクションアイテムの実行可能性を高める質問
- 次のステップを明確にする質問

## 出力形式
JSON形式で返してください: { "questions": [{ "id": "q-1", "text": "...", "reason": "..." }] }
- id: q-<インデックス> 形式
- text: 質問文（日本語、敬体）
- reason: この質問を提案する理由（1〜2文）`;

  const parts = [`トランスクリプト:\n${transcript_text}`];
  if (summary) {
    parts.push(`サマリー: ${summary}`);
  }
  if (topics && topics.length > 0) {
    parts.push(`トピック一覧: ${JSON.stringify(topics)}`);
  }
  parts.push(`\n会議の文脈から「聞いておくべき質問」を3〜5個提案してください。JSONのみ返してください。`);

  return { system, user: parts.join("\n\n") };
}

/**
 * Build the translation prompt.
 * @param {object} payload - { text, target_lang }
 * @returns {{ system: string, user: string }}
 */
export function buildTranslatePrompt(payload) {
  const { text, target_lang } = payload;

  const system = `あなたは高精度な翻訳AIです。会議・ビジネス文書の翻訳を専門とし、文脈を保ちながら自然な翻訳を行います。専門用語・固有名詞はそのまま保持してください。`;
  const user = `以下のテキストを ${target_lang} に翻訳してください。\n\n原文:\n${text}\n\nJSON形式で返してください: { "translated_text": "..." }`;

  return { system, user };
}
