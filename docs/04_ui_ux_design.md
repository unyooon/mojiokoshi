# UI/UX設計書 - リアルタイム文字起こし＆AI分析デスクトップアプリ

## 1. デスクトップアプリ技術選定

### 1.1 技術比較表

| 項目 | Tauri v2 | Electron | SwiftUI + AppKit | Flutter Desktop |
|------|----------|----------|-----------------|-----------------|
| **バンドルサイズ** | ~3-10 MB | ~100-200 MB | ~5-15 MB | ~20-40 MB |
| **メモリ使用量(アイドル)** | ~30-40 MB | ~200-300 MB | ~15-30 MB | ~60-100 MB |
| **起動時間** | <0.5秒 | 1-2秒 | <0.3秒 | ~0.8秒 |
| **レンダリングエンジン** | システムWebView (WKWebView) | Chromium同梱 | ネイティブ | Impellerエンジン |
| **バックエンド言語** | Rust | Node.js | Swift | Dart |
| **フロントエンド** | Web技術(React/Vue等) | Web技術(React/Vue等) | SwiftUI | Flutter Widget |
| **クロスプラットフォーム** | macOS/Windows/Linux/iOS/Android | macOS/Windows/Linux | macOS/iOS のみ | macOS/Windows/Linux/iOS/Android |
| **ネイティブAPI連携** | Rust経由で高性能 | Node.js addon | 完全ネイティブ | Platform Channel |
| **エコシステム成熟度** | 成長中(2024年v2リリース) | 非常に成熟 | Apple公式フレームワーク | 成長中(Desktop安定版2026) |
| **開発効率** | 中（Rust学習コスト） | 高（Web技術活用） | 中（Apple専用知識） | 高（Hot Reload） |
| **セキュリティ** | 高（Rustの安全性＋権限システム） | 中（Chromium脆弱性リスク） | 高（サンドボックス） | 中 |
| **音声処理との連携** | Rustから直接Whisper呼び出し可能 | child_process経由 | Core Audio直接利用 | Platform Channel経由 |

### 1.2 推奨：Tauri v2 + React/TypeScript

**選定理由：**

1. **軽量性**: バンドルサイズ~5MB、メモリ使用量~30-40MB。常駐アプリとして他の作業を圧迫しない
2. **Rustバックエンド**: Whisper.cpp（C/C++）との連携が容易。FFI経由で直接バインディング可能。音声処理のパフォーマンスが重要な本アプリに最適
3. **セキュリティ**: Rust bridgeによる厳格な権限管理。音声データやAPIキーの安全な取り扱い
4. **リアルタイムストリーミング**: Tauri v2のChannelシステムにより、Rust側からフロントエンドへの高速データストリーミングが可能
5. **macOS統合**: WKWebViewを活用し、macOSネイティブに近い操作感を実現
6. **将来性**: 2024年v2安定版リリース後、採用率35%増加。活発なコミュニティ

**フロントエンド構成：**
- **React 19** + **TypeScript 5.x**: コンポーネント指向で複雑なUIを管理
- **Tailwind CSS v4**: ユーティリティファーストで高速スタイリング
- **shadcn/ui**: Radix UIベースのアクセシブルなコンポーネント群
- **Zustand**: 軽量状態管理（リアルタイムデータに最適）
- **Vite**: 高速ビルドツール（Tauri v2公式推奨）

**補足 - 却下理由：**
- **Electron**: メモリ消費が大きく、常駐型MTGツールとしては重い
- **SwiftUI**: macOS専用。将来的なWindows/Linux対応の可能性を閉ざす。WebView系UIの柔軟性に劣る
- **Flutter Desktop**: Dart言語のエコシステムがWeb技術に比べて小さい。Whisperとの連携にPlatform Channelの追加レイヤーが必要

---

## 2. 画面構成・レイアウト設計

### 2.1 メイン画面レイアウト（通常モード）

