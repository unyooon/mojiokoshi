# Claude API統合とインテリジェント機能 設計書

## 1. キーワード・話題の自動検出機能

### 1.1 アーキテクチャ概要

```
[Whisperリアルタイム文字起こし]
        │
        ▼
[テキストバッファ（セグメント単位）]
        │
        ├──→ [リアルタイムパイプライン] ──→ Claude Haiku 4.5（軽量キーワード抽出）
        │                                        │
        │                                        ▼
        │                               [キーワードキャッシュ]
        │                                        │
        │                                        ▼
        │                               [Web検索（Brave Search API）]
        │                                        │
        │                                        ▼
        │                               [補足情報パネル表示]
        │
        └──→ [バッチパイプライン（5分ごと）] ──→ Claude Sonnet 4.5（深い分析）
                                                  │
                                                  ▼
                                          [話題トピック分析]
                                          [要約・アクションアイテム]
```

### 1.2 リアルタイムキーワード抽出プロンプト

```
System Prompt:
あなたは会議の文字起こしテキストからキーワードを抽出する専門家です。
以下のカテゴリに分類してJSON形式で出力してください。

カテゴリ:
- technical_terms: 技術用語、専門用語
- proper_nouns: 固有名詞（人名、組織名、製品名、サービス名）
- abbreviations: 略語・頭字語
- key_topics: 重要な話題・テーマ
- action_words: アクションに関連するキーワード

User Prompt:
会議コンテキスト: {meeting_context}
前回抽出済みキーワード: {previous_keywords}

以下の新しい発言テキストから、まだ抽出されていない重要なキーワードを抽出してください。
既に抽出済みのキーワードは除外してください。

テキスト:
{new_transcript_segment}

出力形式（JSON）:
{
  "keywords": [
    {
      "word": "キーワード",
      "category": "カテゴリ",
      "context": "使用された文脈（短い説明）",
      "confidence": 0.0-1.0,
      "needs_lookup": true/false
    }
  ]
}
```

### 1.3 処理フロー

**リアルタイム処理（Haiku 4.5使用）:**
- Whisperから文字起こしセグメントが到着するたびに実行
- デバウンス: 最低3秒間隔（連続した短い発言を束ねる）
- 直前のコンテキスト（過去5発言分）を含めて送信
- 抽出済みキーワードリストを毎回添付し重複排除
- レスポンスタイム目標: 1秒以内

**バッチ処理（Sonnet 4.5使用・5分ごと）:**
- 過去5分間のトランスクリプト全体を分析
- 話題の転換点を検出
- キーワード間の関連性を分析
- より深い文脈理解に基づくキーワード精緻化

### 1.4 リアルタイム vs バッチのトレードオフ

| 項目 | リアルタイム (Haiku) | バッチ (Sonnet) |
|------|---------------------|-----------------|
| レイテンシ | ~500ms | ~3-5秒 |
| 精度 | 中（表面的） | 高（文脈考慮） |
| コスト/呼出 | ~$0.0001-0.001 | ~$0.001-0.01 |
| 用途 | 即座のキーワード表示 | 話題分析・要約 |
| モデル | Haiku 4.5 | Sonnet 4.5 |
| 頻度 | セグメント到着ごと | 5分ごと |

**推奨: ハイブリッドアプローチ**
- リアルタイム: Haikuで即座にキーワード表示（ユーザー体験重視）
- バッチ: Sonnetで定期的に精緻化（品質重視）
- バッチ結果でリアルタイム結果を補完・修正

### 1.5 Web検索による補足情報取得

**Brave Search API統合:**
```typescript
interface KeywordSearchConfig {
  // 検索対象キーワードのフィルタ条件
  minConfidence: 0.7;           // 信頼度閾値
  categories: ['technical_terms', 'proper_nouns', 'abbreviations'];
  needsLookup: true;            // Claude が検索推奨したもの

  // Brave Search API設定
  braveApiKey: string;
  maxQueriesPerMinute: 10;      // レート制限
  resultCount: 3;               // 各キーワードの検索結果数
}
```

**Claude組み込みWeb検索ツール（代替案）:**
- Anthropic APIの `web_search` ツールを使用する方式
- Claude自身が検索の必要性を判断し自動実行
- $10 / 1,000検索 + 通常トークンコスト
- 検索結果にはソース引用が自動付与
- ドメイン許可/ブロックリストで制御可能

