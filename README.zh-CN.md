# DeepSeek Session Manager

一个用于浏览、筛选和快速恢复本地 Agent CLI 会话的轻量桌面应用，当前支持 DeepSeek TUI、Claude Code 和 Codex。

它会把本地会话文件整理成图形化会话管理器：切换来源、搜索和分组会话、预览元数据、收藏会话、复制恢复命令、打开会话目录，并在新的系统终端中继续选中的会话。

## 支持的会话来源

| 来源 | 默认本地数据 | 恢复命令 |
| --- | --- | --- |
| DeepSeek TUI | `%USERPROFILE%\.deepseek\sessions` | `deepseek.cmd resume <session-id>` 或 `deepseek.ps1 resume <session-id>` |
| Claude Code | `%USERPROFILE%\.claude\projects` | `claude --resume <session-id>` |
| Codex | `%USERPROFILE%\.codex\state_5.sqlite` | `codex.ps1 resume <session-id>` |

Codex 还支持快速回复：在详情面板输入单行 prompt 后，可以直接启动 `codex.ps1 resume <session-id> "<prompt>"`。

## 功能特性

- 可以从侧边栏切换 DeepSeek TUI、Claude Code 和 Codex 会话来源。
- 从默认本地路径读取 DeepSeek JSON 会话、Claude Code JSONL 会话和 Codex SQLite thread 记录。
- 支持按 workspace、日期、模型、模式、收藏分组，也可以显示全部会话。
- 支持按标题、首条用户消息、workspace、模型、模式或 session ID 搜索。
- 支持预览会话元数据、文件路径、恢复命令和首条用户消息摘要。
- 支持收藏会话，且不会修改原始会话文件。
- 支持复制当前来源对应的恢复命令。
- 支持打开会话文件所在目录。
- 支持在新的系统终端中启动选中的会话。
- 支持在 `deepseek.cmd` 和 `deepseek.ps1` 之间切换 DeepSeek 启动脚本。
- 支持中英文界面和亮色/暗色主题。
- 对无法解析或读取失败的会话记录进行隔离显示，避免应用崩溃。

## 安全模型

`0.1.0` 版本默认把 Agent 会话文件当作只读数据。

应用读取：

```text
%USERPROFILE%\.deepseek\sessions
%USERPROFILE%\.claude\projects
%USERPROFILE%\.codex\state_5.sqlite
```

应用只会把收藏和启动配置写入自己的状态文件：

```text
%USERPROFILE%\.deepseek-session-manager\state.json
```

语言和主题等 UI 偏好会保存在 Tauri WebView 内的浏览器 local storage 中。

应用不会上传会话内容，不会调用 AI 模型生成摘要，也不会修改 DeepSeek TUI、Claude Code 或 Codex 的原始会话数据。

## 环境要求

- Windows
- Node.js 22+
- pnpm
- Rust 工具链
- 至少安装一个受支持的 CLI，并确保命令可在 `PATH` 中访问：
  - DeepSeek TUI：`deepseek.cmd` 或 `deepseek.ps1`
  - Claude Code：`claude`
  - Codex：`codex.ps1`

检查 CLI 命令是否可用：

```powershell
deepseek.cmd --version
deepseek.ps1 --version
claude --version
codex.ps1 --version
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

构建 NSIS 安装包：

```powershell
pnpm release
```

调试版程序会生成到：

```text
src-tauri\target\debug\deepseek-session-manager.exe
```

## 项目结构

```text
src/
  App.tsx                 React 应用外壳
  api/                    Tauri IPC 封装
  components/             UI 组件
  hooks/                  应用状态、会话加载、语言和主题 hooks
  lib/                    恢复命令、会话分组/搜索、i18n、格式化逻辑
  styles/                 CSS tokens、布局和组件样式

src-tauri/
  src/main.rs             Tauri 入口和命令注册
  src/commands.rs         暴露给前端的 IPC 命令
  src/providers/          DeepSeek、Claude Code 和 Codex 会话来源
  src/launcher/           平台相关的终端启动逻辑
  src/state.rs            应用自有状态文件读写
  tauri.conf.json         Tauri 应用配置
```

## 当前限制

- 当前主要面向 Windows。
- 恢复会话时会打开新的系统终端，暂未实现内嵌终端。
- `0.1.0` 暂不提供归档、删除、重命名、本地备注和 AI 摘要功能。
- 会话来源路径目前使用各 CLI 的默认位置。

## 后续计划

- 支持自定义会话来源路径。
- 支持本地别名和备注。
- 支持安全归档与恢复。
- 支持最近启动记录。
- 支持可选全文搜索。
- 支持打包并发布到 GitHub Releases。

## License

当前还没有选择开源许可证。如果你希望他人可以使用、修改或再分发这个项目，发布前建议补充一个明确的许可证。