```
+-----------------------------------------------------------------------------------+
|  MojiOkoshi                                              [_] [□] [x]  Menu Bar    |
+-----------------------------------------------------------------------------------+
|  [● REC 00:15:32]  [⏸ Pause]  [■ Stop]  |  Meeting: Weekly Standup  | [⚙ Settings]|
+-----------------------------------------------------------------------------------+
|                           |                                                       |
|   Transcript Panel        |          AI Insights Panel                            |
|   (60% width)             |          (40% width)                                  |
|                           |                                                       |
|  ┌───────────────────┐    |   ┌─────────────────────────────────────┐              |
|  │ 10:00:05           │    |   │  📋 Summary (Auto-updating)         │              |
|  │ 田中: おはよう     │    |   │  ┌─────────────────────────────┐    │              |
|  │ ございます。本日の │    |   │  │ - Q4売上目標の進捗確認       │    │              |
|  │ 議題に入ります。   │    |   │  │ - 新プロダクトのリリース日程  │    │              |
|  │                    │    |   │  │ - チームリソースの再配分      │    │              |
|  │ 10:00:12           │    |   │  └─────────────────────────────┘    │              |
|  │ 佐藤: はい、まず   │    |   │                                      │              |
|  │ Q4の[売上目標]に   │    |   │  ✅ Action Items                     │              |
|  │ ついてですが...    │    |   │  ┌─────────────────────────────┐    │              |
|  │                    │    |   │  │ □ 佐藤: 売上レポート提出(金) │    │              |
|  │ 10:00:25           │    |   │  │ □ 田中: リソース計画書作成   │    │              |
|  │ 鈴木: [競合分析]の │    |   │  └─────────────────────────────┘    │              |
|  │ 結果を共有します。 │    |   │                                      │              |
|  │ A社が新製品を...   │    |   │  🏷 Keywords                         │              |
|  │                    │    |   │  ┌─────────────────────────────┐    │              |
|  │ [Auto-scroll ↓]    │    |   │  │ [売上目標] [競合分析]        │    │              |
|  └───────────────────┘    |   │  │ [リリース日程] [A社]         │    │              |
|                           |   │  └─────────────────────────────┘    │              |
|  ┌───────────────────┐    |   │                                      │              |
|  │ 🔍 Search...       │    |   │  📊 Research Results                 │              |
|  │ Filter: [All ▼]    │    |   │  (Click keyword to investigate)      │              |
|  └───────────────────┘    |   └─────────────────────────────────────┘              |
|                           |                                                       |
+-----------------------------------------------------------------------------------+
|  Status: Recording | Whisper: large-v3 | Speakers: 3 detected | CPU: 12%         |
+-----------------------------------------------------------------------------------+
```

### 2.2 ハイライト＋ポップオーバー表示

```
Transcript Panel with Keyword Highlight & Popover:

  │ 10:00:25                                              │
  │ 鈴木: [競合分析]の結果を共有します。                   │
  │ A社が新製品を発表しました。                           │
  │                                                       │
  │         ┌──────────────────────────────────┐           │
  │         │ 🔍 "競合分析" について            │           │
  │         │                                  │           │
  │         │ Claude調査結果:                   │           │
  │         │ 競合分析とは、競合他社の戦略・    │           │
  │         │ 製品・市場ポジションを体系的に    │           │
  │         │ 評価するプロセスです。             │           │
  │         │                                  │           │
  │         │ 📎 関連会議メモ:                  │           │
  │         │ - 2024/12/01 戦略会議で言及       │           │
  │         │                                  │           │
  │         │ [📋 Copy] [🔗 Deep Dive] [✕]     │           │
  │         └──────────────────────────────────┘           │
```

### 2.3 フローティング/ミニビューモード