**推奨: 二段構え**
1. 基本はBrave Search API直接呼び出し（コスト効率・制御性）
2. ユーザーの手動調査依頼時はClaudeのweb_searchツール使用（品質・文脈理解）

---

## 2. ユーザー選択ワードのClaude調査機能

### 2.1 UIフロー

```
[ユーザーがテキストを選択]
        │
        ▼
[コンテキストメニュー表示]
  ├── 「Claudeで調査」
  ├── 「この用語を説明」
  ├── 「関連情報を検索」
  └── 「翻訳する」
        │
        ▼
[調査リクエスト生成]
  - 選択テキスト
  - 前後の文脈（±3発言）
  - 会議の話題コンテキスト
  - 会議メタデータ（参加者、議題等）
        │
        ▼
[Claude Sonnet 4.5 + web_search ツール]
        │
        ▼
[サイドパネルに結果表示]
```

### 2.2 調査用プロンプト

```
System Prompt:
あなたは会議支援アシスタントです。ユーザーが会議中に選択したテキストについて、
会議の文脈を考慮した調査・解説を行います。

回答は以下の構造で提供してください：
1. 簡潔な定義・説明（1-2文）
2. 会議文脈での関連性
3. 詳細情報（必要に応じて）
4. 関連リンク・参考資料

User Prompt:
## 調査対象
選択テキスト: 「{selected_text}」

## 会議コンテキスト
会議タイトル: {meeting_title}
現在の話題: {current_topic}
発言の前後文脈:
{surrounding_context}

## 調査タイプ: {investigation_type}
- explain: 用語の意味・解説
- research: 詳細な調査・背景情報
- search: Web上の最新情報検索
- translate: 翻訳

選択テキストについて、会議の文脈を考慮した上で{investigation_type}を行ってください。
```

### 2.3 結果表示方式

```typescript
interface InvestigationResult {
  query: string;                  // 調査対象テキスト
  type: 'explain' | 'research' | 'search' | 'translate';
  summary: string;                // 1-2文の要約
  details: string;                // マークダウン形式の詳細
  sources?: {                     // Web検索ソース
    title: string;
    url: string;
    snippet: string;
  }[];
  relatedKeywords?: string[];     // 関連キーワード
  timestamp: Date;
}
```

**表示UI:**
- サイドパネル（右側）にカード形式で表示
- ピン留め可能（会議後も参照可能）
- 調査履歴のタイムライン表示
- コピー・共有機能

---

## 3. MTG中に便利なAI機能

### 3.1 機能一覧と優先度

| 機能 | 優先度 | 使用モデル | 処理タイミング |
|------|--------|------------|---------------|
| リアルタイム要約 | P0 | Haiku 4.5 | 発言ごと or 2分ごと |
| アクションアイテム抽出 | P0 | Sonnet 4.5 | 5分ごと + 会議終了時 |
| 決定事項の自動記録 | P0 | Sonnet 4.5 | 5分ごと + 会議終了時 |
| 議事録の自動生成 | P0 | Sonnet 4.5 | 会議終了時 |
| キーワード自動検出 | P1 | Haiku 4.5 | リアルタイム |
| 話題タイムライン | P1 | Haiku 4.5 | トピック変化時 |
| 質問候補の提案 | P2 | Haiku 4.5 | 適宜 |
| 翻訳（多言語対応） | P2 | Haiku 4.5 | リアルタイム |
| 発言の感情/トーン分析 | P3 | Haiku 4.5 | 発言ごと |
| FAQ/関連ドキュメント表示 | P3 | Sonnet 4.5 | トピック変化時 |
| 前回会議との関連性 | P3 | Sonnet 4.5 | 会議開始時 + 適宜 |

### 3.2 リアルタイム要約

**処理方式: ローリングサマリー**

```
[新しい発言セグメント]
        │
        ▼
[現在の要約 + 新セグメント] ──→ Claude Haiku 4.5
        │
        ▼
[更新された要約]
```

