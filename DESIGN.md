# MojiOkoshi - リアルタイム会議文字起こし＋AI分析システム 設計書

## 1. システム概要

### 1.1 コンセプト
Macローカルで動作するリアルタイム会議支援アプリケーション。Teams/Zoom等のWeb会議の音声をキャプチャし、Whisperでローカル文字起こしを行い、Claude Code（Maxサブスクリプション）によるインテリジェント分析を提供する。

### 1.3 コスト方針
**Claudeサブスクリプション（Max plan）以外の追加課金は一切発生しない構成とする。**

| コンポーネント | コスト | 備考 |
|---|---|---|
| Claude（AI分析・Web検索） | **Maxサブスク内** | Claude Code SDK経由。追加API課金なし |
| 音声キャプチャ | 無料 | ScreenCaptureKit（macOS標準） |
| 音声認識 | 無料 | whisper.cpp / mlx-whisper（OSS） |
| VAD | 無料 | Silero VAD（OSS） |
| 話者分離 | 無料 | pyannote.audio（OSS） |
| Web検索 | 無料 | Claude Code内蔵WebSearch（サブスク内） |
| UI | 無料 | Tauri + React（OSS） |
| DB | 無料 | SQLite（OSS） |

### 1.2 主要機能
1. **リアルタイム文字起こし表示** - 話者分離・タイムスタンプ付き
2. **キーワード自動検出＋Web調査** - 専門用語・固有名詞を自動検出し、補足情報を取得
3. **オンデマンドClaude調査** - 選択したワードや話題をClaudeに調査依頼
4. **MTG支援AI機能** - 要約、アクションアイテム抽出、議事録自動生成等

---

## 2. アーキテクチャ全体図

```
┌─────────────────────────────────────────────────────────────────┐
│                        MojiOkoshi App                          │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │               Frontend (Tauri v2 + React 19)              │  │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌────────────┐  │  │
│  │  │文字起こし │ │AIインサイト│ │調査パネル │ │ タイムライン │  │  │
│  │  │  パネル   │ │  パネル   │ │          │ │            │  │  │
│  │  └──────────┘ └──────────┘ └──────────┘ └────────────┘  │  │
│  └──────────────────────┬───────────────────────────────────┘  │
│                         │ Tauri Commands / Events / Channel     │
│  ┌──────────────────────┴───────────────────────────────────┐  │
│  │                   Backend (Rust Core)                      │  │
│  │                                                            │  │
│  │  ┌────────────┐  ┌────────────┐  ┌─────────────────────┐ │  │
│  │  │Audio Capture│  │  Whisper    │  │  Claude Code SDK    │ │  │
│  │  │  Engine     │──▶│  Engine     │──▶│  (Node.js)         │ │  │
│  │  │(ScreenCap- │  │(whisper.cpp │  │  + 内蔵WebSearch    │ │  │
│  │  │ tureKit /  │  │ / mlx)     │  │  (Maxサブスク内)     │ │  │
│  │  │ CoreAudio  │  │            │  │                     │ │  │
│  │  │ Taps)      │  │            │  │                     │ │  │
│  │  └────────────┘  └────────────┘  └─────────────────────┘ │  │
│  │                                                            │  │
│  │  ┌────────────┐  ┌────────────┐                           │  │
│  │  │  Session    │  │  Export     │                           │  │
│  │  │  Storage    │  │  Engine     │                           │  │
│  │  │  (SQLite)   │  │  (MD/PDF)   │                           │  │
│  │  └────────────┘  └────────────┘                           │  │
│  └──────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
         │                    │                    │
         ▼                    ▼                    ▼
   ┌──────────┐       ┌──────────────────────────────┐
   │ System   │       │ Anthropic Cloud              │
   │ Audio    │       │ (Claude Max サブスク内)        │
   │ (macOS)  │       │  - AI分析                    │
   └──────────┘       │  - WebSearch                 │
                      │  ※ 追加課金なし               │
                      └──────────────────────────────┘
```

### 2.1 データフローパイプライン

```
音声入力 ──▶ 音声バッファ ──▶ VAD検出 ──▶ Whisper推論 ──▶ テキスト出力
                                                              │
                                              ┌───────────────┤
                                              ▼               ▼
                                        UI表示更新      Claude分析キュー
                                                              │
                                              ┌───────────────┤
                                              ▼               ▼
                                      キーワード抽出    リアルタイム要約
                                              │
                                              ▼
                                        Web検索/調査
                                              │
                                              ▼
                                     補足情報をUI表示
```

---

## 3. コンポーネント詳細設計

### 3.1 音声キャプチャエンジン

#### 推奨方式: ScreenCaptureKit + Core Audio Taps ハイブリッド

| 方式 | セットアップ | 安定性 | レイテンシー | アプリ単体キャプチャ | マイク同時キャプチャ | 音声のみ権限 | macOS対応 |
|------|-------------|--------|-------------|-------------------|-------------------|------------|----------|
| **ScreenCaptureKit** | 不要（OS標準） | ◎ | 低 | ◎ | ◎（macOS 15+） | ×（画面録画権限） | 12.3+ |
| **Core Audio Taps** | 不要（OS標準） | ◎ | 非常に低 | ◎（PID指定） | 別途実装要 | ◎ | 14.4+ |
| BlackHole | 要インストール | ○ | ゼロ | × | 別途実装要 | 不要 | 10.9+ |
| Loopback | 要購入($99) | ◎ | 低 | ◎（GUI） | ◎（GUI） | 不要 | 14.5+ |
| Soundflower | **非推奨** | △ | - | × | × | - | レガシー |

#### macOSバージョン別の推奨実装戦略

