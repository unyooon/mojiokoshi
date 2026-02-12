# MojiOkoshi エージェントチーム実装計画

## Context

MojiOkoshiプロジェクトは設計フェーズが完了し（DESIGN.md 1,003行）、コード実装が未開始の状態。GitHub PR-based ワークフローで高品質に実装を進めるための、エージェントチーム構成と運用ルールを定義する。

---

## 1. エージェント構成（5体）

| エージェント | 担当 | 責務 |
|---|---|---|
| **team-lead** | プロジェクト統括 | タスク分解→割り当て、依存関係管理、コンフリクト制御。自らはコードを書かない |
| **rust-backend** | `src-tauri/` 全般 | 音声キャプチャ、Whisper統合、Claude SDK ブリッジ、SQLite、Tauriコマンド + Rustテスト |
| **react-frontend** | `src/` 全般 | UIコンポーネント、Zustandストア、カスタムフック、型定義 + フロントテスト |
| **infra-test** | インフラ・テスト・CI | プロジェクト初期構築、Tauri設定、lint/CI設定、テスト基盤、統合テスト |
| **reviewer** | コードレビュー | 全PRレビュー、DESIGN.md整合性チェック、品質ゲート判定、マージ実行 |

### 作業フロー

```
team-lead: タスク分解 → 割り当て
    ├──→ rust-backend / react-frontend / infra-test: 実装+テスト → PR作成
    └──→ reviewer: レビュー → 承認→マージ / 修正依頼→修正→再レビュー
```

---

## 2. ブランチ戦略

```
main                    ← Phase完了時のみマージ
└── develop             ← PRのマージ先
    ├── feat/p1-audio-capture-trait
    ├── feat/p1-transcript-panel
    └── ...
```

- ブランチ名: `<type>/<phase>-<short-description>` (例: `feat/p1-audio-capture-trait`)
- マージ方式: Squash merge
- Phase完了時に develop → main

---

## 3. PR粒度ルール

| 指標 | 目安 | 上限 |
|---|---|---|
| ファイル数 | 3-8 | 15 |
| 差分行数 | 100-400行 | 800行 |

### PRタイトル規則

```
<type>(<scope>): <description>
```

- type: `feat` / `fix` / `refactor` / `test` / `infra` / `docs`
- scope: `audio` / `whisper` / `claude` / `storage` / `commands` / `transcript` / `insights` / `meeting` / `settings` / `stores` / `hooks` / `types` / `ci` / `tauri`

---

## 4. 疎結合アーキテクチャ

### Rust: trait ベースの依存性注入

```rust
// 全モジュール間はtrait経由。具体実装間の直接依存は禁止
pub trait AudioCapture: Send + Sync { ... }
pub trait SpeechRecognizer: Send + Sync { ... }
pub trait VoiceActivityDetector: Send + Sync { ... }
pub trait AiAnalyzer: Send + Sync { ... }
pub trait SessionStorage: Send + Sync { ... }

// main.rs で AppState に Box<dyn Trait> として組み立て
// commands.rs は AppState 経由でtrait メソッドを呼び出す
```

### React: Props Down / Events Up

- ストアへの直接アクセスは `App.tsx` レベルのみ
- 末端コンポーネントは props のみに依存
- Tauri呼び出しはカスタムフック経由

### 型共有: tauri-specta で自動生成

- Rust の `#[derive(specta::Type)]` → TypeScript型を自動生成
- 手動の型二重管理を排除

### ファイルサイズ制限

| 種別 | 上限行数 |
|---|---|
| Rust モジュール | 200行 |
| React コンポーネント | 150行 |
| Zustand ストア | 150行 |
| カスタムフック | 120行 |
| テスト | 400行 |

---

## 5. テスト戦略

| 領域 | ツール | 方針 |
|---|---|---|
| Rust | `cargo test` + mockall | trait のモック実装でテスト。macOS API テストは `#[ignore]` |
| React | Vitest + React Testing Library | コンポーネント/ストア/フックごとにテスト |
| Tauri連携 | `@tauri-apps/api` モック | invoke/listen をモック |
| CI | GitHub Actions | PR作成時に lint + test + typecheck 自動実行 |

### カバレッジ目標

- Phase 1: Rust 60% / React 50%
- Phase 2: Rust 70% / React 60%
- Phase 3+: Rust 80% / React 70%

---

## 6. 品質ゲート（PRマージ必須条件）

1. 全テストパス（CI）
2. `cargo clippy -- -D warnings` warning ゼロ（CI）
3. `npm run lint` + `tsc --noEmit` エラーゼロ（CI）
4. reviewer の承認（1 approve 以上）
5. 1ファイル200行以下（テスト除く）
6. 新規公開API にテストあり
7. DESIGN.md との整合性