```
┌──────────────────────────────────┐
│  MojiOkoshi  ● 00:15:32  [⏸][■] │
│──────────────────────────────────│
│  田中: Q4の目標について...       │
│  佐藤: 売上レポートを...         │
│  鈴木: 競合分析の結果を...  ↓    │
│──────────────────────────────────│
│  Keywords: [売上目標] [競合分析]  │
└──────────────────────────────────┘
(Always on Top / 半透明 / ドラッグ可能 / リサイズ可能)
サイズ: 約 400x250px
```

### 2.4 会議履歴画面

```
+-----------------------------------------------------------------------------------+
|  MojiOkoshi  >  Meeting History                                    [⚙ Settings]   |
+-----------------------------------------------------------------------------------+
|  🔍 Search meetings...        | Date: [All ▼]  | Tag: [All ▼]                    |
+-----------------------------------------------------------------------------------+
|                                                                                   |
|  ┌─────────────────────────────────────────────────────────────────────────┐       |
|  │  📅 2025-02-12  Weekly Standup                           Duration: 45min│       |
|  │  Speakers: 田中, 佐藤, 鈴木  |  Keywords: 売上目標, 競合分析         │       |
|  │  Summary: Q4売上進捗と競合分析について議論...                          │       |
|  │  [📄 Open] [📤 Export ▼] [🗑 Delete]                                  │       |
|  └─────────────────────────────────────────────────────────────────────────┘       |
|                                                                                   |
|  ┌─────────────────────────────────────────────────────────────────────────┐       |
|  │  📅 2025-02-10  Product Planning                         Duration: 60min│       |
|  │  Speakers: 田中, 高橋, 山田  |  Keywords: リリース計画, UI改善        │       |
|  │  Summary: 新機能のリリーススケジュールと...                            │       |
|  │  [📄 Open] [📤 Export ▼] [🗑 Delete]                                  │       |
|  └─────────────────────────────────────────────────────────────────────────┘       |
|                                                                                   |
+-----------------------------------------------------------------------------------+
```

### 2.5 設定画面

```
+-----------------------------------------------------------------------------------+
|  MojiOkoshi  >  Settings                                                          |
+-----------------------------------------------------------------------------------+
|                                                                                   |
|  ┌─ General ──────────────────────────────────────────────────────────────┐       |
|  │  Language:          [日本語 ▼]                                         │       |
|  │  Theme:             [○ Light  ● Dark  ○ System]                       │       |
|  │  Launch at Login:   [✓]                                                │       |
|  │  Mini View Default: [ ]                                                │       |
|  └────────────────────────────────────────────────────────────────────────┘       |
|                                                                                   |
|  ┌─ Audio & Transcription ────────────────────────────────────────────────┐       |
|  │  Audio Source:      [System Audio + Microphone ▼]                      │       |
|  │  Whisper Model:     [large-v3 ▼]  (Accuracy: ★★★★★  Speed: ★★☆☆☆)   │       |
|  │  Language:          [Auto-detect ▼]                                    │       |
|  │  VAD Sensitivity:   [████████░░] 80%                                  │       |
|  │  Speaker Diarization: [✓] Enabled                                     │       |
|  └────────────────────────────────────────────────────────────────────────┘       |
|                                                                                   |
|  ┌─ AI & API ─────────────────────────────────────────────────────────────┐       |
|  │  Claude API Key:    [sk-ant-•••••••••••••] [👁 Show] [Test]           │       |
|  │  Claude Model:      [claude-sonnet-4-5-20250929 ▼]                             │       |
|  │  Auto-Summarize:    [✓] Every [5 ▼] minutes                           │       |
|  │  Auto-Keywords:     [✓]                                                │       |
|  │  Research Depth:    [○ Quick  ● Standard  ○ Deep]                     │       |
|  └────────────────────────────────────────────────────────────────────────┘       |
|                                                                                   |
|  ┌─ Highlight Rules ──────────────────────────────────────────────────────┐       |
|  │  ┌──────────────────────────────────────────────────────────────┐      │       |
|  │  │  Rule 1: Technical Terms    Color: [🟦 Blue]   [Edit][Del]  │      │       |
|  │  │  Rule 2: Action Items       Color: [🟩 Green]  [Edit][Del]  │      │       |
|  │  │  Rule 3: People Names       Color: [🟨 Yellow] [Edit][Del]  │      │       |
|  │  │  [+ Add Rule]                                               │      │       |
|  │  └──────────────────────────────────────────────────────────────┘      │       |
|  └────────────────────────────────────────────────────────────────────────┘       |
|                                                                                   |
|  ┌─ Export ────────────────────────────────────────────────────────────────┐       |
|  │  Default Format:    [Markdown ▼]                                       │       |
|  │  Notion Integration: [Connect to Notion...]                            │       |
|  │  Auto-Export:       [ ] Save to [~/Documents/MojiOkoshi/]              │       |
|  └────────────────────────────────────────────────────────────────────────┘       |
|                                                                                   |
|                                            [Cancel]  [Save Settings]              |
+-----------------------------------------------------------------------------------+
```

