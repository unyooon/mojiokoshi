# MojiOkoshi

**macOS ローカルで動作するリアルタイム会議文字起こし＋AI 分析デスクトップアプリ**

Teams / Zoom 等の Web 会議の音声をキャプチャし、Whisper でローカル文字起こしを行い、Claude によるインテリジェント分析をリアルタイムに提供します。

<p align="center">
  <img src="https://img.shields.io/badge/platform-macOS-blue" alt="Platform: macOS">
  <img src="https://img.shields.io/badge/Tauri-v2-orange" alt="Tauri v2">
  <img src="https://img.shields.io/badge/React-19-blue" alt="React 19">
  <img src="https://img.shields.io/badge/Rust-2021-orange" alt="Rust 2021">
  <img src="https://img.shields.io/badge/license-MIT-green" alt="License: MIT">
</p>

---

## 特徴

- **完全ローカル処理** — 音声キャプチャ・文字起こし・VAD はすべてローカルで実行。外部サーバー不要
- **リアルタイム文字起こし** — 話者分離・タイムスタンプ付きで会議中にリアルタイム表示
- **AI インサイト** — キーワード検出、要約、アクションアイテム抽出、議事録自動生成
- **多言語翻訳** — 日本語↔英語のセグメント単位翻訳
- **感情・トーン分析** — 発言ごとの感情分類（6 種）とスコアリング
- **会議リンク** — キーワード類似度に基づく過去会議の自動関連付け
- **カスタム辞書** — ユーザー定義のキーワード辞書で用語管理
- **全文検索** — SQLite FTS5 による高速トランスクリプト検索
- **Markdown エクスポート** — 会議記録を構造化された Markdown で出力
- **ダークモード** — システム設定連動のテーマ切り替え
- **ミニビュー** — Always-on-top のフローティングオーバーレイ

## アーキテクチャ

```
┌─────────────────────────────────────────────────────┐
│                   MojiOkoshi App                     │
│                                                      │
│  Frontend (React 19 + TypeScript + Tailwind CSS v4)  │
│  ┌────────────┐ ┌──────────┐ ┌───────────────────┐  │
│  │ Transcript  │ │   AI     │ │  Meeting Controls │  │
│  │   Panel     │ │ Insights │ │  + Settings       │  │
│  └──────┬──────┘ └────┬─────┘ └────────┬──────────┘  │
│         └─────────────┼────────────────┘              │
│                       │ Tauri IPC (type-safe)         │
│  Backend (Rust)       │                               │
│  ┌────────────┐ ┌─────┴──────┐ ┌──────────────────┐  │
│  │   Audio     │ │  Whisper   │ │  Claude Code SDK │  │
│  │  Capture    │→│  Engine    │→│  (AI Analysis)   │  │
│  │(ScreenCap-  │ │(whisper.cpp│ │                  │  │
│  │ tureKit)    │ │+ Silero    │ │                  │  │
│  │             │ │  VAD)      │ │                  │  │
│  └─────────────┘ └────────────┘ └──────────────────┘  │
│  ┌────────────┐ ┌─────────────┐ ┌─────────────────┐  │
│  │   SQLite    │ │  Speaker    │ │     Export       │  │
│  │  (Storage   │ │ Diarization │ │   (Markdown)     │  │
│  │  + FTS5)    │ │ (pyannote)  │ │                  │  │
│  └─────────────┘ └─────────────┘ └─────────────────┘  │
└─────────────────────────────────────────────────────┘
```

## 技術スタック