**プロンプト:**
```
System Prompt:
あなたは会議のリアルタイム要約アシスタントです。
現在の要約と新しい発言を受け取り、要約を更新してください。

ルール:
- 要約は箇条書き形式
- 最大10項目（古い項目は統合）
- 各項目は1行以内
- 話題ごとにグループ化
- 重要度の高い情報を優先

User Prompt:
## 現在の要約
{current_summary}

## 新しい発言
話者: {speaker}
時刻: {timestamp}
内容: {text}

要約を更新してください。変更がない場合は現在の要約をそのまま返してください。
```

### 3.3 アクションアイテム抽出

**プロンプト:**
```
System Prompt:
あなたは会議のアクションアイテムを抽出する専門家です。
発言内容から明示的・暗黙的なアクションアイテムを抽出してください。

User Prompt:
## 会議トランスクリプト（直近5分間）
{transcript_segment}

## 既に抽出済みのアクションアイテム
{existing_action_items}

新たなアクションアイテムを以下のJSON形式で抽出してください：
{
  "action_items": [
    {
      "description": "アクション内容",
      "assignee": "担当者名（不明ならnull）",
      "deadline": "期限（言及があれば）",
      "priority": "high/medium/low",
      "source_quote": "根拠となる発言の引用",
      "timestamp": "発言時刻"
    }
  ]
}
```

### 3.4 決定事項の自動記録

**プロンプト:**
```
System Prompt:
あなたは会議での決定事項を検出・記録する専門家です。
合意・承認・決定を示す発言パターンを認識してください。

検出パターン:
- 明示的決定: 「〜に決まりました」「〜で行きましょう」「承認します」
- 合意形成: 「では〜ということで」「異論なければ〜」
- 方針決定: 「〜の方向で進めます」「〜を採用します」

User Prompt:
## 会議トランスクリプト（直近5分間）
{transcript_segment}

## 既に記録済みの決定事項
{existing_decisions}

新たな決定事項を以下のJSON形式で出力してください：
{
  "decisions": [
    {
      "description": "決定内容",
      "context": "背景・理由",
      "participants": ["関与者"],
      "timestamp": "決定時刻",
      "source_quote": "根拠となる発言"
    }
  ]
}
```

### 3.5 議事録の自動生成（会議終了時）

**プロンプト:**
```
System Prompt:
あなたは会議議事録の作成アシスタントです。
会議全体のトランスクリプトと中間分析結果を基に、構造化された議事録を生成します。

User Prompt:
## 会議情報
タイトル: {meeting_title}
日時: {date_time}
参加者: {participants}
時間: {duration}

## 会議トランスクリプト
{full_transcript}

## 中間分析結果
要約: {rolling_summary}
アクションアイテム: {action_items}
決定事項: {decisions}
キーワード: {keywords}
話題タイムライン: {topic_timeline}

以下の形式で議事録を生成してください：

# 議事録

## 基本情報
- 日時・参加者・時間

## エグゼクティブサマリー
（3-5文で会議全体を要約）

## 議論内容
### トピック1: {topic_name}
- 議論の要点
- 発言者と主な主張

### トピック2: ...

## 決定事項
（番号付きリスト）

## アクションアイテム
| 項目 | 担当 | 期限 | 優先度 |
|------|------|------|--------|

## 次回に向けて
（未解決事項、次回の議題候補）
```

### 3.6 話題タイムライン

**処理方式:**
- Haikuで発言ごとに現在の話題を判定
- 話題変化を検出したらタイムラインに追加
- UIにタイムラインバーとして表示

```typescript
interface TopicSegment {
  topic: string;
  startTime: Date;
  endTime?: Date;
  keywords: string[];
  speakerDistribution: Record<string, number>; // 話者ごとの発言割合
}
```

### 3.7 質問候補の提案

- 会議の文脈から、議論が不十分な点や曖昧な点を検出
- 参加者が質問すべき候補を提案
- Haikuで軽量に処理、3-5個の質問を常時表示

### 3.8 翻訳（多言語会議対応）

- Haikuで文字起こしテキストをリアルタイム翻訳
- 対応言語: 日本語 <-> 英語 を主要ターゲット
- 原文と翻訳を並列表示

### 3.9 発言の感情/トーン分析

- 各発言の感情トーンを分析（ポジティブ/ニュートラル/ネガティブ/緊張等）
- 会議全体の感情トレンドを可視化
- チーム雰囲気のモニタリング

---

## 4. Claude API設計

### 4.1 モデル使い分け戦略

