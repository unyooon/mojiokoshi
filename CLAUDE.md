# MojiOkoshi - Project Conventions

## Architecture

- **Framework**: Tauri v2 (Rust backend) + React 19 + TypeScript (frontend)
- **Design source of truth**: `DESIGN.md`
- **Branch strategy**: `develop` for PRs, `main` for phase releases

## Rust Conventions

- `unwrap()` / `expect()` are **forbidden** outside of tests
- Use `thiserror` for error type definitions
- Trait-based dependency injection: all modules communicate through traits
- All traits require `Send + Sync`
- File limit: 200 lines (excluding tests)
- Clippy: deny `unwrap_used`, `expect_used`
- Run `cargo clippy -- -D warnings` before committing

## TypeScript Conventions

- `any` type is **forbidden** - use `unknown` or proper types
- Do not use `React.FC` - use plain functions with typed props
- ESLint `exhaustive-deps` must be satisfied
- File limits: Components 150 lines, Stores 150 lines, Hooks 120 lines
- Tests: up to 400 lines

## Frontend Patterns

- **State management**: Zustand stores
- **Styling**: Tailwind CSS v4 (CSS-first, no config file)
- **UI components**: shadcn/ui
- **Path alias**: `@/` maps to `src/`
- **Props Down / Events Up**: Store access at App level, pass via props

## Testing

- Rust: `cargo test` + `mockall` for trait mocks
- Frontend: Vitest + React Testing Library
- Tauri commands: mock `@tauri-apps/api`

## Commands

```bash
npm run lint          # ESLint
npm run format:check  # Prettier check
npm run typecheck     # TypeScript check
npm run build         # Frontend build
cd src-tauri && cargo clippy -- -D warnings  # Rust lint
cd src-tauri && cargo test   # Rust tests
```

## Agent Team Development Flow

本プロジェクトではClaude Codeのエージェントチームによる並列開発を採用する。
Claude Code のメインセッション（team-lead）は **PM（プロジェクトマネージャー）** として振る舞い、指示・タスク分解・依存関係管理のみを行う。コードの実装・テスト・レビュー・マージはすべてエージェントが担当する。

### チーム構成（5体）

| エージェント | 担当 | 責務 |
|---|---|---|
| **team-lead** | 統括・PM | タスク分解・割り当て・依存関係管理。コードは書かず指示のみ行う |
| **rust-backend** | `src-tauri/` | 音声キャプチャ、Whisper統合、Claude SDKブリッジ、SQLite、Tauriコマンド |
| **react-frontend** | `src/` | UIコンポーネント、Zustandストア、カスタムフック、型定義 |
| **infra-test** | インフラ・CI | プロジェクト初期構築、Tauri設定、lint/CI、テスト基盤 |
| **reviewer** | レビュー | 全PRレビュー、DESIGN.md整合性チェック、品質ゲート検証、GitHub PR上でのレビューコメント |

### 開発の流れ

1. **計画**: PM が DESIGN.md に基づきPRを分解し、依存関係を整理する
2. **タスク割り当て**: PM が各エージェントにタスクを割り当てる（依存関係のないタスクは並列実行）
3. **実装**: 実装エージェント（rust-backend / react-frontend / infra-test）がコードを書き、品質ゲートを通過した上でコミット・pushする
4. **PR作成**: 実装エージェントがPRを作成する
5. **レビュー**: reviewer エージェントが以下を検証し、GitHub PR上でレビューコメントを残す
   - DESIGN.md との整合性
   - 品質ゲート全通過（lint, typecheck, clippy, test, format）
   - コーディングスタンダード準拠
   - セキュリティ・スレッド安全性
6. **修正**: 指摘事項があれば実装エージェントが修正し、再度pushする
7. **承認**: reviewer エージェントが全指摘解消を確認し、Approveする
8. **マージ**: reviewer の Approve を受けて、**実装エージェントが** PR をマージする（PMはマージしない）

### 注意事項

- PM は直接ファイルを編集したり、git操作を行わない
- 品質ゲートの実行も各エージェントの責務
- マージ権限は実装エージェントにあり、reviewer の Approve が前提条件

### ブランチ戦略

```
main                    ← Phase完了時のみマージ
└── develop             ← 各PRのコミット先
```

- ブランチ名規約: `<type>/<phase>-<short-description>`（例: `feat/p1-audio-capture-trait`）
- マージ方式: Squash merge

### 品質ゲート（PRマージ必須条件）

1. 全テストパス（`cargo test` + `vitest`）
2. `cargo clippy -- -D warnings` warning ゼロ
3. `npm run lint` + `npm run typecheck` エラーゼロ
4. `npm run format:check` + `cargo fmt --check` 通過
5. reviewer エージェントによる承認
6. 1ファイル200行以下（テスト除く）
7. 新規公開APIにテストあり
8. DESIGN.md との整合性

### PR粒度ルール

| 指標 | 目安 | 上限 |
|---|---|---|
| ファイル数 | 3-8 | 15 |
| 差分行数 | 100-400行 | 800行 |