---

## 3. コンポーネント設計

### 3.1 コンポーネントツリー

```
App
├── TitleBar (カスタムタイトルバー)
├── MeetingControls (録音制御)
│   ├── RecordButton
│   ├── PauseButton
│   ├── StopButton
│   ├── Timer
│   └── MeetingTitle
├── MainLayout
│   ├── TranscriptPanel (文字起こし表示)
│   │   ├── TranscriptEntry[]
│   │   │   ├── Timestamp
│   │   │   ├── SpeakerLabel (色分け)
│   │   │   └── TranscriptText
│   │   │       └── HighlightedKeyword[] (クリック可能)
│   │   ├── AutoScrollController
│   │   └── SearchBar
│   │       ├── SearchInput
│   │       └── FilterDropdown
│   ├── ResizableDivider
│   └── InsightsPanel (AI分析)
│       ├── TabBar [Summary | Actions | Keywords | Research]
│       ├── SummaryTab
│       │   └── AutoUpdatingSummary
│       ├── ActionItemsTab
│       │   └── ActionItem[] (チェックボックス付き)
│       ├── KeywordsTab
│       │   └── KeywordBadge[] (クリックで調査)
│       └── ResearchTab
│           └── ResearchResult[]
│               ├── ResearchQuery
│               └── ResearchContent
├── KeywordPopover (ポップオーバー)
│   ├── KeywordTitle
│   ├── ClaudeResearchContent
│   ├── RelatedMeetings
│   └── ActionButtons [Copy | DeepDive | Close]
├── StatusBar
│   ├── RecordingStatus
│   ├── WhisperModelInfo
│   ├── SpeakerCount
│   └── SystemMetrics (CPU/Memory)
└── FloatingMiniView (フローティング)
    ├── CompactControls
    ├── RecentTranscripts (直近3行)
    └── KeywordBadges
```

### 3.2 主要コンポーネント詳細

#### TranscriptEntry コンポーネント

```typescript
interface TranscriptEntry {
  id: string;
  timestamp: number;        // Unix timestamp
  speakerId: string;
  speakerName: string;
  speakerColor: string;     // 話者別の色
  text: string;
  keywords: Keyword[];      // 検出されたキーワード
  isPartial: boolean;       // リアルタイム途中結果
  confidence: number;       // 認識信頼度
}

interface Keyword {
  id: string;
  text: string;
  startIndex: number;
  endIndex: number;
  category: 'technical' | 'action' | 'person' | 'custom';
  color: string;
  researchResult?: ResearchResult;
}
```

#### 話者カラーパレット