```
macOS 15+   → ScreenCaptureKit（マイク同時キャプチャ: captureMicrophone = true）
macOS 14.4+ → Core Audio Taps（音声のみ権限で動作）+ AVAudioEngine（マイク）
macOS 13+   → ScreenCaptureKit（システム音声）+ AVAudioEngine（マイク）
macOS 12.3+ → ScreenCaptureKit（基本音声）+ AVAudioEngine（マイク）
```

#### ScreenCaptureKit（メイン方式）の詳細

**選定理由：**
- macOS標準APIのためサードパーティ依存なし
- `SCContentFilter`によるアプリ単体の音声キャプチャが可能（Teams/Zoomだけをキャプチャ）
- `SCStreamConfiguration`で音声フォーマット（サンプルレート、チャンネル数）を柔軟に設定可能
- macOS 15以降では`captureMicrophone`プロパティでマイクも同時キャプチャ可能
- Apple Siliconに最適化済み

**macOSバージョン別の機能追加：**

| macOS | 追加機能 |
|-------|---------|
| 12.3 (Monterey) | 基本的な画面・音声キャプチャ |
| 13.0 (Ventura) | 音声同期クロック、改善されたAPI |
| 14.0 (Sonoma) | コンテンツピッカー、音声のみサブ権限 |
| 15.0 (Sequoia) | **マイクキャプチャ対応**（`captureMicrophone`）、HDR、レコーディング出力 |

```swift
// ScreenCaptureKit による音声キャプチャ（macOS 15+対応）
import ScreenCaptureKit

class AudioCaptureEngine {
    private var stream: SCStream?

    func startCapture(for app: SCRunningApplication) async throws {
        let content = try await SCShareableContent.current
        let filter = SCContentFilter(desktopIndependentWindow: /* ... */)

        let config = SCStreamConfiguration()
        config.capturesAudio = true
        config.sampleRate = 16000        // Whisper推奨サンプルレート
        config.channelCount = 1           // モノラル
        config.captureMicrophone = true   // macOS 15+: マイクも同時キャプチャ

        stream = SCStream(filter: filter, configuration: config, delegate: self)
        try stream?.addStreamOutput(self, type: .audio, sampleHandlerQueue: .global())
        try stream?.addStreamOutput(self, type: .microphone, sampleHandlerQueue: .global())
        try await stream?.startCapture()
    }
}
```

#### Core Audio Taps（補助方式 / macOS 14.4+）

**主要API:**
- `CATapDescription` - タップの設定（ターゲットプロセス、排他設定等）
- `AudioHardwareCreateProcessTap` - プロセスタップの作成
- `AudioHardwareCreateAggregateDevice` - アグリゲートデバイスの構築

**利点:** 音声のみの権限で動作（画面録画権限不要）、プロセス単位の精密なキャプチャ
**注意:** ドキュメントが乏しく、実装が複雑

#### macOSセキュリティ要件
- **画面収録権限**（ScreenCaptureKit使用時）: `System Preferences > Privacy & Security > Screen Recording`
- **音声キャプチャ権限**（Core Audio Taps使用時）: `NSAudioCaptureUsageDescription`をInfo.plistに記載
- **マイクアクセス**: `Info.plist`に`NSMicrophoneUsageDescription`を記載
- 初回起動時にユーザーへ権限許可を促すオンボーディングフローを実装

#### フォールバック: BlackHole
- macOS 12.3未満のユーザー向け
- `brew install blackhole-2ch`で簡単にインストール可能
- 仮想オーディオデバイスとしてシステム音声全体をルーティング
- アプリ単体のフィルタリングは不可