```
┌─────────────────────────────────────────────────────┐
│                  モデル選択マトリクス                    │
├──────────────┬──────────────┬───────────────────────┤
│ Haiku 4.5    │ Sonnet 4.5   │ Opus 4.5/4.6         │
│ $1/$5 per M  │ $3/$15 per M │ $5/$25 per M         │
├──────────────┼──────────────┼───────────────────────┤
│ キーワード抽出│ アクション    │ （基本不使用）          │
│ リアルタイム  │  アイテム抽出 │ 超複雑な調査時のみ      │
│  要約        │ 決定事項記録  │ Fallbackとして         │
│ 話題検出     │ 議事録生成    │                       │
│ 翻訳         │ 深い調査      │                       │
│ 感情分析     │ バッチ分析    │                       │
│ 質問候補     │ Web検索付き   │                       │
│              │  調査         │                       │
└──────────────┴──────────────┴───────────────────────┘
```

**判定ロジック:**
```typescript
function selectModel(task: AITask): ModelId {
  switch (task.type) {
    // リアルタイム・低レイテンシ要求
    case 'keyword_extraction':
    case 'rolling_summary':
    case 'topic_detection':
    case 'translation':
    case 'sentiment_analysis':
    case 'question_suggestions':
      return 'claude-haiku-4-5-20251001';

    // 高品質・複雑な分析
    case 'action_item_extraction':
    case 'decision_recording':
    case 'meeting_minutes':
    case 'deep_investigation':
    case 'batch_analysis':
      return 'claude-sonnet-4-5-20250929';

    // 最高品質（ほぼ使わない）
    case 'complex_research':
      return 'claude-opus-4-6';

    default:
      return 'claude-haiku-4-5-20251001';
  }
}
```

### 4.2 ストリーミングAPI活用

**SSE（Server-Sent Events）ベースのストリーミング:**

```typescript
import Anthropic from '@anthropic-ai/sdk';

const client = new Anthropic({ apiKey: process.env.ANTHROPIC_API_KEY });

async function streamKeywordExtraction(
  transcript: string,
  context: MeetingContext
): AsyncGenerator<KeywordResult> {
  const stream = await client.messages.stream({
    model: 'claude-haiku-4-5-20251001',
    max_tokens: 1024,
    system: KEYWORD_EXTRACTION_SYSTEM_PROMPT,
    messages: [
      {
        role: 'user',
        content: buildKeywordPrompt(transcript, context)
      }
    ]
  });

  let buffer = '';
  for await (const event of stream) {
    if (event.type === 'content_block_delta' &&
        event.delta.type === 'text_delta') {
      buffer += event.delta.text;

      // JSONの完全なオブジェクトが来たら即座にyield
      const parsed = tryParsePartialJSON(buffer);
      if (parsed) {
        yield parsed;
        buffer = '';
      }
    }
  }
}
```

**ストリーミングの活用ポイント:**
- リアルタイム要約: テキストが生成されるたびにUIを更新
- キーワード抽出: JSONオブジェクト単位でインクリメンタルに表示
- 調査結果: マークダウンをストリーミング表示
- 議事録生成: セクション単位でプログレッシブに表示

### 4.3 コンテキストウィンドウ管理（長時間会議対応）

**戦略: 三層コンテキスト管理**

```
┌─────────────────────────────────────────┐
│  Layer 1: System Prompt（固定・キャッシュ） │
│  - 役割定義、出力形式                      │
│  - 会議メタデータ                          │
│  - プロンプトキャッシュで90%コスト削減       │
│  ※ cache_control ブレークポイント設定      │
├─────────────────────────────────────────┤
│  Layer 2: ローリングコンテキスト           │
│  - 直近N分のトランスクリプト               │
│  - 現在の要約・キーワード・決定事項         │
│  - コンパクション機能で自動管理             │
├─────────────────────────────────────────┤
│  Layer 3: フルアーカイブ（ローカル保存）     │
│  - 全トランスクリプト                      │
│  - 全分析結果                             │
│  - 会議終了時の議事録生成に使用             │
└─────────────────────────────────────────┘
```

**コンテキストウィンドウ使用量の目安:**

