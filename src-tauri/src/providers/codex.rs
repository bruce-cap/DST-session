//! Provides Codex SQLite session listing and quick-reply resume planning.

use super::{
    codex_command, launch_cwd, status_for_command, AgentCheckContext, Provider, ResumeRequest,
    CODEX_PS1_COMMAND,
};
use crate::json_util::{compact, number_at, string_at};
use crate::model::{
    LaunchArg, LaunchPlan, ProviderCapabilities, ProviderDescriptor, SessionRecord, ShellWrap,
};
use crate::paths::{default_codex_db_path, normalize_windows_path};
use crate::providers::deepseek::invalid_record;
use crate::time::{ms_to_local_rfc3339, ms_to_rfc3339};
use serde_json::Value;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

pub struct CodexProvider;

impl Provider for CodexProvider {
    fn descriptor(&self) -> ProviderDescriptor {
        ProviderDescriptor {
            id: "codex".to_string(),
            display_name_key: "source_codex".to_string(),
            short_name: "Codex".to_string(),
            icon_key: "codex".to_string(),
            badge_key: "codex".to_string(),
            default_group_by: "workspace".to_string(),
            command_label: CODEX_PS1_COMMAND.to_string(),
            badge_text: "O".to_string(),
            capabilities: ProviderCapabilities {
                quick_reply: true,
                launcher_toggle: true,
                favorite: true,
                open_session_folder: true,
                resume: true,
                copy_command: true,
            },
        }
    }

    fn list_sessions(&self, override_path: Option<PathBuf>) -> Result<Vec<SessionRecord>, String> {
        list_sessions(override_path.unwrap_or_else(default_codex_db_path))
    }

    fn check_agent(&self, context: AgentCheckContext) -> crate::model::DeepseekStatus {
        let launcher = context.launcher.unwrap_or_else(|| "ps1".to_string());
        status_for_command(codex_command(&launcher))
    }

    fn plan_resume(&self, request: ResumeRequest) -> Result<LaunchPlan, String> {
        let launcher = request.launcher.unwrap_or_else(|| "ps1".to_string());
        let command = codex_command(&launcher);
        let mut args = vec![
            LaunchArg {
                value: "resume".to_string(),
                single_line: false,
                shell_quote: launcher == "ps1",
            },
            LaunchArg {
                value: request.session_id,
                single_line: false,
                shell_quote: launcher == "ps1",
            },
        ];
        if let Some(prompt) = request.prompt {
            args.push(LaunchArg {
                value: prompt,
                single_line: true,
                shell_quote: launcher == "ps1",
            });
        }

        Ok(LaunchPlan {
            program: command.to_string(),
            args,
            cwd: launch_cwd(request.workspace.map(|w| normalize_windows_path(&w))),
            shell_wrap: if launcher == "ps1" {
                ShellWrap::PowerShellScript
            } else {
                ShellWrap::CmdStart
            },
            prefer_windows_terminal: true,
            error_command_label: command.to_string(),
            use_call_operator: launcher == "ps1",
        })
    }
}

pub fn list_usage_records_from_db(
    db_path: Option<PathBuf>,
) -> Result<Vec<crate::usage::UsageRecord>, String> {
    read_codex_threads_for_usage(db_path.unwrap_or_else(default_codex_db_path))
}

