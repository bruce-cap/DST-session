# DeepSeek Session Manager

A lightweight local desktop app for browsing and resuming DeepSeek TUI sessions.

DeepSeek TUI can resume a conversation with a command such as:

```powershell
deepseek-tui.cmd resume <session-id>
```

This app gives those local sessions a graphical manager: search, group, preview, favorite, open the JSON folder, and resume a session from the UI.

## Features

- Read local DeepSeek TUI session JSON files from the default sessions directory.
- Group sessions by workspace, date, model, mode, favorites, or show all sessions.
- Search by title, first user message, workspace, model, mode, or session ID.
- Preview session metadata and the first user-message summary.
- Favorite sessions without modifying the original DeepSeek session JSON.
- Copy the resume command.
- Open the session JSON folder.
- Launch a new terminal window and run `deepseek-tui.cmd resume <session-id>`.

## Safety Model

Version `0.1.0` treats DeepSeek TUI session files as read-only.

The app reads:

```text
%USERPROFILE%\.deepseek\sessions
```

The app writes only its own state file for favorites and launch settings:

```text
%USERPROFILE%\.deepseek-session-manager\state.json
```

It does not upload session content, call an AI model for summaries, or modify the original DeepSeek TUI JSON files.

## Requirements

- Windows
- Node.js 22+
- pnpm
- Rust toolchain
- DeepSeek TUI installed and available as `deepseek-tui.cmd`

Check the DeepSeek TUI command:

```powershell
deepseek-tui.cmd --version
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

The debug binary is generated at:

```text
src-tauri\target\debug\deepseek-session-manager.exe
```

## Project Structure

```text
src/
  App.tsx              React UI
  App.css              Desktop styling
  lib/
    commands.ts        DeepSeek command helpers
    session.ts         Session normalization, grouping, search helpers

src-tauri/
  src/main.rs          Tauri commands for local file scans, state, folder open, resume launch
  tauri.conf.json      Tauri app config

docs/
  superpowers/plans/   Implementation plan
```

## Current Limitations

- The first version is focused on Windows.
- Resume launches in a new system terminal. Embedded terminal support is not implemented yet.
- Session archive, delete, rename, and AI-generated summaries are intentionally left out of `0.1.0`.
- The app currently scans the default DeepSeek sessions directory.

## Roadmap

- Custom sessions directory setting.
- Editable local aliases and notes.
- Safe archive and restore workflow.
- Recent launch history.
- Optional full-text search.
- Release packaging for GitHub Releases.

## License

No license has been selected yet. Add one before publishing if you want others to use, modify, or redistribute the project.
