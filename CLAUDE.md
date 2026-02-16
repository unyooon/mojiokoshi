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

### チーム構成（5体）

| エージェント | 担当 | 責務 |
|---|---|---|
| **team-lead** | 統括 | タスク分解・割り当て・依存関係管理。コードは書かない |
| **rust-backend** | `src-tauri/` | 音声キャプチャ、Whisper統合、Claude SDKブリッジ、SQLite、Tauriコマンド |
| **react-frontend** | `src/` | UIコンポーネント、Zustandストア、カスタムフック、型定義 |
| **infra-test** | インフラ・CI | プロジェクト初期構築、Tauri設定、lint/CI、テスト基盤 |
| **reviewer** | レビュー | 全PRレビュー、DESIGN.md整合性チェック、品質ゲート |

### 開発の流れ

1. **計画**: team-lead がDESIGN.mdに基づきPRを分解し、依存関係を整理する
2. **Wave実行**: 依存関係のないPRを並列でエージェントに割り当てる
3. **コミット**: 各エージェントが品質ゲートを通過した上でコミットする
4. **PR作成**: Wave/Phase完了時に `develop` → `main` のPRを作成する
5. **レビュー**: reviewer エージェントが以下を検証する
   - DESIGN.md との整合性
   - コーディングスタンダード準拠
   - 品質ゲート全通過（lint, typecheck, clippy, test, format）
   - セキュリティ・スレッド安全性
6. **マージ**: レビュー指摘を修正した上でマージする

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