fn list_sessions(db_path: PathBuf) -> Result<Vec<SessionRecord>, String> {
    if !db_path.exists() {
        return Err(format!(
            "Codex 数据库不存在：{}。请确认已安装 Codex CLI 并至少运行过一次，或在设置中指定自定义路径。",
            db_path.display()
        ));
    }

    let conn = rusqlite::Connection::open_with_flags(
        &db_path,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_URI,
    )
    .map_err(|error| format!("打开 Codex 数据库失败 {}: {error}", db_path.display()))?;

    let mut stmt = conn
        .prepare(
            "SELECT id, title, first_user_message, cwd, \
                    updated_at_ms, created_at_ms, rollout_path, model, tokens_used \
             FROM threads \
             WHERE archived = 0 \
             ORDER BY updated_at_ms DESC",
        )
        .map_err(|error| format!("查询 Codex threads 失败: {error}"))?;

    let rows = stmt
        .query_map([], |row| {
            Ok(CodexRow {
                id: row.get::<_, String>(0)?,
                title: row.get::<_, Option<String>>(1)?.unwrap_or_default(),
                preview: row.get::<_, Option<String>>(2)?.unwrap_or_default(),
                cwd: row.get::<_, Option<String>>(3)?.unwrap_or_default(),
                updated_at_ms: row.get::<_, Option<i64>>(4)?,
                created_at_ms: row.get::<_, Option<i64>>(5)?,
                rollout_path: row.get::<_, Option<String>>(6)?.unwrap_or_default(),
                model: row.get::<_, Option<String>>(7)?.unwrap_or_default(),
                tokens_used: row.get::<_, Option<i64>>(8)?.unwrap_or_default().max(0) as u64,
            })
        })
        .map_err(|error| format!("枚举 Codex threads 失败: {error}"))?;

    let mut records = Vec::new();
    for row in rows {
        match row {
            Ok(raw) => records.push(codex_record_from_row(raw)),
            Err(error) => records.push(invalid_record(
                "codex",
                "",
                &db_path.to_string_lossy(),
                format!("读取 Codex 线程失败: {error}"),
            )),
        }
    }

    Ok(records)
}

struct CodexRow {
    id: String,
    title: String,
    preview: String,
    cwd: String,
    updated_at_ms: Option<i64>,
    created_at_ms: Option<i64>,
    rollout_path: String,
    model: String,
    tokens_used: u64,
}

fn read_codex_threads_for_usage(
    db_path: PathBuf,
) -> Result<Vec<crate::usage::UsageRecord>, String> {
    if !db_path.exists() {
        return Err(format!(
            "Codex 数据库不存在：{}。请确认已安装 Codex CLI 并至少运行过一次，或在设置中指定自定义路径。",
            db_path.display()
        ));
    }

    let conn = rusqlite::Connection::open_with_flags(
        &db_path,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_URI,
    )
    .map_err(|error| format!("打开 Codex 数据库失败 {}: {error}", db_path.display()))?;

    let mut stmt = conn
        .prepare(
            "SELECT id, model, tokens_used, created_at_ms, updated_at_ms, archived
             FROM threads
             WHERE tokens_used > 0",
        )
        .map_err(|error| format!("查询 Codex usage threads 失败: {error}"))?;

    let rows = stmt
        .query_map([], |row| {
            let model = row.get::<_, Option<String>>(1)?.unwrap_or_default();
            let tokens_used = row.get::<_, Option<i64>>(2)?.unwrap_or_default().max(0) as u64;
            Ok(crate::usage::UsageRecord {
                source: "codex".to_string(),
                usage_id: row.get::<_, String>(0)?,
                created_at: row.get::<_, Option<i64>>(3)?.map(ms_to_local_rfc3339),
                fallback_at: row.get::<_, Option<i64>>(4)?.map(ms_to_local_rfc3339),
                model: if model.trim().is_empty() {
                    "unknown".to_string()
                } else {
                    model
                },
                total_tokens: tokens_used,
                message_count: 0,
            })
        })
        .map_err(|error| format!("枚举 Codex usage threads 失败: {error}"))?;

    collect_usage_rows(rows)
}