| 会議時間 | 推定トークン数 | 管理戦略 |
|----------|---------------|---------|
| ~30分 | ~15,000 | そのまま全文送信 |
| 30-60分 | ~30,000 | ローリングウィンドウ（直近15分 + 要約） |
| 1-2時間 | ~60,000 | コンパクション活用 |
| 2時間超 | ~120,000+ | コンパクション + 要約チェーン |

**コンパクション設定:**
```typescript
const messageRequest = {
  model: 'claude-sonnet-4-5-20250929',
  max_tokens: 4096,
  // サーバーサイドコンパクション有効化
  context_management: {
    edits: [{ strategy: 'compact_20260112' }],
    // コンパクション発動閾値
    trigger: { input_tokens: 150000 }
  },
  system: [
    {
      type: 'text',
      text: SYSTEM_PROMPT,
      cache_control: { type: 'ephemeral' } // プロンプトキャッシュ
    }
  ],
  messages: conversationHistory
};
```

**ローリングウィンドウ実装:**
```typescript
class MeetingContextManager {
  private fullTranscript: TranscriptSegment[] = [];
  private rollingSummary: string = '';
  private activeKeywords: Keyword[] = [];
  private actionItems: ActionItem[] = [];
  private decisions: Decision[] = [];

  // APIに送信するコンテキストを構築
  buildContext(windowMinutes: number = 15): string {
    const cutoff = Date.now() - windowMinutes * 60 * 1000;
    const recentSegments = this.fullTranscript
      .filter(s => s.timestamp.getTime() > cutoff);

    return JSON.stringify({
      rolling_summary: this.rollingSummary,
      active_keywords: this.activeKeywords.slice(-30),
      action_items: this.actionItems,
      decisions: this.decisions,
      recent_transcript: recentSegments
    });
  }
}
```

### 4.4 コスト最適化戦略

**1. プロンプトキャッシュ（最大90%削減）**
- System Promptを `cache_control: { type: 'ephemeral' }` でキャッシュ
- 5分間のデフォルトTTLで連続呼び出しに対応
- キャッシュ読み取り: 通常の0.1倍コスト

**2. バッチAPI（50%削減）**
- 会議終了後の議事録生成
- 非リアルタイムの分析タスク
- 単一リクエストでも50%割引

**3. モデル階層化**
- リアルタイム処理: Haiku ($1/$5) — コスト最小
- 品質要求処理: Sonnet ($3/$15) — コスパ最良
- Opusは基本不使用

**4. トークン最適化**
- レスポンスはJSON形式で簡潔に
- 不要なテキストを含めないプロンプト設計
- max_tokensを適切に制限

**コスト試算（1時間の会議）:**

| 処理 | モデル | 呼び出し回数 | 推定コスト |
|------|--------|-------------|-----------|
| キーワード抽出 | Haiku | ~60回 | ~$0.06 |
| リアルタイム要約 | Haiku | ~30回 | ~$0.03 |
| 話題検出 | Haiku | ~60回 | ~$0.03 |
| アクションアイテム | Sonnet | ~12回 | ~$0.12 |
| 決定事項記録 | Sonnet | ~12回 | ~$0.12 |
| 議事録生成 | Sonnet (Batch) | 1回 | ~$0.05 |
| **合計** | | | **~$0.41** |

※プロンプトキャッシュ適用後。キャッシュなしの場合は約$1.50-2.00。

### 4.5 レート制限対策

**Tier別の制限と対策:**

| Tier | RPM | 対策 |
|------|-----|------|
| Tier 1 | 50 RPM | 開発初期はこれで十分 |
| Tier 2 | 1,000 RPM | 本番運用目標 |
| Tier 4 | 4,000 RPM | 大規模展開時 |

**実装パターン:**

```typescript
class RateLimitedClient {
  private queue: RequestQueue;
  private tokenBucket: TokenBucket;

  constructor(private config: RateLimitConfig) {
    this.tokenBucket = new TokenBucket({
      rpm: config.requestsPerMinute,
      itpm: config.inputTokensPerMinute,
      otpm: config.outputTokensPerMinute
    });
    this.queue = new RequestQueue();
  }

  async request(params: MessageParams): Promise<MessageResponse> {
    // トークンバケットの空きを確認
    await this.tokenBucket.acquire(
      estimateTokens(params)
    );

    try {
      return await this.client.messages.create(params);
    } catch (error) {
      if (error.status === 429) {
        // レート制限ヒット: 指数バックオフ
        const retryAfter = error.headers['retry-after'] || 60;
        await sleep(retryAfter * 1000);
        return this.request(params); // リトライ
      }
      throw error;
    }
  }
}
```

