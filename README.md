# Agent Session Manager

A lightweight local desktop app for browsing, filtering, and resuming agent CLI sessions from DeepSeek TUI, Claude Code, and Codex.

It turns local session files into a graphical session manager: switch sources, search and group sessions, preview metadata, favorite sessions, copy resume commands, open session folders, and launch a selected session in a new terminal.

## Supported Session Sources

| Source | Default local data | Resume command |
| --- | --- | --- |
| DeepSeek TUI | `%USERPROFILE%\.deepseek\sessions` | `deepseek.cmd resume <session-id>` or `deepseek.ps1 resume <session-id>` |
| Claude Code | `%USERPROFILE%\.claude\projects` | `claude --resume <session-id>` |
| Codex | `%USERPROFILE%\.codex\state_5.sqlite` | `codex.ps1 resume <session-id>` |

Codex also supports quick reply: type a single-line prompt in the detail panel and launch `codex.ps1 resume <session-id> "<prompt>"` directly.

## Features

- Switch between DeepSeek TUI, Claude Code, and Codex session sources from the sidebar.
- Read DeepSeek JSON sessions, Claude Code JSONL sessions, and Codex SQLite thread records from their default local paths.
- Group sessions by workspace, date, model, mode, favorites, or show all sessions.
- Search by title, first user message, workspace, model, mode, or session ID.
- Preview session metadata, file path, resume command, and first user-message summary.
- Favorite sessions without modifying the original session files.
- Copy the provider-specific resume command.
- Open the session file folder.
- Launch the selected session in a new system terminal.
- Choose the DeepSeek launcher between `deepseek.cmd` and `deepseek.ps1`.
- Toggle Chinese/English UI language and light/dark theme.
- Isolate invalid or unreadable session records instead of crashing the app.

## Safety Model

The current version treats agent session files as read-only.

The app reads:

```text
%USERPROFILE%\.deepseek\sessions
%USERPROFILE%\.claude\projects
%USERPROFILE%\.codex\state_5.sqlite
```

The app writes only its own state file for favorites and launch settings:

```text
%USERPROFILE%\.agent-session-manager\state.json
```

If you already used the old DeepSeek Session Manager name, the app copies missing `state.json` / `index.sqlite` files from the legacy `%USERPROFILE%\.deepseek-session-manager` directory into `%USERPROFILE%\.agent-session-manager` on startup, so existing favorites and settings are not lost during the rename.

UI preferences such as language and theme are stored in browser local storage inside the Tauri WebView.

The app does not upload session content, call an AI model for summaries, or modify the original DeepSeek TUI, Claude Code, or Codex session data.

## Requirements

- Windows
- Node.js 22+
- pnpm
- Rust toolchain
- At least one supported CLI installed and available on `PATH`:
  - DeepSeek TUI: `deepseek.cmd` or `deepseek.ps1`
  - Claude Code: `claude`
  - Codex: `codex.ps1`

Check the CLI commands:

```powershell
deepseek.cmd --version
deepseek.ps1 --version
claude --version
codex.ps1 --version
```

## Development

Install dependencies:

```powershell
pnpm install
```

Run the desktop app in development mode:

```powershell
pnpm tauri dev
```

Run checks:

```powershell
pnpm test -- --run
pnpm typecheck
cargo check --manifest-path src-tauri/Cargo.toml
```

Build a debug desktop binary:

```powershell
pnpm tauri build --debug
```

Build an NSIS installer:

```powershell
pnpm release
```

The debug binary is generated at:

```text
src-tauri\target\debug\agent-session-manager.exe
```

## Project Structure

```text
src/
  App.tsx                 React app shell
  api/                    Tauri IPC wrappers
  components/             UI components
  hooks/                  App state, session loading, locale, and theme hooks
  lib/                    Resume commands, session grouping/search, i18n, formatting
  styles/                 CSS tokens, layout, and component styles

src-tauri/
  src/main.rs             Tauri entrypoint and command registration
  src/commands.rs         IPC commands exposed to the UI
  src/providers/          DeepSeek, Claude Code, and Codex session providers
  src/launcher/           Platform-specific terminal launch helpers
  src/state.rs            App-owned state file read/write
  tauri.conf.json         Tauri app config
```

## Current Limitations

- The app is primarily focused on Windows.
- Resume launches in a new system terminal. Embedded terminal support is not implemented.
- Session archive, delete, rename, local notes, and AI-generated summaries are intentionally left out for now.
- Source paths are currently the default CLI locations.

## Roadmap

- Custom source path settings.
- Editable local aliases and notes.
- Safe archive and restore workflow.
- Recent launch history.
- Optional full-text search.
- Release packaging for GitHub Releases.

## License

MIT License
