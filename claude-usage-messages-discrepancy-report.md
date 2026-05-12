# Claude Usage Messages 统计差异调查报告

## 背景

当前 usage token 界面中，Claude 的 Messages 数量显示为 `9,982`，而 Claude 官方 app 显示为 `8,010`，两者相差较大。

本次只做原因排查，不修改代码。

## 结论

主要原因是：**当前应用的 Claude usage 聚合把 `.claude/projects` 下的 `subagents` JSONL 会话也计入了 Messages，而 Claude 官方 app 大概率不把 subagent 会话计入顶层 Messages 口径。**

排除 `subagents` 后，本地扫描结果约为 `7,924`，已经非常接近官方 app 的 `8,010`。包含 `subagents` 时会多出约 `2,086` 条 messages，这基本解释了当前 `9,982` 与官方数字之间的主要差距。

## 代码路径

Usage 页面展示的 Messages 来自后端 SQLite usage 聚合结果：

- 前端读取并展示：`src/components/UsagePage.tsx:45-48`
- 后端汇总读取：`src-tauri/src/index.rs`
- Claude usage 刷新入口：`src-tauri/src/usage.rs:85-89`
- Claude usage JSONL 解析：`src-tauri/src/providers/claude.rs:82-101`

Claude provider 的 usage 扫描逻辑会递归遍历所有 `.jsonl` 文件：

- `src-tauri/src/providers/claude.rs:191-205`

这段递归没有跳过 `subagents` 目录，因此 subagent 会话会被纳入 usage 统计。

## Messages 当前统计口径

在 Claude usage 解析中，每个有 token 的 JSONL 文件会按以下规则统计 `message_count`：

- `type = "user"`：+1
- `type = "assistant"`：+1
- `type = "result"`：只用于 token 聚合，不增加 message 数

对应代码：

- `src-tauri/src/providers/claude.rs:211-249`

之后聚合逻辑会按 session/date/model 写入 SQLite：

- `src-tauri/src/usage.rs:94-149`

## 关键发现

普通 Claude session 列表扫描时，代码明确跳过了 `subagents` 目录：

- `src-tauri/src/providers/claude.rs:168-171`

但 Claude usage 扫描没有跳过 `subagents`。这导致：

- 会话列表口径：不含 subagents
- usage messages 口径：包含 subagents

此外，测试中还明确覆盖了“usage 包含 subagents”的行为：

- `src-tauri/src/providers/claude.rs:532`

说明这不是意外递归，而是当前代码已有测试认可的统计口径。

## 本地只读统计结果

对本机 `.claude/projects` 做只读扫描后得到：

| 口径 | Messages |
| --- | ---: |
| 当前 SQLite 中 Claude messages | 9,982 |
| 按当前代码口径直接扫描，包含 subagents | 约 10,010 |
| 排除 subagents 后扫描 | 约 7,924 |
| Claude 官方 app | 8,010 |

`7,924` 与官方 `8,010` 只差 `86`；而包含 subagents 后会增加约 `2,086` 条 messages。由此判断，主要差异来自 subagent 统计口径。

## 剩余小差异的可能原因

排除 subagents 后仍与官方 app 相差约 `86`，可能原因包括：

1. 官方 app 可能包含某些当前扫描未计入的特殊记录。
2. 官方 app 对 user-only、placeholder、tool/system-derived 记录的处理口径可能不同。
3. 当前 SQLite usage 聚合可能不是最新刷新结果。
4. Claude 官方 app 可能使用服务端或内部归一化后的统计口径，而不是完全等同于本地 JSONL 行计数。

## 建议方向

如果目标是尽量对齐 Claude 官方 app，后续可以考虑：

1. Claude usage 扫描默认跳过 `subagents` 目录。
2. 同步调整相关测试中“usage includes subagents”的预期。
3. 在修改后刷新 Claude usage，并再次与官方 app 数字对比。

本报告仅记录调查结论，未对代码做任何修改。
