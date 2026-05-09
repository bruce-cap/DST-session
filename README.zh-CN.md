# DeepSeek Session Manager

一个用于浏览、管理和快速恢复 DeepSeek TUI 与 Claude Code 会话的本地桌面应用。

DeepSeek TUI 可以通过下面的命令恢复历史会话：

```powershell
deepseek-tui.cmd resume <session-id>
```

这个项目在此基础上提供了一个图形化界面，方便你在 DeepSeek TUI 和 Claude Code 两种会话来源之间切换，搜索、分组、预览、收藏本地会话，打开对应会话文件目录，并从界面中快速继续某个会话。

## 功能特性

- 可以在顶部栏切换 DeepSeek TUI 和 Claude Code 会话来源。
- 从默认 sessions 目录读取本地 DeepSeek TUI 会话 JSON 文件。
- 从默认 projects 目录读取本地 Claude Code 会话 JSONL 文件。
- 支持按 workspace、日期、模型、模式、收藏分组，也可以显示全部会话。
- 支持按标题、首条用户消息、workspace、模型、模式或 session ID 搜索。
- 支持预览会话元数据和首条用户消息摘要。
- 支持收藏会话，且不会修改 DeepSeek 原始 session JSON。
- 支持复制恢复命令。
- 支持打开 session JSON 所在目录。
- 支持打开新的系统终端并执行 `deepseek-tui.cmd resume <session-id>` 或 `claude --resume <session-id>`。

## 安全模型

`0.1.0` 版本默认把 DeepSeek TUI 和 Claude Code 的 session 文件当作只读数据。

应用读取：

```text
%USERPROFILE%\.deepseek\sessions
%USERPROFILE%\.claude\projects
```

应用只会把收藏和启动配置写入自己的状态文件：

```text
%USERPROFILE%\.deepseek-session-manager\state.json
```

应用不会上传会话内容，不会调用 AI 模型生成摘要，也不会修改 DeepSeek TUI 或 Claude Code 原始会话文件。

## 环境要求

- Windows
- Node.js 22+
- pnpm
- Rust 工具链
- 已安装 DeepSeek TUI，并且命令行中可以使用 `deepseek-tui.cmd`
- 已安装 Claude Code，并且命令行中可以使用 `claude`

检查 DeepSeek TUI 命令是否可用：

```powershell
deepseek-tui.cmd --version
claude --version
```

## 本地开发

安装依赖：

```powershell
pnpm install
```

以开发模式启动桌面应用：

```powershell
pnpm tauri dev
```

运行检查：

```powershell
pnpm test -- --run
pnpm typecheck
cargo check --manifest-path src-tauri/Cargo.toml
```

构建调试版桌面程序：

```powershell
pnpm tauri build --debug
```

调试版程序会生成到：

```text
src-tauri\target\debug\deepseek-session-manager.exe
```

## 项目结构

```text
src/
  App.tsx              React 界面
  App.css              桌面端样式
  lib/
    commands.ts        DeepSeek 命令辅助函数
    session.ts         会话规范化、分组和搜索逻辑

src-tauri/
  src/main.rs          Tauri 命令：本地扫描、状态读写、打开目录、恢复会话
  tauri.conf.json      Tauri 应用配置

docs/
  superpowers/plans/   实现计划
```

## 当前限制

- 第一版主要面向 Windows。
- 恢复会话时会打开新的系统终端，暂未实现内嵌终端。
- `0.1.0` 暂不提供归档、删除、重命名和 AI 摘要功能。
- 当前默认扫描 DeepSeek TUI 的默认 sessions 目录。

## 后续计划

- 支持自定义 sessions 目录。
- 支持本地别名和备注。
- 支持安全归档与恢复。
- 支持最近启动记录。
- 支持可选全文搜索。
- 支持打包并发布到 GitHub Releases。

## License

当前还没有选择开源许可证。如果你希望他人可以使用、修改或再分发这个项目，发布前建议补充一个明确的许可证。
