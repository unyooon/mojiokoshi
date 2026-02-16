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