**キャッシュトークンの活用:**
- プロンプトキャッシュのキャッシュヒットトークンはITPM制限にカウントされない
- これにより実効スループットが5-10倍に向上

**優先度ベースのキューイング:**
```typescript
enum RequestPriority {
  CRITICAL = 0,    // ユーザー起点の調査（即座に応答必要）
  HIGH = 1,        // リアルタイム要約
  MEDIUM = 2,      // キーワード抽出
  LOW = 3,         // バッチ分析
  BACKGROUND = 4   // 感情分析等
}
```

---

## 5. プロンプトテンプレート設計

### 5.1 設計原則

1. **構造化出力**: 全てJSON形式で出力を要求
2. **インクリメンタル処理**: 差分のみを返すよう設計
3. **コンテキスト最小化**: 必要最小限の情報のみプロンプトに含める
4. **日本語最適化**: 日本語のビジネス会議に特化した指示
5. **キャッシュ親和性**: System Promptを固定化し、変動部分はUser Promptに

### 5.2 共通System Prompt（キャッシュ対象）

```
あなたは日本語の会議を支援するAIアシスタント「Mojiokoshi」です。

## 基本ルール
- 出力は常にJSON形式
- 日本語で応答
- 簡潔かつ正確に
- 推測は confidence を下げて明示
- 不明な場合は null を返す

## 会議コンテキスト
この会議の情報:
- タイトル: {meeting_title}
- 参加者: {participants}
- 議題: {agenda}
- 開始時刻: {start_time}
```

### 5.3 各機能のプロンプトテンプレート一覧

（セクション1-3で詳述済み。ここでは全体構造のみ）

```typescript
const PROMPT_TEMPLATES = {
  // リアルタイム処理（Haiku用）
  realtime: {
    keyword_extraction: '...', // セクション1.2参照
    rolling_summary: '...',    // セクション3.2参照
    topic_detection: '...',
    translation: '...',
    sentiment_analysis: '...',
    question_suggestions: '...',
  },

  // バッチ処理（Sonnet用）
  batch: {
    action_items: '...',       // セクション3.3参照
    decisions: '...',          // セクション3.4参照
    meeting_minutes: '...',    // セクション3.5参照
    deep_investigation: '...', // セクション2.2参照
    batch_analysis: '...',
  }
} as const;
```

---

## 6. 技術実装アーキテクチャ

### 6.1 全体フロー

```
┌──────────────────────────────────────────────────────────────┐
│                    Electron Main Process                      │
│                                                              │
│  ┌─────────────┐    ┌──────────────────────────────────┐    │
│  │ Audio Engine │───▶│ Whisper Transcription Engine     │    │
│  └─────────────┘    └──────────┬───────────────────────┘    │
│                                │                             │
│                                ▼                             │
│  ┌─────────────────────────────────────────────────────┐    │
│  │           AI Processing Pipeline                     │    │
│  │                                                     │    │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────┐  │    │
│  │  │ Rate Limiter  │  │ Prompt Cache │  │  Model   │  │    │
│  │  │ & Queue       │  │ Manager      │  │ Selector │  │    │
│  │  └──────┬───────┘  └──────┬───────┘  └────┬─────┘  │    │
│  │         │                  │                │        │    │
│  │         ▼                  ▼                ▼        │    │
│  │  ┌──────────────────────────────────────────────┐   │    │
│  │  │         Claude API Client (Streaming)         │   │    │
│  │  └──────────────────────────────────────────────┘   │    │
│  │         │              │              │              │    │
│  │         ▼              ▼              ▼              │    │
│  │  [Haiku Tasks]  [Sonnet Tasks]  [Web Search]        │    │
│  └─────────────────────────────────────────────────────┘    │
│                                │                             │
│                                ▼                             │
│  ┌─────────────────────────────────────────────────────┐    │
│  │              Result Aggregator & Store                │    │
│  │  (keywords, summary, actions, decisions, timeline)    │    │
│  └──────────────────────┬──────────────────────────────┘    │
│                          │ IPC                               │
└──────────────────────────┼───────────────────────────────────┘
                           │
                           ▼
┌──────────────────────────────────────────────────────────────┐
│                   Electron Renderer Process                    │
│                                                              │
│  ┌──────────┐ ┌─────────┐ ┌──────────┐ ┌────────────────┐  │
│  │Transcript│ │Keywords │ │ Summary  │ │ Investigation  │  │
│  │  Panel   │ │ Panel   │ │ Panel    │ │ Side Panel     │  │
│  └──────────┘ └─────────┘ └──────────┘ └────────────────┘  │
└──────────────────────────────────────────────────────────────┘
```

