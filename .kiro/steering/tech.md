# Tech Stack

## Runtime & Frameworks

- **Desktop shell**: Tauri 2 (Rust backend + system webview)
- **Frontend**: React 19 + TypeScript 5.9, built with Vite 7
- **Backend**: Rust (edition 2021), crates: `tauri`, `serde`, `serde_json`, `dirs`
- **Tests**: Vitest 4 (node environment, globals enabled)
- **Package manager**: pnpm (required — `pnpm-lock.yaml` is the source of truth)

## Conventions

### TypeScript / React
- `strict: true` is on. No implicit `any`, no loose casts.
- Module resolution is `Bundler`. Use ES module imports with file extensions omitted.
- Function components only. Hooks at top level. Avoid default exports for utilities; the top-level `App` is the sole default export.
- Keep pure helpers in `src/lib/*` and cover them with Vitest in co-located `*.test.ts` files.
- Types live in `src/types.ts`. Reuse exported types rather than redeclaring shapes.
- Tauri IPC uses `invoke<T>("command_name", { ... })`. Command names are snake_case; Rust returns camelCase via `#[serde(rename_all = "camelCase")]`.
- Build-time constant `__APP_VERSION__` is injected by Vite from `package.json`. Declare it in typings before use.

### Rust / Tauri
- All Tauri commands return `Result<T, String>`. Error messages are user-facing — write them in Chinese, matching existing tone.
- Never modify source session files. Only read, and only write to `app_state_path()`.
- Use `dirs::home_dir()` for cross-platform home resolution; fall back to `"."` (existing pattern).
- Path handling: prefer `PathBuf` / `Path`. Use `path.to_string_lossy()` when crossing into serialized output.
- Windows-specific launch logic lives behind `#[cfg(target_os = "windows")]`. Keep POSIX fallbacks compiling.
- Do not introduce external HTTP, telemetry, or AI-inference dependencies.

### Style
- Two-space indent everywhere (TS, JSON, Rust uses rustfmt defaults — 4 spaces).
- Quotes: double quotes in TS/TSX, Rust follows rustfmt.
- Keep commit-sized changes: small, tested, reversible.

## Common Commands

Run from the repo root. All use pnpm.

```powershell
pnpm install                                # install JS deps
pnpm dev                                    # vite dev server only (port 1420)
pnpm tauri dev                              # run full desktop app in dev
pnpm build                                  # tsc + vite build (frontend)
pnpm typecheck                              # tsc --noEmit
pnpm test -- --run                          # single-run vitest (never use watch in automation)
pnpm tauri build --debug                    # debug desktop binary
pnpm release                                # tauri build --bundles nsis
cargo check --manifest-path src-tauri/Cargo.toml   # rust-only check
```

### Command Rules for Automation

- Always pass `--run` to vitest in CI / agent contexts. Never start watch mode.
- Never run `pnpm tauri dev` or `pnpm dev` as a blocking call; they are long-running dev servers. Tell the user to start them manually when needed.
- The shell is Windows `cmd`. Use `&` to chain commands, not `&&`. Prefer separate invocations when possible.

## Environment Requirements

- Windows 10/11
- Node.js 22+
- pnpm
- Rust stable toolchain (with MSVC target)
- `deepseek-tui.cmd` on PATH (for resume launch)
- `claude` / `claude.cmd` on PATH (for Claude Code resume)