```typescript
const SPEAKER_COLORS = {
  light: [
    { bg: '#EFF6FF', text: '#1D4ED8', border: '#93C5FD' },  // Blue
    { bg: '#F0FDF4', text: '#15803D', border: '#86EFAC' },  // Green
    { bg: '#FFF7ED', text: '#C2410C', border: '#FDBA74' },  // Orange
    { bg: '#FAF5FF', text: '#7E22CE', border: '#C4B5FD' },  // Purple
    { bg: '#FDF2F8', text: '#BE185D', border: '#F9A8D4' },  // Pink
    { bg: '#ECFDF5', text: '#047857', border: '#6EE7B7' },  // Emerald
    { bg: '#FEF3C7', text: '#92400E', border: '#FCD34D' },  // Amber
    { bg: '#F0F9FF', text: '#0369A1', border: '#7DD3FC' },  // Sky
  ],
  dark: [
    { bg: '#1E3A5F', text: '#93C5FD', border: '#1D4ED8' },
    { bg: '#14532D', text: '#86EFAC', border: '#15803D' },
    { bg: '#431407', text: '#FDBA74', border: '#C2410C' },
    { bg: '#3B0764', text: '#C4B5FD', border: '#7E22CE' },
    { bg: '#500724', text: '#F9A8D4', border: '#BE185D' },
    { bg: '#064E3B', text: '#6EE7B7', border: '#047857' },
    { bg: '#451A03', text: '#FCD34D', border: '#92400E' },
    { bg: '#0C4A6E', text: '#7DD3FC', border: '#0369A1' },
  ]
};
```

---

## 4. インタラクション設計

### 4.1 キーボードショートカット

| ショートカット | アクション |
|---------------|-----------|
| `Cmd + Shift + R` | 録音開始/停止 |
| `Cmd + Shift + P` | 一時停止/再開 |
| `Cmd + Shift + M` | ミニビュー切替 |
| `Cmd + F` | 文字起こし内検索 |
| `Cmd + E` | エクスポート |
| `Cmd + ,` | 設定画面 |
| `Cmd + K` | キーワード一覧表示 |
| `Cmd + S` | 要約パネルにフォーカス |
| `Cmd + 1` | Summaryタブ |
| `Cmd + 2` | Action Itemsタブ |
| `Cmd + 3` | Keywordsタブ |
| `Cmd + 4` | Researchタブ |
| `Escape` | ポップオーバー/検索を閉じる |
| `Cmd + Shift + C` | 選択テキストをClaudeで調査 |

### 4.2 コンテキストメニュー（テキスト選択時）

```
┌──────────────────────────┐
│  🔍 Claudeで調査          │
│  📋 コピー                │
│  ──────────────────────  │
│  📌 キーワードとして登録  │
│  🏷  ハイライトに追加     │
│  ──────────────────────  │
│  🔗 Notionに送信          │
│  📝 アクションアイテム化  │
└──────────────────────────┘
```

### 4.3 ドラッグ&ドロップ操作

- **テキスト → アクションアイテムパネル**: ドラッグでアクションアイテムとして登録
- **テキスト → キーワードパネル**: ドラッグでカスタムキーワードとして追加
- **音声ファイル → アプリウィンドウ**: 過去の音声ファイルの文字起こし開始
- **テキスト → 外部アプリ**: ドラッグで他アプリへテキスト共有

### 4.4 会議制御フロー

```
[Idle] ──(Start)──> [Recording] ──(Pause)──> [Paused]
                         │                       │
                         │                   (Resume)
                         │                       │
                     (Stop)                      ▼
                         │              [Recording]
                         ▼
                    [Processing]
                         │
                    (Complete)
                         │
                         ▼
                  [Review & Export]
```

### 4.5 リアルタイムデータフロー

```
[System Audio] → [Audio Capture (Rust)] → [VAD Processing]
                                               │
                                          [Voice Detected]
                                               │
                                               ▼
                                    [Whisper Transcription]
                                               │
                              ┌────────────────┼────────────────┐
                              ▼                ▼                ▼
                     [Partial Result]   [Final Result]   [Speaker ID]
                              │                │                │
                              ▼                ▼                ▼
                     [UI: Typing...]   [UI: Committed]  [UI: Color]
                                               │
                                               ▼
                                    [Keyword Extraction]
                                               │
                                    ┌──────────┼──────────┐
                                    ▼          ▼          ▼
                              [Highlight]  [Summary]  [Actions]
                                    │          │          │
                                    ▼          ▼          ▼
                              [UI Update] [AI Panel]  [AI Panel]
```