### コーディングスタンダード

- Rust: `unwrap()` / `expect()` はテスト以外禁止、`thiserror` でエラー型定義
- TS: `any` 型禁止、`React.FC` 不使用、ESLint exhaustive-deps 遵守

---

## 7. Phase 1 の具体的PRリスト（20 PR）

### Phase 0: 初期セットアップ（infra-test のみ、直列）

| # | PR | 担当 |
|---|---|---|
| 1 | `infra(tauri): scaffold Tauri v2 project with React + TS + Tailwind + shadcn` | infra-test |
| 2 | `infra(ci): add linters and GitHub Actions CI` | infra-test |
| 3 | `infra(tauri): add directory structure and shared type scaffolding` | infra-test |

### Phase 1 第1波（並列）

| # | PR | 担当 | 依存 |
|---|---|---|---|
| 4 | `feat(audio): add AudioCapture trait and audio types` | rust-backend | #3 |
| 5 | `feat(types): add core TypeScript domain types` | react-frontend | #3 |
| 6 | `test(setup): add Vitest config and Rust test helpers` | infra-test | #3 |

### Phase 1 第2波（並列）

| # | PR | 担当 | 依存 |
|---|---|---|---|
| 7 | `feat(audio): implement ScreenCaptureKit capture` | rust-backend | #4 |
| 8 | `feat(stores): implement transcriptStore with Zustand` | react-frontend | #5 |
| 9 | `feat(meeting): implement MeetingControls and Timer` | react-frontend | #5 |

### Phase 1 第3波（並列）

| # | PR | 担当 | 依存 |
|---|---|---|---|
| 10 | `feat(whisper): add SpeechRecognizer trait and whisper-rs engine` | rust-backend | #4 |
| 11 | `feat(whisper): implement Silero VAD and audio pipeline` | rust-backend | #10 |
| 12 | `feat(transcript): implement TranscriptPanel with virtual scroll` | react-frontend | #8 |
| 13 | `feat(storage): add SessionStorage trait and SQLite implementation` | rust-backend | #3 |

### Phase 1 第4波（並列）

| # | PR | 担当 | 依存 |
|---|---|---|---|
| 14 | `feat(commands): add Tauri commands for capture and transcription` | rust-backend | #7,#11,#13 |
| 15 | `feat(hooks): implement useTauriEvents for transcript streaming` | react-frontend | #8,#14 |
| 16 | `feat(layout): implement main layout with resizable panels` | react-frontend | #12 |
| 17 | `feat(meeting): implement session management UI` | react-frontend | #9,#15 |
| 18 | `feat(tauri): add macOS permission onboarding flow` | rust-backend | #7 |
| 19 | `test(p1): add integration tests for audio-to-transcript pipeline` | infra-test | #14 |
| 20 | `feat(stores): implement settingsStore and basic settings UI` | react-frontend | #5 |

### 並行度の可視化

```
Wave 0: [infra-test: PR#1] → [infra-test: PR#2] → [infra-test: PR#3]
Wave 1: [rust: PR#4] | [react: PR#5] | [infra: PR#6]
Wave 2: [rust: PR#7] | [react: PR#8, #9]
Wave 3: [rust: PR#10, #11, #13] | [react: PR#12]
Wave 4: [rust: PR#14, #18] | [react: PR#15, #16, #17, #20] | [infra: PR#19]
```

---

## 8. `.claude/agents/reviewer.md` 設定

reviewer エージェント用のカスタム指示を `.claude/agents/reviewer.md` に配置し、レビュー基準を定義する。

---

## 9. Phase完了マイルストーン

| Phase | 完了条件 |
|---|---|
| Phase 0 | `cargo build` + `npm run dev` + `npm run tauri dev` で起動 |
| Phase 1 | 音声キャプチャ → Whisper文字起こし → UI表示が動作 |
| Phase 2 | Claude Code SDK でキーワード抽出・要約が動作 |
| Phase 3 | アクションアイテム、議事録、ミニビューが動作 |
| Phase 4 | ダークモード、検索、エクスポートが動作 |

---

## 関連ファイル

- `/Users/yuta/Code/Anemonet/mojiokoshi/DESIGN.md` - 全設計仕様（唯一の真実の源泉）
- `/Users/yuta/Code/Anemonet/mojiokoshi/docs/04_ui_ux_design.md` - UI/UX詳細設計
- `/Users/yuta/Code/Anemonet/mojiokoshi/docs/design/claude-api-integration-design.md` - Claude統合参考資料