### 6.2 APIクライアント設計

```typescript
// src/services/claude/ClaudeService.ts
class ClaudeService {
  private client: Anthropic;
  private rateLimiter: RateLimitedClient;
  private promptCache: PromptCacheManager;
  private contextManager: MeetingContextManager;

  constructor(config: ClaudeConfig) {
    this.client = new Anthropic({ apiKey: config.apiKey });
    this.rateLimiter = new RateLimitedClient(config.rateLimit);
    this.promptCache = new PromptCacheManager();
    this.contextManager = new MeetingContextManager();
  }

  // リアルタイムキーワード抽出
  async extractKeywords(segment: TranscriptSegment): Promise<Keyword[]> { ... }

  // リアルタイム要約更新
  async updateSummary(segment: TranscriptSegment): Promise<string> { ... }

  // アクションアイテム抽出（バッチ）
  async extractActionItems(segments: TranscriptSegment[]): Promise<ActionItem[]> { ... }

  // ユーザー調査依頼
  async investigate(query: InvestigationQuery): Promise<InvestigationResult> { ... }

  // 議事録生成
  async generateMinutes(meeting: MeetingData): Promise<MeetingMinutes> { ... }
}
```

### 6.3 エラーハンドリングとフォールバック

```typescript
class AIProcessingPipeline {
  async process(segment: TranscriptSegment): Promise<void> {
    // 各処理を並列実行（独立したタスク）
    const tasks = [
      this.safeExecute('keywords', () => this.claude.extractKeywords(segment)),
      this.safeExecute('summary', () => this.claude.updateSummary(segment)),
      this.safeExecute('topic', () => this.claude.detectTopic(segment)),
    ];

    await Promise.allSettled(tasks);
  }

  private async safeExecute(name: string, fn: () => Promise<any>): Promise<void> {
    try {
      const result = await fn();
      this.emit(`${name}:update`, result);
    } catch (error) {
      if (error.status === 429) {
        // レート制限: キューに入れてリトライ
        this.queue.add(name, fn, RequestPriority.HIGH);
      } else if (error.status === 500 || error.status === 503) {
        // サーバーエラー: 指数バックオフでリトライ
        await this.retryWithBackoff(name, fn);
      } else {
        // その他: ログ記録、UIにはエラー表示しない（グレースフルデグレード）
        console.error(`AI processing error (${name}):`, error);
      }
    }
  }
}
```

---

## 7. セキュリティ考慮事項

### 7.1 APIキー管理
- Electron Main Process内でのみAPIキーを保持
- Renderer Processには公開しない
- OS のキーチェーン/資格情報マネージャーに保存
- 環境変数 or 設定ファイル（暗号化）

### 7.2 データプライバシー
- 会議内容がClaude APIに送信されることをユーザーに明示
- オフラインモード対応（AI機能なしでの基本文字起こし）
- 送信するテキストの最小化（必要な文脈のみ）
- ローカルでの分析結果保持（外部サーバーに保存しない）

### 7.3 コスト管理
- 月額上限設定機能
- リアルタイム使用量表示
- コスト超過時の自動停止 or グレースフルデグレード

---

## 8. 開発ロードマップ

### Phase 1（MVP）
- [ ] Claude API基本統合（Haiku/Sonnet）
- [ ] リアルタイムキーワード抽出
- [ ] ローリング要約
- [ ] 議事録自動生成

### Phase 2
- [ ] ユーザー選択テキストの調査機能
- [ ] アクションアイテム・決定事項抽出
- [ ] 話題タイムライン
- [ ] Web検索統合

### Phase 3
- [ ] 翻訳機能
- [ ] 質問候補提案
- [ ] 感情分析
- [ ] 前回会議との関連性
- [ ] FAQ/関連ドキュメント表示