---

## 5. 状態管理設計

### 5.1 Zustand Store構成

```typescript
// stores/meetingStore.ts
interface MeetingState {
  // 会議制御
  status: 'idle' | 'recording' | 'paused' | 'processing' | 'review';
  startTime: number | null;
  duration: number;
  meetingTitle: string;

  // 文字起こし
  entries: TranscriptEntry[];
  partialEntry: TranscriptEntry | null;

  // 話者
  speakers: Map<string, Speaker>;

  // キーワード
  detectedKeywords: Keyword[];
  highlightRules: HighlightRule[];

  // AI分析
  summary: string;
  actionItems: ActionItem[];
  researchResults: Map<string, ResearchResult>;

  // UI状態
  autoScroll: boolean;
  activeTab: 'summary' | 'actions' | 'keywords' | 'research';
  searchQuery: string;
  filterSpeaker: string | null;
  viewMode: 'full' | 'mini';

  // アクション
  startRecording: () => Promise<void>;
  pauseRecording: () => void;
  stopRecording: () => Promise<void>;
  addEntry: (entry: TranscriptEntry) => void;
  updatePartial: (entry: TranscriptEntry) => void;
  investigateKeyword: (keyword: string) => Promise<void>;
  investigateSelection: (text: string) => Promise<void>;
  exportMeeting: (format: 'markdown' | 'pdf' | 'notion') => Promise<void>;
}
```

### 5.2 Tauri コマンド（Rust ↔ Frontend）

```typescript
// Tauri Commands (Frontend → Rust)
invoke('start_audio_capture', { source: 'system_and_mic' });
invoke('stop_audio_capture');
invoke('set_whisper_model', { model: 'large-v3' });
invoke('export_transcript', { format: 'markdown', path: '/path/to/file' });
invoke('get_meeting_history');
invoke('delete_meeting', { meetingId: 'xxx' });
invoke('test_api_key', { apiKey: 'sk-ant-...' });

// Tauri Events (Rust → Frontend, via Channel)
listen('transcript:partial', (event) => { /* 途中結果 */ });
listen('transcript:final', (event) => { /* 確定結果 */ });
listen('transcript:speaker', (event) => { /* 話者識別 */ });
listen('keyword:detected', (event) => { /* キーワード検出 */ });
listen('ai:summary-updated', (event) => { /* 要約更新 */ });
listen('ai:action-item', (event) => { /* アクションアイテム */ });
listen('ai:research-result', (event) => { /* 調査結果 */ });
listen('audio:level', (event) => { /* 音声レベル */ });
listen('system:metrics', (event) => { /* CPU/メモリ */ });
```

---

## 6. ダークモード対応

### 6.1 カラートークン設計

```typescript
const theme = {
  light: {
    // 背景
    bgPrimary: '#FFFFFF',
    bgSecondary: '#F9FAFB',
    bgTertiary: '#F3F4F6',
    bgPanel: '#FFFFFF',

    // テキスト
    textPrimary: '#111827',
    textSecondary: '#6B7280',
    textMuted: '#9CA3AF',

    // ボーダー
    border: '#E5E7EB',
    borderFocus: '#3B82F6',

    // アクセント
    accentPrimary: '#3B82F6',
    accentSuccess: '#10B981',
    accentWarning: '#F59E0B',
    accentDanger: '#EF4444',
    accentRecording: '#EF4444',

    // キーワードハイライト
    highlightTechnical: '#DBEAFE',
    highlightAction: '#D1FAE5',
    highlightPerson: '#FEF3C7',
    highlightCustom: '#EDE9FE',
  },
  dark: {
    bgPrimary: '#111827',
    bgSecondary: '#1F2937',
    bgTertiary: '#374151',
    bgPanel: '#1F2937',

    textPrimary: '#F9FAFB',
    textSecondary: '#D1D5DB',
    textMuted: '#6B7280',

    border: '#374151',
    borderFocus: '#60A5FA',

    accentPrimary: '#60A5FA',
    accentSuccess: '#34D399',
    accentWarning: '#FBBF24',
    accentDanger: '#F87171',
    accentRecording: '#F87171',

    highlightTechnical: '#1E3A5F',
    highlightAction: '#064E3B',
    highlightPerson: '#451A03',
    highlightCustom: '#3B0764',
  }
};
```