fn collect_usage_rows(
    rows: rusqlite::MappedRows<
        '_,
        impl FnMut(&rusqlite::Row<'_>) -> rusqlite::Result<crate::usage::UsageRecord>,
    >,
) -> Result<Vec<crate::usage::UsageRecord>, String> {
    let mut records = Vec::new();
    for row in rows {
        records.push(row.map_err(|error| format!("读取 Codex usage row 失败: {error}"))?);
    }
    Ok(records)
}

fn codex_record_from_row(row: CodexRow) -> SessionRecord {
    let workspace = normalize_windows_path(&row.cwd);
    let preview = compact(&row.preview);
    let title = if row.title.trim().is_empty() {
        if preview.is_empty() {
            "(untitled)".to_string()
        } else {
            preview.clone()
        }
    } else {
        row.title
    };
    let usage = codex_rollout_usage(Path::new(&row.rollout_path));
    let model = if row.model.trim().is_empty() {
        usage.model
    } else {
        row.model
    };
    SessionRecord {
        source: "codex".to_string(),
        short_id: row.id.chars().take(8).collect(),
        id: row.id,
        title,
        preview,
        created_at: row.created_at_ms.map(ms_to_rfc3339),
        updated_at: row.updated_at_ms.map(ms_to_rfc3339),
        message_count: usage.message_count,
        total_tokens: row.tokens_used,
        model,
        workspace,
        mode: String::new(),
        path: normalize_windows_path(&row.rollout_path),
        invalid_reason: None,
    }
}

#[derive(Default)]
struct CodexUsage {
    message_count: u64,
    total_tokens: u64,
    model: String,
}

fn codex_rollout_usage(path: &Path) -> CodexUsage {
    let Ok(content) = fs::read_to_string(path) else {
        return CodexUsage::default();
    };

    let mut usage = CodexUsage::default();
    let mut seen_usage = HashSet::new();

    for line in content.lines().filter(|line| !line.trim().is_empty()) {
        let Ok(json) = serde_json::from_str::<Value>(line) else {
            continue;
        };

        if codex_is_message_record(&json) {
            usage.message_count += 1;
        }
        if usage.model.is_empty() {
            usage.model = codex_model(&json);
        }

        let Some(last_usage) = json
            .get("payload")
            .and_then(|payload| payload.get("info"))
            .and_then(|info| info.get("last_token_usage"))
        else {
            continue;
        };

        let dedupe_key = format!(
            "{}|{}|{}",
            string_at(&json, "timestamp").unwrap_or_default(),
            usage.model,
            last_usage
        );
        if !seen_usage.insert(dedupe_key) {
            continue;
        }

        usage.total_tokens += number_at(last_usage, "total_tokens").unwrap_or_else(|| {
            number_at(last_usage, "input_tokens").unwrap_or(0)
                + number_at(last_usage, "output_tokens").unwrap_or(0)
                + number_at(last_usage, "reasoning_output_tokens").unwrap_or(0)
        });
    }

    usage
}

fn codex_model(json: &Value) -> String {
    json.get("payload")
        .and_then(|payload| payload.get("info"))
        .and_then(|info| string_at(info, "model"))
        .or_else(|| {
            json.get("payload")
                .and_then(|payload| string_at(payload, "model"))
        })
        .or_else(|| string_at(json, "model"))
        .unwrap_or_default()
}

fn codex_is_message_record(json: &Value) -> bool {
    matches!(
        json.get("type").and_then(Value::as_str),
        Some("message") | Some("user_message") | Some("assistant_message")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codex_record_from_row_uses_preview_as_fallback_title() {
        let record = codex_record_from_row(CodexRow {
            id: "thread-1".to_string(),
            title: "".to_string(),
            preview: "hello\nworld".to_string(),
            cwd: r"\\?\C:\repo".to_string(),
            updated_at_ms: Some(0),
            created_at_ms: Some(0),
            rollout_path: r"\\?\C:\rollout.jsonl".to_string(),
            model: "gpt-5".to_string(),
            tokens_used: 42,
        });
        assert_eq!(record.title, "hello world");
        assert_eq!(record.workspace, r"C:\repo");
        assert_eq!(record.path, r"C:\rollout.jsonl");
        assert_eq!(record.model, "gpt-5");
        assert_eq!(record.total_tokens, 42);
    }

    #[test]
    fn codex_record_from_row_prefers_thread_tokens_used_over_rollout_usage() {
        let dir = std::env::temp_dir().join(format!(
            "dst-session-codex-record-token-test-{}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("rollout.jsonl");
        fs::write(
            &path,
            serde_json::json!({
                "timestamp": "2026-05-12T00:00:00Z",
                "type": "event_msg",
                "payload": {
                    "type": "token_count",
                    "info": {
                        "last_token_usage": {
                            "input_tokens": 100,
                            "output_tokens": 30,
                            "total_tokens": 130
                        }
                    }
                }
            })
            .to_string(),
        )
        .unwrap();

        let record = codex_record_from_row(CodexRow {
            id: "thread-1".to_string(),
            title: "Token source".to_string(),
            preview: String::new(),
            cwd: r"C:\repo".to_string(),
            updated_at_ms: Some(0),
            created_at_ms: Some(0),
            rollout_path: path.to_string_lossy().to_string(),
            model: "gpt-5".to_string(),
            tokens_used: 42,
        });

        fs::remove_file(&path).ok();
        fs::remove_dir(&dir).ok();

        assert_eq!(record.total_tokens, 42);
    }

    #[test]
    fn list_usage_records_from_db_reads_all_positive_threads_without_rollout() {
        let dir =
            std::env::temp_dir().join(format!("dst-session-codex-db-test-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let db_path = dir.join("state_5.sqlite");
        {
            let conn = rusqlite::Connection::open(&db_path).unwrap();
            conn.execute_batch(
                "CREATE TABLE threads (
                    id TEXT NOT NULL,
                    model TEXT,
                    tokens_used INTEGER,
                    created_at_ms INTEGER,
                    updated_at_ms INTEGER,
                    archived INTEGER NOT NULL DEFAULT 0
                );
                INSERT INTO threads (id, model, tokens_used, created_at_ms, updated_at_ms, archived)
                VALUES
                    ('active-thread', 'gpt-5', 123, 0, 1000, 0),
                    ('archived-thread', '', 456, 2000, 3000, 1),
                    ('empty-thread', 'gpt-5', 0, 4000, 5000, 0);",
            )
            .unwrap();
        }

        let mut records = list_usage_records_from_db(Some(db_path.clone())).unwrap();
        records.sort_by(|left, right| left.usage_id.cmp(&right.usage_id));

        fs::remove_file(&db_path).ok();
        fs::remove_dir(&dir).ok();

        assert_eq!(records.len(), 2);
        assert_eq!(records[0].source, "codex");
        assert_eq!(records[0].usage_id, "active-thread");
        assert!(records[0].created_at.as_deref().is_some_and(|value| value
            .starts_with("1970-01-01T")
            || value.starts_with("1969-12-31T")));
        assert!(records[0].fallback_at.as_deref().is_some_and(|value| value
            .starts_with("1970-01-01T")
            || value.starts_with("1969-12-31T")));
        assert_eq!(records[0].model, "gpt-5");
        assert_eq!(records[0].total_tokens, 123);
        assert_eq!(records[0].message_count, 0);
        assert_eq!(records[1].usage_id, "archived-thread");
        assert_eq!(records[1].model, "unknown");
        assert_eq!(records[1].total_tokens, 456);
    }

    #[test]
    fn codex_rollout_usage_sums_last_token_usage_once() {
        let dir =
            std::env::temp_dir().join(format!("dst-session-codex-test-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("rollout.jsonl");
        let line = serde_json::json!({
            "timestamp": "2026-05-12T00:00:00Z",
            "type": "event_msg",
            "payload": {
                "type": "token_count",
                "info": {
                    "last_token_usage": {
                        "input_tokens": 100,
                        "cached_input_tokens": 20,
                        "output_tokens": 30,
                        "reasoning_output_tokens": 5,
                        "total_tokens": 135
                    }
                }
            }
        })
        .to_string();
        fs::write(&path, format!("{line}\n{line}\n")).unwrap();

        let usage = codex_rollout_usage(&path);

        fs::remove_file(&path).ok();
        fs::remove_dir(&dir).ok();

        assert_eq!(usage.total_tokens, 135);
    }
}