#### 参考実装
- [screencapturekit-rs](https://github.com/svtlabs/screencapturekit-rs) - Rustバインディング
- [AudioCap](https://github.com/insidegui/AudioCap) - Core Audio Tapsサンプル実装
- [Azayaka](https://github.com/Mnpn/Azayaka) - ScreenCaptureKit実装例

---

### 3.2 Whisper音声認識エンジン

#### 推奨構成: whisper.cpp（Tauri統合時） / mlx-whisper（Python統合時）

| ライブラリ | 言語 | Apple Silicon最適化 | リアルタイム対応 | 速度（largeモデル） | メモリ |
|-----------|------|-------------------|----------------|-------------------|--------|
| **whisper.cpp** | C++ | ◎ (CoreML/Metal) | ◎ | 1.23秒 | 低 |
| **mlx-whisper** | Python (MLX) | ◎ (MLXネイティブ) | ◎ | 1.02秒 | 中 |
| WhisperKit | Swift/CoreML | ◎ (ANE最大活用) | ◎ | 0.19秒(CoreML版) | 中 |
| faster-whisper | Python (CTranslate2) | ○ | ○ | - | 中 |
| OpenAI API | クラウド | N/A | △ | N/A | N/A |

**whisper.cpp を第一推奨とする理由（Tauri/Rust構成の場合）：**
- Rustバインディング（`whisper-rs`）が成熟しておりTauriバックエンドと親和性が高い
- CoreML変換モデルによりApple Neural Engine/Metal GPUに対応
- メモリフットプリントが小さく、会議アプリと共存可能
- C++ネイティブ実装で最小のオーバーヘッド

**mlx-whisper も有力な選択肢：**
- whisper.cppの2〜3倍高速（Apple MLXフレームワークネイティブ）
- Pythonスクリプトでリアルタイム文字起こしプログラムを直接構築可能
- Tauriバックエンドからサブプロセスとして呼び出す構成

#### モデル選定（日本語）

| モデル | パラメータ数 | メモリ | 日本語精度 | 推奨用途 |
|-------|------------|-------|----------|---------|
| tiny | 39M | ~1GB | × 実用困難 | テスト用 |
| base | 74M | ~1GB | △ | 軽量環境 |
| small | 244M | ~2GB | ○ | バランス型（M1） |
| medium | 769M | ~5GB | ◎ | ビジネス用途 |
| **large-v3-turbo** | 809M | ~5GB | ◎ large-v2相当・8倍高速 | **推奨デフォルト** |
| large-v3 | 1.5B | ~10GB | ◎+ | 最高精度（M3 Pro以上推奨） |

**推奨: `large-v3-turbo` をデフォルト、設定で切替可能**
- large-v2と同等の精度を保ちつつ8倍高速
- ~5GBメモリでM1以上なら十分動作可能
- Apple Silicon統合メモリによりGPUメモリ不足の問題なし

**日本語特化モデル（オプション選択肢）：**
- **kotoba-whisper-v1.0**: large-v3より良好なCER/WERを達成、large-v3の6.3倍高速
- 日本語主体の会議で特に有効

#### リアルタイム処理パイプライン

```
音声ストリーム
    │
    ▼
┌──────────────────┐
│ Ring Buffer       │  ← 連続的に音声データを蓄積
│ (30秒分保持)      │
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│ VAD (Silero VAD)  │  ← 音声区間を検出（発話の開始/終了を判定）
│ 1.8MB / ~1ms処理  │     TPR 87.7%（FPR 5%時）
└────────┬─────────┘
         │ 発話区間を切り出し
         ▼
┌──────────────────┐
│ Whisper推論        │  ← スライディングウィンドウ + Local Agreement Policy
│ (whisper.cpp/mlx) │     確定テキスト + 部分テキストを出力
└────────┬─────────┘
         │
         ├──▶ 確定テキスト → UIに追加表示
         │
         └──▶ 部分テキスト → UIにプレビュー表示（グレーアウト）
```

**処理パラメータ:**
- バッファサイズ: 30秒のリングバッファ
- VAD閾値: 0.5（調整可能）
- 最小発話長: 0.5秒
- 最大発話長: 30秒（超えたら強制区切り）
- end-to-endレイテンシ目標: 380〜520ms

**Local Agreement Policy:**
- 2つの連続更新間の合意により確定テキストを出力
- スライディングウィンドウのチャンクサイズの約2倍のレイテンシ
- Levenshtein距離ベースの合意アルゴリズムで最も正確な転写を抽出

#### 話者分離（Diarization）

- **pyannote.audio 4.0 + community-1** ベースの話者分離
- WhisperX方式: Whisper文字起こしセグメントとpyannote話者セグメントの時間的交差に基づくアラインメント
- マイク入力（自分の声）をアンカーとして話者を識別
- DER (Diarization Error Rate): 11〜19%（標準ベンチマーク）
- 最初は「自分」と「相手」の2者分離から始め、段階的に多話者対応

#### 将来的な選択肢: Apple SpeechAnalyzer
- WWDC25で発表、macOS Tahoe / iOS 26以降
- Whisper Large V3 Turboより55%高速（34分動画を45秒で処理）
- Neural Engine完全活用、OS標準APIで追加依存なし
- macOS Tahoe普及後にネイティブAPI移行を検討

---

### 3.3 Claude Code SDK 統合 & インテリジェント機能

#### 3.3.0 Claude Code SDK による統合方式

**従来のClaude API（従量課金）ではなく、Claude Code SDK（Maxサブスク内）を使用する。**

```typescript
import { Claude } from "@anthropic-ai/claude-code";

// アプリ起動時に1つのインスタンスを生成し、会議中ずっと使い回す
const claude = new Claude({
  model: "sonnet",  // Max planで利用可能なモデル
});

// 文字起こし結果をまとめて分析依頼
const result = await claude.sendMessage(`
  以下の直近3分間の会議テキストを分析してください:
  1. 重要キーワードを抽出（JSON形式）
  2. 要約を生成
  3. アクションアイテムがあれば抽出
  4. 不明な専門用語があればWebSearchで調査

  テキスト: ${transcriptChunk}
`);
```

**利点：**
- Maxサブスクリプション内で追加課金なし
- Web検索もClaude Code内蔵のWebSearchで対応（別途API契約不要）
- モデル選択不要（Maxプランで利用可能なモデルを自動使用）

**設計上の考慮点：**

| 項目 | 対策 |
|------|------|
| レート制限 | バッチ処理（3分単位でまとめて送信）で呼び出し回数を抑制 |
| レイテンシー | Claude Codeインスタンスを常駐させ、起動オーバーヘッドを排除 |
| 並行処理 | キーワード抽出+要約+アクションアイテムを1回のプロンプトに統合 |
| オフライン | AI機能なしの基本文字起こしモードにフォールバック |

#### 3.3.1 統合プロンプト戦略

**API従量課金と異なり、サブスク内で利用するため、機能ごとにモデルを分ける必要がない。
1回のリクエストで複数タスクをまとめて処理する「統合プロンプト」方式を採用する。**

```
┌──────────────────────────────────────────────────┐
│            Claude Code SDK 統合プロンプト           │
│                                                    │
│  従来（API従量課金）:                               │
│    キーワード抽出 → Haiku  (1回目の呼び出し)        │
│    要約生成       → Haiku  (2回目の呼び出し)        │
│    アクション抽出 → Sonnet (3回目の呼び出し)        │
│    Web検索        → Brave  (別途API課金)            │
│    = 3回のAPI呼び出し + 外部API課金                  │
│                                                    │
│  新方式（Claude Code SDK）:                         │
│    全タスク統合   → Claude Code (1回の呼び出し)      │
│    Web検索        → 内蔵WebSearch (サブスク内)       │
│    = 1回の呼び出し、追加課金なし                     │
└──────────────────────────────────────────────────┘
```

#### 3.3.2 キーワード自動検出＋Web調査

```
文字起こしテキスト（3分間隔でバッチ送信）
    │
    ▼
┌──────────────────────┐
│ Claude Code SDK       │  ← 統合プロンプトで一括処理
│                       │     - キーワード抽出
│ 1回の呼び出しで:       │     - Web検索（内蔵WebSearch）
│ ・キーワード抽出       │     - 補足情報生成
│ ・要約生成             │     - 要約
│ ・アクションアイテム    │     - アクションアイテム抽出
│ ・Web調査（必要時）     │
│                       │     ※ 全てMaxサブスク内
└────────┬─────────────┘
         │ JSON構造化レスポンス
         ▼
┌──────────────────────┐
│ 既知キーワードDB確認    │  ← SQLiteローカルキャッシュで重複排除
└────────┬─────────────┘
         │
         ▼
  UIに反映（ハイライト + ポップオーバー + サイドパネル更新）
```

**Web検索は全てClaude Code内蔵のWebSearchを使用（追加API契約不要）。**

**キーワード抽出プロンプト:**

```
あなたは会議の文字起こしテキストから重要なキーワードを抽出するアシスタントです。

以下のテキストから、参加者が理解を深めるために調査すべきキーワードを抽出してください。

## すでに抽出済みのキーワード（重複排除用）
{already_extracted_keywords}

## 抽出対象
- 技術用語・専門用語（例: RAG, LLM, Kubernetes）
- 固有名詞（企業名、製品名、人名）
- 略語・頭字語
- 業界特有の用語

## 除外対象
- 一般的な日常語
- すでに抽出済みのキーワード
- すでに文脈から意味が明らかな語

## 出力形式（JSON配列）
[
  {
    "word": "キーワード",
    "type": "tech_term | proper_noun | acronym | industry_term",
    "context": "会議での使用文脈（短い抜粋）",
    "priority": "high | medium | low"
  }
]

## テキスト
{transcribed_text}
```

#### 3.3.3 オンデマンドClaude調査機能

ユーザーがテキストを選択し、右クリック（`⌘+I`）で調査を依頼する機能。

**調査プロンプト（Claude Code内蔵WebSearch付き）:**

```
あなたは会議中のリアルタイムリサーチアシスタントです。
必要に応じてWebSearchで最新情報を調査してください。

## 会議の文脈
{meeting_context_summary}

## 最近の発言（直近5分）
{recent_transcript}

## 調査対象
「{selected_text}」

上記の会議の文脈を踏まえて、選択されたテキストについて以下を調査・説明してください：

1. **概要**: 選択されたワード/話題の簡潔な説明
2. **会議での関連性**: この会議の文脈でどう関係するか
3. **補足情報**: 知っておくと有用な追加情報
4. **関連リンク**: 参考になるリソース（もしあれば）

回答は簡潔に、会議中にサッと読める長さでお願いします。
```

#### 3.3.4 MTG支援AI機能一覧

全てClaude Code SDK経由（Maxサブスク内）で処理。統合プロンプトで複数機能を1回の呼び出しにまとめる。

| # | 機能 | 説明 | トリガー | 処理方式 | 優先度 |
|---|------|------|---------|---------|--------|
| 1 | **リアルタイム要約** | 3分ごとに直近の議論を要約 | 自動 | 統合プロンプト（3分バッチ） | P0 |
| 2 | **アクションアイテム抽出** | 行動指示を自動検出 | 自動 | 統合プロンプト（3分バッチ） | P0 |
| 3 | **決定事項記録** | 決定事項を自動検出 | 自動 | 統合プロンプト（3分バッチ） | P0 |
| 4 | **議事録自動生成** | 会議終了後に全体議事録をMarkdownで生成 | 会議終了時 | 単独呼び出し | P0 |
| 5 | **キーワード自動検出** | 専門用語・固有名詞を検出し内蔵WebSearchで調査 | 自動 | 統合プロンプト（3分バッチ） | P1 |
| 6 | **話題タイムライン** | 議論された話題を時系列で整理 | 自動 | 統合プロンプト（3分バッチ） | P1 |
| 7 | **質問候補提案** | 「聞いておくべき質問」を提案 | オンデマンド | 単独呼び出し | P2 |
| 8 | **用語集自動生成** | 専門用語の一覧＋解説を生成 | 会議終了時 | 単独呼び出し | P2 |
| 9 | **多言語翻訳** | 英語↔日本語のリアルタイム翻訳表示 | 設定で有効化 | 単独呼び出し | P2 |
| 10 | **感情・トーン分析** | 議論のテンション変化を可視化 | 自動 | 統合プロンプト（3分バッチ） | P3 |
| 11 | **前回会議リンク** | 過去会議を自動リンク | 自動 | ローカルDB検索 | P3 |
| 12 | **FAQ/関連ドキュメント表示** | 関連ドキュメントを提示 | 自動 | 単独呼び出し | P3 |

#### 3.3.5 コンテキストウィンドウ管理

長時間会議（1時間以上）での二層管理戦略：

```
┌─────────────────────────────────────────────────┐
│           コンテキスト管理戦略（二層）              │
│                                                  │
│  Layer 1: ローリングコンテキスト（Claude Codeへ送信）│
│  ├── 直近15分の詳細テキスト（全文保持）             │
│  ├── 15〜60分前は5分単位の要約に圧縮               │
│  ├── 60分以上前は会議全体の要約 + キーワード一覧    │
│  │                                               │
│  Layer 2: フルアーカイブ（ローカルSQLite保存）       │
│  └── 全文字起こしテキストをローカルに保存           │
│      議事録生成時に参照                            │
└─────────────────────────────────────────────────┘
```

#### 3.3.6 コスト

**Claude Maxサブスクリプション以外のコストは発生しない。**

| 項目 | コスト |
|------|--------|
| Claude Code SDK（AI分析全般） | Maxサブスク内 |
| Web検索（内蔵WebSearch） | Maxサブスク内 |
| Whisper（ローカル推論） | 無料（OSS） |
| 音声キャプチャ | 無料（OS標準） |
| その他全コンポーネント | 無料（OSS） |
| **合計** | **Maxサブスク料金のみ** |

#### 3.3.7 レート制限対策

Claude Maxプランにはメッセージ数の使用量上限があるため、以下の対策を実施：

| 対策 | 説明 |
|------|------|
| **統合プロンプト** | キーワード抽出+要約+アクションアイテムを1回の呼び出しに統合（呼び出し回数を1/3〜1/5に削減） |
| **3分バッチ** | 3分間の文字起こしをまとめて送信（1時間の会議で約20回の呼び出し） |
| **キーワードキャッシュ** | SQLiteで既出キーワードを管理し、重複調査を排除 |
| **優先度キュー** | ユーザー操作（⌘+I調査） > 自動バッチ処理 の優先度で処理 |
| **オフラインフォールバック** | 上限到達時はAI機能なしの基本文字起こしモードに移行 |

---

### 3.4 UI/UXデザイン

#### 3.4.1 技術選定: Tauri v2 + React + TypeScript

| 技術 | 選定理由 |
|------|---------|
| **Tauri v2** | バンドルサイズ~5MB、メモリ~30-40MB、起動<0.5秒。Rustバックエンドでwhisper.cppと直接連携。2024年v2安定版リリース |
| **React 19** | エコシステム充実、コンポーネント設計に適合 |
| **TypeScript 5** | 型安全性、大規模コードベースの保守性 |
| **Tailwind CSS v4** | 高速なUI開発、ダークモード対応が容易 |
| **shadcn/ui** | 高品質なUIコンポーネント、カスタマイズ性 |
| **Zustand** | 軽量状態管理、リアルタイムデータの更新に最適 |
| **Vite** | 高速な開発サーバー・ビルド |

**Electron不採用の理由:** メモリ200〜300MBで常駐ツールとしては重すぎる

#### 3.4.2 Tauri ↔ Frontend 通信設計

**Tauri Commands（Frontend → Rust）:**
- `start_audio_capture`, `stop_audio_capture` - 音声キャプチャ制御
- `set_whisper_model` - モデル切替
- `investigate_keyword` - Claude Code SDK経由で調査依頼
- `export_transcript` - エクスポート

**Tauri Events/Channel（Rust → Frontend）:**
- `transcript:partial` - 部分テキスト更新
- `transcript:final` - 確定テキスト追加
- `keyword:detected` - キーワード検出通知
- `ai:summary-updated` - 要約更新
- `ai:action-item` - アクションアイテム検出

#### 3.4.3 メインウィンドウレイアウト（左60% / 右40%）

```
┌─────────────────────────────────────────────────────────────────┐
│ MojiOkoshi   ● 録音中 00:32:15   [⏸ 一時停止] [⏹ 停止]  [⚙]  │
├────────────────────────────────────────┬────────────────────────┤
│                                        │                        │
│  文字起こし（60%）                       │  AIインサイト（40%）     │
│                                        │                        │
│  00:30:12 [田中] 🔵                    │  ┌────────────────────┐│
│  前回のスプリントで [RAG] の実装が      │  │ リアルタイム要約      ││
│  完了しまして...                        │  │                    ││
│                                        │  │ RAGの実装完了報告と  ││
│  00:30:25 [鈴木] 🟢                    │  │ 次スプリントの計画に  ││
│  なるほど、[LangChain] のバージョンは   │  │ ついて議論中。       ││
│  どれを使いましたか？                   │  │                    ││
│                                        │  └────────────────────┘│
│  00:30:38 [田中] 🔵                    │                        │
│  v0.2を使っています。[LCEL] で         │  ┌────────────────────┐│
│  チェーンを組んでいます。               │  │ アクションアイテム    ││
│                                        │  │                    ││
│  ┌──────────────────────────────┐      │  │ • RAGのベンチマーク  ││
│  │ "LCEL" について調査中...       │      │  │   結果を共有(田中)   ││
│  │                               │      │  │ • LangChainのアップ  ││
│  │ LCEL (LangChain Expression    │      │  │   グレード検討(鈴木) ││
│  │ Language) は LangChain v0.2で │      │  │                    ││
│  │ 導入された宣言的なチェーン構築  │      │  └────────────────────┘│
│  │ 記法です。従来のSequential     │      │                        │
│  │ Chainに代わる新しい方式...     │      │  ┌────────────────────┐│
│  └──────────────────────────────┘      │  │ 検出キーワード        ││
│                                        │  │                    ││
│  00:31:05 [鈴木] 🟢                    │  │  RAG  LangChain    ││
│  パフォーマンスはどうでしたか？▏        │  │  LCEL  v0.2        ││
│  （入力中...）                          │  │  スプリント          ││
│                                        │  │                    ││
│                                        │  └────────────────────┘│
├────────────────────────────────────────┴────────────────────────┤
│  [議事録生成] [エクスポート] [検索]              [話題タイムライン]│
└─────────────────────────────────────────────────────────────────┘
```

- 話者ごとに色分け表示（最大8人まで自動カラーアサイン、ライト/ダーク両対応）
- `@tanstack/react-virtual`による仮想スクロールで長時間会議にも対応
- 50msバッチ更新でUIパフォーマンスを維持

#### 3.4.4 キーワードハイライト & ポップオーバー

```
テキスト内のハイライト表示:

  "前回のスプリントで [RAG] の実装が完了しまして、
   [LangChain] のv0.2を使って [LCEL] で..."

   ※ [RAG] [LangChain] [LCEL] は色付きハイライト表示
      - tech_term: 青系
      - proper_noun: 緑系
      - acronym: 紫系
      - industry_term: オレンジ系

ハイライトクリック時のポップオーバー:
  ┌──────────────────────────────────┐
  │ RAG (Retrieval-Augmented         │
  │     Generation)                  │
  │                                  │
  │ LLMに外部知識ベースから取得した   │
  │ 情報を付与して回答精度を高める    │
  │ 手法。この会議ではRAGの実装完了   │
  │ が報告されている。               │
  │                                  │
  │ [さらに詳しく調査]  [コピー]      │
  └──────────────────────────────────┘
```

#### 3.4.5 コンテキストメニュー（テキスト選択時）

```
テキストを選択して右クリック:

  ┌──────────────────────────┐
  │ Claudeに調査させる  ⌘I   │
  │ 要約を生成               │
  │ Webで検索                │
  │ コピー            ⌘C    │
  │ ブックマーク       ⌘B    │
  │ アクションアイテムに追加   │
  └──────────────────────────┘
```

#### 3.4.6 フローティングミニビュー（400x250px）

会議中に他のアプリを使いながら確認できるミニウィンドウ:

```
┌──────────────────────────────┐
│ MojiOkoshi ● REC  00:32:15  │
│                              │
│ [田中] LCELでチェーンを組ん  │
│ でいます。                    │
│                              │
│ 新規: RAGベンチマーク共有     │
│                     [展開 ↗] │
└──────────────────────────────┘
```
- 常に最前面表示（Always on Top）
- ドラッグで位置移動可能
- クリックでメインウィンドウを展開
- 透明度を調整可能

#### 3.4.7 設定画面

```
┌─────────────────────────────────────────────┐
│ 設定                                         │
├─────────────────────────────────────────────┤
│                                              │
│ 音声設定                                     │
│   音声キャプチャ: [ScreenCaptureKit ▼]       │
│   対象アプリ: [Microsoft Teams ▼]            │
│   マイク入力: [MacBook Pro マイク ▼]          │
│   サンプルレート: [16000 Hz ▼]               │
│                                              │
│ Whisper設定                                  │
│   モデル: [large-v3-turbo ▼]  [ダウンロード]  │
│   日本語特化: □ kotoba-whisperを使用          │
│   言語: [日本語 ▼]  □ 自動検出               │
│   VAD感度: [━━━━━●━━━] 0.5                  │
│                                              │
│ Claude Code設定                               │
│   接続状態: [● 接続済み (Max plan)]            │
│   分析間隔: [3分 ▼]                           │
│   Web検索: [● 有効]  （内蔵WebSearch使用）     │
│                                              │
│ エクスポート設定                               │
│   デフォルト形式: [Markdown ▼]                │
│   保存先: [~/Documents/MojiOkoshi/]           │
│                                              │
│ 表示設定                                      │
│   テーマ: [システムに従う ▼]                   │
│   フォントサイズ: [14px ▼]                     │
│   ハイライト色: 種別ごとに設定可能             │
│                                              │
│                        [保存] [キャンセル]     │
└─────────────────────────────────────────────┘
```

---

## 4. 技術スタック一覧

| カテゴリ | 技術 | バージョン | 用途 |
|---------|------|----------|------|
| **アプリフレームワーク** | Tauri | v2.x | デスクトップアプリ基盤 |
| **バックエンド言語** | Rust | 1.75+ | コアロジック、音声処理 |
| **フロントエンド** | React | 19.x | UI構築 |
| **型システム** | TypeScript | 5.x | フロントエンド型安全性 |
| **スタイリング** | Tailwind CSS | 4.x | UIスタイリング |
| **UIコンポーネント** | shadcn/ui | latest | 高品質UIパーツ |
| **状態管理** | Zustand | 5.x | リアルタイム状態管理 |
| **ビルドツール** | Vite | 6.x | 高速な開発・ビルド |
| **仮想スクロール** | @tanstack/react-virtual | latest | 長時間会議の文字起こし表示 |
| **音声キャプチャ** | ScreenCaptureKit / Core Audio Taps | macOS 12.3+ / 14.4+ | システム音声取得 |
| **Rustバインディング** | screencapturekit-rs | latest | ScreenCaptureKitのRust連携 |
| **音声認識** | whisper.cpp (whisper-rs) | latest | ローカル文字起こし |
| **音声認識（代替）** | mlx-whisper | latest | Apple Silicon最適化版 |
| **VAD** | Silero VAD | v5 | 音声区間検出 |
| **話者分離** | pyannote.audio 4.0 | latest | 話者識別 |
| **AI統合** | Claude Code SDK (@anthropic-ai/claude-code) | latest | テキスト分析（Maxサブスク内） |
| **Web検索** | Claude Code 内蔵 WebSearch | - | キーワード調査（Maxサブスク内） |
| **ローカルDB** | SQLite (rusqlite) | latest | セッション・キャッシュ保存 |
| **全文検索** | SQLite FTS5 | latest | 会議履歴の全文検索 |
| **エクスポート** | pulldown-cmark / printpdf | latest | MD/PDF生成 |

---

## 5. データモデル

### 5.1 SQLite スキーマ

```sql
-- 会議セッション
CREATE TABLE sessions (
    id TEXT PRIMARY KEY,
    title TEXT,
    started_at DATETIME NOT NULL,
    ended_at DATETIME,
    target_app TEXT,              -- "Microsoft Teams", "Zoom" etc.
    whisper_model TEXT,           -- 使用モデル
    status TEXT DEFAULT 'active'  -- active / completed / archived
);

-- 文字起こしセグメント
CREATE TABLE segments (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id TEXT NOT NULL REFERENCES sessions(id),
    speaker TEXT,              -- 話者名
    speaker_color TEXT,        -- 話者カラーコード
    text TEXT NOT NULL,
    start_time REAL NOT NULL,  -- 会議開始からの秒数
    end_time REAL NOT NULL,
    confidence REAL,           -- 認識信頼度
    is_partial BOOLEAN DEFAULT FALSE,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- 検出キーワード
CREATE TABLE keywords (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id TEXT NOT NULL REFERENCES sessions(id),
    segment_id INTEGER REFERENCES segments(id),
    word TEXT NOT NULL,
    keyword_type TEXT,         -- tech_term / proper_noun / acronym / industry_term
    priority TEXT,             -- high / medium / low
    context TEXT,
    supplementary_info TEXT,   -- Claude/Web検索で取得した補足情報
    web_search_done BOOLEAN DEFAULT FALSE,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- AI分析結果
CREATE TABLE ai_insights (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id TEXT NOT NULL REFERENCES sessions(id),
    insight_type TEXT NOT NULL, -- summary / action_item / decision / question / investigation / topic
    content TEXT NOT NULL,
    assignee TEXT,             -- アクションアイテムの担当者
    time_range_start REAL,
    time_range_end REAL,
    model_used TEXT,           -- claude code sdk
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- ブックマーク
CREATE TABLE bookmarks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id TEXT NOT NULL REFERENCES sessions(id),
    segment_id INTEGER REFERENCES segments(id),
    note TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- 全文検索インデックス（会議履歴検索用）
CREATE VIRTUAL TABLE segments_fts USING fts5(
    text, speaker,
    content='segments',
    content_rowid='id'
);

-- インデックス
CREATE INDEX idx_segments_session ON segments(session_id);
CREATE INDEX idx_segments_time ON segments(session_id, start_time);
CREATE INDEX idx_keywords_session ON keywords(session_id);
CREATE INDEX idx_keywords_word ON keywords(word);
CREATE INDEX idx_insights_session ON ai_insights(session_id);
CREATE INDEX idx_insights_type ON ai_insights(session_id, insight_type);
```

---

## 6. キーボードショートカット

| ショートカット | 機能 |
|-------------|------|
| `⌘ + Shift + R` | 録音開始/停止 |
| `⌘ + Shift + P` | 一時停止/再開 |
| `⌘ + Shift + F` | フローティングミニビュー切替 |
| `⌘ + F` | 文字起こし内検索 |
| `⌘ + I` | 選択テキストをClaudeに調査依頼 |
| `⌘ + Shift + S` | 現時点までの要約を生成 |
| `⌘ + E` | 議事録エクスポート |
| `⌘ + B` | 現在の位置をブックマーク |
| `⌘ + 1` | 文字起こしパネルにフォーカス |
| `⌘ + 2` | AIインサイトパネルにフォーカス |
| `⌘ + ,` | 設定画面を開く |
| `Space` | 録音一時停止/再開（フォーカス時） |

---

## 7. プロジェクト構造

```
mojiokoshi/
├── src-tauri/                     # Rust バックエンド
│   ├── src/
│   │   ├── main.rs
│   │   ├── audio/                 # 音声キャプチャ
│   │   │   ├── capture.rs         # ScreenCaptureKit統合
│   │   │   ├── core_audio_tap.rs  # Core Audio Taps統合
│   │   │   └── microphone.rs      # マイク入力
│   │   ├── whisper/               # Whisper統合
│   │   │   ├── engine.rs          # whisper-rs統合
│   │   │   ├── vad.rs             # Silero VAD
│   │   │   └── pipeline.rs        # 処理パイプライン
│   │   ├── claude/                # Claude Code SDK統合
│   │   │   ├── sdk_bridge.rs      # Claude Code SDK (Node.js) ブリッジ
│   │   │   ├── analyzer.rs        # 統合分析（キーワード+要約+アクション）
│   │   │   ├── investigation.rs   # オンデマンド調査
│   │   │   └── prompts.rs         # プロンプトテンプレート
│   │   ├── storage/               # データ永続化
│   │   │   ├── db.rs              # SQLite操作
│   │   │   └── export.rs          # エクスポート（MD/PDF）
│   │   └── commands.rs            # Tauriコマンド定義
│   └── Cargo.toml
├── src/                           # React フロントエンド
│   ├── components/
│   │   ├── transcript/            # 文字起こし表示
│   │   │   ├── TranscriptPanel.tsx
│   │   │   ├── SegmentLine.tsx
│   │   │   ├── KeywordHighlight.tsx
│   │   │   └── KeywordPopover.tsx
│   │   ├── insights/              # AIインサイト
│   │   │   ├── InsightsPanel.tsx
│   │   │   ├── SummaryCard.tsx
│   │   │   ├── ActionItemList.tsx
│   │   │   └── KeywordList.tsx
│   │   ├── meeting/               # 会議制御
│   │   │   ├── MeetingControls.tsx
│   │   │   ├── FloatingMiniView.tsx
│   │   │   └── TopicTimeline.tsx
│   │   └── settings/              # 設定
│   │       └── SettingsDialog.tsx
│   ├── stores/                    # Zustand ストア
│   │   ├── transcriptStore.ts
│   │   ├── insightsStore.ts
│   │   └── settingsStore.ts
│   ├── hooks/                     # カスタムフック
│   │   ├── useTauriEvents.ts
│   │   └── useKeyboardShortcuts.ts
│   ├── lib/                       # ユーティリティ
│   │   └── tauri-commands.ts      # Tauriコマンド型定義
│   └── types/                     # 型定義
│       └── index.ts
├── package.json
├── tsconfig.json
├── tailwind.config.ts
└── DESIGN.md
```

---

## 8. 開発フェーズ計画

### Phase 1: 基盤構築（MVP）
**目標**: 音声キャプチャ→文字起こし→表示の基本パイプライン

- [ ] Tauri v2プロジェクトの初期セットアップ（React + TypeScript + Tailwind）
- [ ] ScreenCaptureKitによる音声キャプチャ実装（screencapturekit-rs）
- [ ] whisper.cpp (whisper-rs) の組み込み + large-v3-turboモデル
- [ ] Silero VADによるリアルタイム処理パイプライン
- [ ] 基本的な文字起こし表示UI（タイムスタンプ、仮想スクロール）
- [ ] SQLiteによるセッション管理
- [ ] 録音開始/停止/一時停止コントロール
- [ ] macOS権限オンボーディングフロー

### Phase 2: AI統合
**目標**: Claude Code SDKによるインテリジェント機能の追加

- [ ] Claude Code SDK (Node.js) のTauriバックエンドへの統合
- [ ] 統合プロンプト実装（キーワード抽出+要約+アクションアイテムを一括処理）
- [ ] 内蔵WebSearchによるキーワード自動調査
- [ ] キーワードハイライト表示 + ポップオーバー
- [ ] リアルタイム要約機能（3分バッチ）
- [ ] テキスト選択→Claude調査機能（⌘+I）

### Phase 3: 高度な機能
**目標**: MTG支援機能の充実

- [ ] アクションアイテム自動抽出
- [ ] 決定事項の自動記録
- [ ] 話者分離の実装（pyannote.audio）
- [ ] 議事録自動生成（Markdown）
- [ ] フローティングミニビュー（Always on Top）
- [ ] AIインサイトパネルの充実

### Phase 4: 洗練 & エクスポート
**目標**: UX改善とエクスポート機能

- [ ] 話題タイムライン表示
- [ ] ダークモード対応
- [ ] 会議履歴管理・検索（FTS5全文検索）
- [ ] PDFエクスポート
- [ ] Notionエクスポート連携
- [ ] 設定画面の充実
- [ ] パフォーマンス最適化

### Phase 5: 拡張機能
**目標**: 付加価値機能

- [ ] 多言語翻訳対応
- [ ] 感情・トーン分析
- [ ] 前回会議との自動リンク
- [ ] カスタムキーワード辞書
- [ ] 質問候補提案
- [ ] Apple SpeechAnalyzer対応（macOS Tahoe以降）
- [ ] kotoba-whisper日本語特化モード

---

## 9. 非機能要件

### 9.1 パフォーマンス
- 文字起こしのend-to-endレイテンシー: 380〜520ms
- UI更新: 50msバッチ更新、60fps維持
- メモリ使用量: ~500MB（Whisper large-v3-turboモデル使用時）
- アプリバンドルサイズ: ~5MB（Whisperモデルは別途ダウンロード）
- CPU使用率: 会議アプリの動作に影響しないこと（30%以下目標）

### 9.2 セキュリティ
- Claude Code認証情報はmacOS Keychainに暗号化保存
- 文字起こしデータはローカルSQLiteのみに保存
- Claude Code SDKへの送信データは会議テキストの一部のみ（最小限の情報、直近15分+要約）
- オフラインモード対応（AI機能なしの基本文字起こし）

### 9.3 プライバシー
- 音声データは一時バッファのみ、ディスクに保存しない
- テキストデータのローカル保持期間を設定可能
- セッション単位の削除機能

### 9.4 互換性
- macOS 13 (Ventura) 以上を推奨対象
- Apple Silicon (M1/M2/M3/M4) ネイティブ対応
- Intel Macはfallback対応（Whisperの推論速度低下あり）

---

## 10. 将来の拡張案

1. **Slack/Teams連携**: 議事録を直接チャンネルに投稿
2. **Jira/Linear連携**: アクションアイテムをタスクチケットとして自動作成
3. **Google Calendar連携**: 会議予定と自動連動して録音開始
4. **社内用語辞書**: 組織固有の用語をプリセット登録
5. **マルチデバイス同期**: iCloud経由で複数Macのデータを同期
6. **iOS companion app**: iPhoneから会議メモを確認
7. **Plugin System**: サードパーティ拡張機能の仕組み
8. **Apple SpeechAnalyzer**: macOS Tahoe普及後のネイティブAPI移行

---

## 付録A: 各エージェント調査の詳細ドキュメント

- `docs/04_ui_ux_design.md` - UI/UX詳細設計書
- `docs/design/claude-api-integration-design.md` - Claude統合設計書（API版の参考資料）

## 付録B: Claude Code SDK 統合の補足

### CLI経由の呼び出し（代替方式）

Claude Code SDKが使えない環境では、CLIをパイプで呼び出すことも可能：

```bash
echo "会議テキストを分析してください: ..." | claude -p --output-format json
```

### Maxプランの使用量上限への対応

1時間の会議で統合プロンプト方式を使用した場合の呼び出し回数見込み：

| 処理 | 回数 |
|------|------|
| 3分バッチ（統合分析） | ~20回 |
| オンデマンド調査（⌘+I） | ~5回（ユーザー依存） |
| 議事録生成（会議終了時） | 1回 |
| **合計** | **~26回/時間** |

Maxプランの使用量上限内で十分に運用可能。