### 6.2 実装方針

- macOSのシステム設定（ダーク/ライトモード）に自動追従
- Tailwind CSSの `dark:` プレフィックスを活用
- CSS変数（Custom Properties）でテーマカラーを管理
- `prefers-color-scheme` メディアクエリ + ユーザー設定のオーバーライド

---

## 7. エクスポート機能

### 7.1 Markdown出力フォーマット

```markdown
# Meeting: Weekly Standup
**Date:** 2025-02-12 10:00 - 10:45
**Speakers:** 田中, 佐藤, 鈴木

## Summary
Q4の売上目標進捗と競合分析について議論。新プロダクトのリリース日程を確定。

## Action Items
- [ ] 佐藤: 売上レポート提出（金曜日まで）
- [ ] 田中: リソース計画書作成
- [ ] 鈴木: 競合分析レポートの更新

## Keywords
売上目標, 競合分析, リリース日程, A社, リソース配分

## Transcript
**[10:00:05] 田中:** おはようございます。本日の議題に入ります。

**[10:00:12] 佐藤:** はい、まずQ4の売上目標についてですが...

**[10:00:25] 鈴木:** 競合分析の結果を共有します。A社が新製品を...
```

### 7.2 PDF出力

- HTMLテンプレートからPDF生成（Rust側で `wkhtmltopdf` または `printpdf` クレート使用）
- ヘッダー/フッター、ページ番号、会社ロゴ対応

### 7.3 Notion連携

- Notion API経由でデータベースにページ作成
- 見出し、テーブル、チェックリストブロックに自動変換
- 設定画面でIntegration Token入力

---

## 8. 検索・フィルタ機能

### 8.1 トランスクリプト内検索

- **インクリメンタルサーチ**: 入力と同時にリアルタイムマッチ
- **ハイライト表示**: マッチ箇所を視覚的に強調
- **前/次ボタン**: マッチ間をジャンプ（`Cmd+G` / `Cmd+Shift+G`）
- **正規表現対応**: 高度な検索パターン

### 8.2 フィルタ機能

- **話者フィルタ**: 特定の話者の発言のみ表示
- **時間範囲フィルタ**: タイムスタンプ範囲で絞り込み
- **キーワードフィルタ**: 特定カテゴリのキーワードを含む発言のみ
- **信頼度フィルタ**: 認識信頼度による絞り込み

### 8.3 会議履歴検索

- **全文検索**: 過去の全会議のトランスクリプトを横断検索
- **日付範囲**: カレンダーUIで期間指定
- **タグ**: ユーザー定義タグによる分類
- **話者**: 特定の参加者が含まれる会議を検索

---

## 9. アクセシビリティ

- **キーボードナビゲーション**: 全機能がキーボードのみで操作可能
- **フォーカス管理**: 明確なフォーカスインジケーター
- **スクリーンリーダー対応**: ARIA属性の適切な設定
- **フォントサイズ調整**: `Cmd + +/-` でフォントサイズ変更
- **ハイコントラストモード**: カラーだけに依存しない情報伝達

---

## 10. パフォーマンス最適化

### 10.1 仮想スクロール

