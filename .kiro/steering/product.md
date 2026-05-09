# Product

DeepSeek Session Manager is a lightweight local desktop app (Windows-first) for browsing and resuming DeepSeek TUI and Claude Code sessions.

## Purpose

DeepSeek TUI and Claude Code store each conversation as a local file and can resume via CLI:

- DeepSeek TUI: `deepseek-tui.cmd resume <session-id>`
- Claude Code: `claude --resume <session-id>`

This app gives those sessions a graphical manager so users don't have to memorize IDs or dig through JSON files.

## Core Capabilities

- Switch between DeepSeek TUI and Claude Code as the session source.
- Scan local session files:
  - DeepSeek: `%USERPROFILE%\.deepseek\sessions\*.json`
  - Claude Code: `%USERPROFILE%\.claude\projects\**\*.jsonl`
- List, search, and group sessions (workspace, date, model, mode, favorites, all).
- Preview session metadata and the first user message.
- Favorite sessions, copy the resume command, open the session file folder.
- Launch a new terminal window running the resume command in the session's workspace.

## Safety Model

- Treat all source session files as **read-only**. Never modify, move, or delete them.
- Only write to the app's own state file: `%USERPROFILE%\.deepseek-session-manager\state.json` (favorites and launch settings).
- Do not upload session content, call remote AI services for summaries, or send telemetry. All processing stays on the local machine.
- Destructive features (archive, delete, rename, AI summaries) are intentionally out of scope for v0.1.

## Scope Boundaries

- Windows is the primary target. macOS/Linux paths exist in code but are not the focus.
- Resume always opens a new system terminal. Embedded terminals are a future feature.
- UI copy is primarily Simplified Chinese; keep user-facing strings consistent with existing style.