| レイヤー | 技術 |
|---------|------|
| **フレームワーク** | [Tauri v2](https://v2.tauri.app/) |
| **フロントエンド** | React 19, TypeScript 5.7, Vite 6 |
| **スタイリング** | Tailwind CSS v4, shadcn/ui |
| **状態管理** | Zustand 5 |
| **仮想スクロール** | @tanstack/react-virtual |
| **バックエンド** | Rust 2021 Edition |
| **音声キャプチャ** | ScreenCaptureKit (macOS) |
| **音声認識** | whisper.cpp (large-v3-turbo) |
| **VAD** | Silero VAD |
| **話者分離** | pyannote.audio (Python subprocess) |
| **AI 分析** | Claude Code SDK |
| **データベース** | SQLite (rusqlite) + FTS5 |
| **型安全 IPC** | tauri-specta (自動 TypeScript バインディング生成) |
| **エラーハンドリング** | thiserror |
| **CI** | GitHub Actions (macOS + Ubuntu) |

## 必要要件

- **macOS** 13.0 (Ventura) 以降
- **Rust** 1.75 以降
- **Node.js** 22 以降
- **Python** 3.10 以降（pyannote.audio 話者分離用）
- **Claude Max サブスクリプション**（AI 分析機能に必要）

## セットアップ

### 1. リポジトリのクローン

```bash
git clone https://github.com/unyooon/mojiokoshi.git
cd mojiokoshi
```

### 2. フロントエンド依存のインストール

```bash
npm install
```

### 3. Rust ツールチェーンの確認

```bash
rustup update stable
rustup target add aarch64-apple-darwin  # Apple Silicon の場合
```

### 4. Python 環境のセットアップ（話者分離用）

```bash
python3 -m venv .venv
source .venv/bin/activate
pip install pyannote.audio
```

### 5. 開発サーバーの起動

```bash
npm run tauri dev
```

アプリが起動し、「MojiOkoshi」ウィンドウに `Backend v0.1.0 - ok` が表示されれば正常です。

### 6. プロダクションビルド

```bash
npm run tauri build
```

`src-tauri/target/release/bundle/` にアプリバンドルが生成されます。

## 使い方

1. **録音開始** — 録音ボタンをクリックすると macOS の画面キャプチャ権限を要求。許可後、システム音声のキャプチャが開始されます。
2. **リアルタイム文字起こし** — 音声が認識されるとトランスクリプトパネルにリアルタイム表示。話者名・タイムスタンプ付き。
3. **AI インサイト** — 右パネルでサマリー、キーワード、アクションアイテム、決定事項、トピックタイムラインなどを確認。
4. **検索** — `Cmd+F` で全文検索。ハイライト付きで該当箇所にジャンプ。
5. **翻訳** — 「翻訳」タブで EN↔JA の切り替えと翻訳を実行。
6. **エクスポート** — 「エクスポート」ボタンで Markdown 形式の議事録を生成。クリップボードコピーまたはファイル保存。
7. **ミニビュー** — フローティングオーバーレイで他のウィンドウの上にトランスクリプトを表示。
8. **設定** — `Cmd+,` で設定ダイアログ。テーマ・音声・AI・エクスポートの設定。

## 開発コマンド

### フロントエンド

```bash
npm run dev              # Vite 開発サーバー起動
npm run build            # プロダクションビルド
npm run typecheck        # TypeScript 型チェック
npm run lint             # ESLint
npm run lint:fix         # ESLint 自動修正
npm run format           # Prettier フォーマット
npm run format:check     # フォーマットチェック
npm run test             # Vitest 実行
npm run test:watch       # Vitest ウォッチモード
npm run test:coverage    # カバレッジレポート
```

### バックエンド (Rust)

```bash
cd src-tauri
cargo check              # コンパイルチェック
cargo clippy -- -D warnings  # Lint（warnings をエラー扱い）
cargo test               # テスト実行
cargo fmt --check        # フォーマットチェック
```

### Tauri

```bash
npm run tauri dev        # 開発モードでアプリ起動
npm run tauri build      # プロダクションビルド
```

## プロジェクト構成

```
mojiokoshi/
├── src/                          # フロントエンド (React + TypeScript)
│   ├── components/
│   │   ├── insights/             # AI インサイトパネル群
│   │   │   ├── InsightsPanel.tsx  #   タブ切り替えコンテナ
│   │   │   ├── SummaryCard.tsx    #   要約カード
│   │   │   ├── KeywordList.tsx    #   キーワード一覧
│   │   │   ├── ActionItemList.tsx #   アクションアイテム
│   │   │   ├── DecisionList.tsx   #   決定事項
│   │   │   ├── SpeakerPanel.tsx   #   話者情報
│   │   │   ├── TopicTimeline.tsx  #   トピックタイムライン
│   │   │   ├── TranslationPanel.tsx    # 翻訳パネル
│   │   │   ├── SentimentChart.tsx      # 感情分析チャート
│   │   │   ├── MeetingLinksPanel.tsx   # 関連会議パネル
│   │   │   └── KeywordDictionaryPanel.tsx # カスタム辞書
│   │   ├── transcript/           # 文字起こし表示
│   │   │   ├── TranscriptPanel.tsx #  仮想スクロール対応
│   │   │   ├── SegmentLine.tsx     #  セグメント行
│   │   │   ├── SearchBar.tsx       #  検索バー (Cmd+F)
│   │   │   └── SearchHighlight.tsx #  検索ハイライト
│   │   ├── meeting/              # 会議コントロール
│   │   ├── layout/               # レイアウト
│   │   ├── mini/                 # ミニビュー
│   │   └── settings/             # 設定ダイアログ
│   ├── stores/                   # Zustand ストア
│   ├── hooks/                    # カスタムフック
│   ├── types/                    # 型定義
│   └── bindings.ts               # tauri-specta 自動生成バインディング
│
├── src-tauri/                    # バックエンド (Rust)
│   └── src/
│       ├── audio/                # 音声キャプチャ (ScreenCaptureKit)
│       ├── whisper/              # 音声認識 (whisper.cpp + Silero VAD)
│       ├── claude/               # AI 分析 (Claude Code SDK)
│       ├── diarization/          # 話者分離 (pyannote.audio)
│       ├── storage/              # SQLite ストレージ
│       │   ├── sqlite.rs         #   接続管理・テーブル初期化
│       │   ├── keyword_store.rs  #   キーワードストア
│       │   ├── search_store.rs   #   FTS5 全文検索
│       │   ├── settings_store.rs #   設定永続化
│       │   ├── translation_store.rs    # 翻訳ストア
│       │   ├── sentiment_store.rs      # 感情分析ストア
│       │   ├── meeting_link_store.rs   # 会議リンクストア
│       │   └── keyword_dictionary_store.rs # カスタム辞書ストア
│       ├── export/               # エクスポート (Markdown)
│       ├── error.rs              # AppError (thiserror)
│       ├── commands.rs           # Tauri コマンド
│       └── lib.rs                # アプリエントリポイント
│
├── .github/workflows/ci.yml     # GitHub Actions CI
├── DESIGN.md                     # 詳細設計書
├── CLAUDE.md                     # コーディング規約
└── docs/                         # 追加設計ドキュメント
```

## コスト

**Claude Max サブスクリプション以外の追加課金は一切発生しません。**

| コンポーネント | コスト | 備考 |
|---|---|---|
| 音声キャプチャ | 無料 | ScreenCaptureKit（macOS 標準） |
| 音声認識 | 無料 | whisper.cpp（OSS） |
| VAD | 無料 | Silero VAD（OSS） |
| 話者分離 | 無料 | pyannote.audio（OSS） |
| AI 分析 | Max サブスク内 | Claude Code SDK |
| UI | 無料 | Tauri + React（OSS） |
| データベース | 無料 | SQLite（OSS） |

## テスト

```bash
# 全テスト実行
cd src-tauri && cargo test    # Rust: 161 tests
cd .. && npx vitest run       # Frontend: 36 tests
```

テストは以下を網羅しています:

- **ユニットテスト** — 各ストア・コマンド・ユーティリティ
- **統合テスト** — 音声→文字起こし→UI パイプライン、FTS5 検索、エクスポート、翻訳・感情分析・会議リンク・辞書の CRUD
- **フロントエンドテスト** — Zustand ストアの状態管理

## コーディング規約

詳細は [CLAUDE.md](./CLAUDE.md) を参照。

| ルール | 内容 |
|--------|------|
| Rust `unwrap()` / `expect()` | テスト以外で **禁止** |
| TypeScript `any` | **禁止**（`unknown` または適切な型を使用） |
| エラーハンドリング | `thiserror` による `AppError` 型 |
| DI パターン | Trait-based（すべて `Send + Sync`） |
| ファイル行数上限 | Rust: 200 行、React: 150 行、Store: 150 行 |
| CI 必須チェック | clippy, rustfmt, ESLint, Prettier, TypeScript, テスト |

## ライセンス

MIT

## 謝辞

- [Tauri](https://tauri.app/) — 軽量クロスプラットフォームデスクトップフレームワーク
- [whisper.cpp](https://github.com/ggerganov/whisper.cpp) — ローカル音声認識
- [pyannote.audio](https://github.com/pyannote/pyannote-audio) — 話者分離
- [Silero VAD](https://github.com/snakers4/silero-vad) — 音声区間検出
- [Claude](https://www.anthropic.com/claude) — AI 分析エンジン