- 長時間会議（数千エントリ）でも滑らかなスクロール
- `@tanstack/react-virtual` を使用した仮想化リスト
- 表示範囲外のDOMノードを自動的に解放

### 10.2 リアルタイム更新の最適化

- **バッチ更新**: 50msごとにUI更新をバッチ処理
- **Partial結果のdebounce**: 中間結果の更新頻度を制御
- **メモ化**: `React.memo` と `useMemo` で不要な再レンダリング防止
- **Web Worker**: キーワード検出・ハイライト処理をワーカースレッドで実行

### 10.3 データ永続化

- **SQLite** (via Tauri plugin): 会議データ、設定の保存
- **インデックス**: 全文検索用のFTS5インデックス
- **ファイルストレージ**: 音声ファイル、エクスポートファイルの管理

---

## 11. 推奨ディレクトリ構造

```
src/
├── main.tsx                    # エントリポイント
├── App.tsx                     # ルートコンポーネント
├── components/
│   ├── layout/
│   │   ├── TitleBar.tsx
│   │   ├── MainLayout.tsx
│   │   ├── StatusBar.tsx
│   │   └── ResizableDivider.tsx
│   ├── meeting/
│   │   ├── MeetingControls.tsx
│   │   ├── RecordButton.tsx
│   │   ├── Timer.tsx
│   │   └── MeetingTitle.tsx
│   ├── transcript/
│   │   ├── TranscriptPanel.tsx
│   │   ├── TranscriptEntry.tsx
│   │   ├── SpeakerLabel.tsx
│   │   ├── HighlightedText.tsx
│   │   ├── AutoScrollController.tsx
│   │   └── SearchBar.tsx
│   ├── insights/
│   │   ├── InsightsPanel.tsx
│   │   ├── SummaryTab.tsx
│   │   ├── ActionItemsTab.tsx
│   │   ├── KeywordsTab.tsx
│   │   └── ResearchTab.tsx
│   ├── popover/
│   │   ├── KeywordPopover.tsx
│   │   └── ContextMenu.tsx
│   ├── mini/
│   │   └── FloatingMiniView.tsx
│   ├── history/
│   │   ├── MeetingHistory.tsx
│   │   └── MeetingCard.tsx
│   └── settings/
│       ├── SettingsPage.tsx
│       ├── AudioSettings.tsx
│       ├── AISettings.tsx
│       ├── HighlightRuleEditor.tsx
│       └── ExportSettings.tsx
├── stores/
│   ├── meetingStore.ts
│   ├── settingsStore.ts
│   └── historyStore.ts
├── hooks/
│   ├── useTauriEvents.ts
│   ├── useAutoScroll.ts
│   ├── useKeyboardShortcuts.ts
│   ├── useContextMenu.ts
│   └── useTheme.ts
├── lib/
│   ├── tauri-commands.ts       # Tauriコマンド呼び出し
│   ├── keyword-detector.ts     # キーワード検出ロジック
│   ├── export.ts               # エクスポート処理
│   └── theme.ts                # テーマ設定
├── types/
│   ├── transcript.ts
│   ├── meeting.ts
│   ├── keyword.ts
│   └── settings.ts
└── styles/
    └── globals.css
```

---

## 12. 技術スタック総括

| レイヤー | 技術 | バージョン |
|---------|------|-----------|
| デスクトップフレームワーク | Tauri v2 | 2.x |
| フロントエンドフレームワーク | React | 19.x |
| 言語 | TypeScript | 5.x |
| ビルドツール | Vite | 6.x |
| CSSフレームワーク | Tailwind CSS | 4.x |
| UIコンポーネント | shadcn/ui (Radix UI) | latest |
| 状態管理 | Zustand | 5.x |
| 仮想スクロール | @tanstack/react-virtual | 3.x |
| バックエンド言語 | Rust | 1.8x |
| ローカルDB | SQLite (via tauri-plugin-sql) | - |
| アイコン | Lucide React | latest |
| アニメーション | Framer Motion | 11.x |
