# Agent Session Manager V0.1 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a local Tauri + React desktop app that reads DeepSeek TUI session JSON files, shows them in a searchable GUI, supports favorites, opens session file folders, and launches `deepseek resume <session-id>`.

**Architecture:** Rust owns trusted local operations: scanning session JSON, reading/writing the app state file, opening folders, checking and launching `deepseek`. React owns presentation state: grouping, searching, selected session, and command preview. The original DeepSeek session JSON files remain read-only in V0.1.

**Tech Stack:** Tauri 2, Rust, React, TypeScript, Vite, Vitest, CSS.

---

### Task 1: Project Skeleton

**Files:**
- Create: `package.json`
- Create: `index.html`
- Create: `tsconfig.json`
- Create: `tsconfig.node.json`
- Create: `vite.config.ts`
- Create: `src/main.tsx`
- Create: `src/App.tsx`
- Create: `src/App.css`
- Create: `src/vite-env.d.ts`
- Create: `src-tauri/Cargo.toml`
- Create: `src-tauri/tauri.conf.json`
- Create: `src-tauri/build.rs`
- Create: `src-tauri/src/main.rs`
- Create: `.gitignore`

- [ ] **Step 1: Create a minimal Tauri + React project structure**

Use root-level React/Vite files and `src-tauri` for Rust. Keep existing PRD and PowerShell script in the repository root.

- [ ] **Step 2: Install dependencies**

Run:

```powershell
pnpm install
```

Expected: dependency installation completes with a generated lockfile.

- [ ] **Step 3: Verify dev commands are wired**

Run:

```powershell
pnpm typecheck
pnpm test -- --run
```

Expected: typecheck succeeds and Vitest exits successfully once tests exist.

### Task 2: Session Parsing Core

**Files:**
- Create: `src/lib/session.ts`
- Create: `src/lib/session.test.ts`

- [ ] **Step 1: Write failing tests for session normalization**

Tests cover: metadata parsing, title fallback from first user text block, search matching, grouping key generation, and local time formatting.

- [ ] **Step 2: Run tests and confirm RED**

Run:

```powershell
pnpm test -- --run src/lib/session.test.ts
```

Expected: fails because `src/lib/session.ts` does not exist yet.

- [ ] **Step 3: Implement the minimal TypeScript session helpers**

Implement pure frontend helpers only: `normalizeSession`, `matchesSession`, `groupSessions`, `formatDateTime`, `formatTokenCount`.

- [ ] **Step 4: Run tests and confirm GREEN**

Run:

```powershell
pnpm test -- --run src/lib/session.test.ts
```

Expected: all session helper tests pass.

### Task 3: Tauri Command Layer

**Files:**
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/src/main.rs`
- Create: `src/types.ts`

- [ ] **Step 1: Add Rust commands**

Commands:

```text
list_sessions(sessions_dir?: string) -> Vec<SessionRecord>
get_app_state() -> AppState
set_favorite(session_id: String, favorite: bool) -> AppState
open_session_folder(path: String) -> Result<(), String>
check_deepseek() -> DeepseekStatus
resume_session(session_id: String, workspace?: String, launch_mode?: String) -> Result<(), String>
```

- [ ] **Step 2: Keep original sessions read-only**

Only `get_app_state` and `set_favorite` write to `%USERPROFILE%\.agent-session-manager\state.json`. Do not write to `%USERPROFILE%\.deepseek\sessions`.

- [ ] **Step 3: Verify Rust compilation**

Run:

```powershell
pnpm tauri build --debug
```

Expected: Rust commands compile.

### Task 4: React GUI

**Files:**
- Modify: `src/App.tsx`
- Modify: `src/App.css`
- Modify: `src/main.tsx`
- Modify: `src/types.ts`

- [ ] **Step 1: Implement the three-zone interface**

Top toolbar: search, group selector, refresh, deepseek status.

Main layout: left grouping sidebar, center session list, right details.

- [ ] **Step 2: Implement session actions**

Actions:

```text
Refresh
Select session
Favorite/unfavorite
Copy resume command
Open JSON folder
Resume session
```

- [ ] **Step 3: Implement safe error states**

Show readable UI messages for missing `deepseek`, missing workspace, invalid session files, and empty search results.

### Task 5: Verification

**Files:**
- No new files unless fixes require them.

- [ ] **Step 1: Run frontend tests**

Run:

```powershell
pnpm test -- --run
```

Expected: all Vitest tests pass.

- [ ] **Step 2: Run TypeScript check**

Run:

```powershell
pnpm typecheck
```

Expected: no TypeScript errors.

- [ ] **Step 3: Run Rust check**

Run:

```powershell
cargo check --manifest-path src-tauri/Cargo.toml
```

Expected: no Rust errors.

- [ ] **Step 4: Run Tauri debug build**

Run:

```powershell
pnpm tauri build --debug
```

Expected: debug desktop bundle builds successfully.
