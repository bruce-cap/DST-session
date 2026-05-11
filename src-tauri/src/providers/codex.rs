//! Provides Codex SQLite session listing and quick-reply resume planning.

use super::{status_for_command, AgentCheckContext, Provider, ResumeRequest, CODEX_COMMAND};
use crate::json_util::compact;
use crate::model::{LaunchArg, LaunchPlan, ProviderCapabilities, ProviderDescriptor, SessionRecord, ShellWrap};
use crate::paths::{default_codex_db_path, normalize_windows_path, workspace_dir};
use crate::providers::deepseek::invalid_record;
use crate::time::ms_to_rfc3339;
use std::path::PathBuf;

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
            command_label: CODEX_COMMAND.to_string(),
            badge_text: "O".to_string(),
            capabilities: ProviderCapabilities {
                quick_reply: true,
                launcher_toggle: false,
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

    fn check_agent(&self, _context: AgentCheckContext) -> crate::model::DeepseekStatus {
        status_for_command(CODEX_COMMAND)
    }

    fn plan_resume(&self, request: ResumeRequest) -> Result<LaunchPlan, String> {
        let mut args = vec![
            LaunchArg { value: "resume".to_string(), single_line: false, shell_quote: false },
            LaunchArg { value: request.session_id, single_line: false, shell_quote: true },
        ];
        if let Some(prompt) = request.prompt {
            args.push(LaunchArg { value: prompt, single_line: true, shell_quote: true });
        }

        Ok(LaunchPlan {
            program: CODEX_COMMAND.to_string(),
            args,
            cwd: Some(workspace_dir(request.workspace.map(|w| normalize_windows_path(&w))).to_string_lossy().to_string()),
            shell_wrap: ShellWrap::PowerShellScript,
            prefer_windows_terminal: true,
            error_command_label: CODEX_COMMAND.to_string(),
            use_call_operator: true,
        })
    }
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
                    updated_at_ms, created_at_ms, rollout_path \
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
}

fn codex_record_from_row(row: CodexRow) -> SessionRecord {
    let workspace = normalize_windows_path(&row.cwd);
    let preview = compact(&row.preview);
    let title = if row.title.trim().is_empty() {
        if preview.is_empty() { "(untitled)".to_string() } else { preview.clone() }
    } else {
        row.title
    };

    SessionRecord {
        source: "codex".to_string(),
        short_id: row.id.chars().take(8).collect(),
        id: row.id,
        title,
        preview,
        created_at: row.created_at_ms.map(ms_to_rfc3339),
        updated_at: row.updated_at_ms.map(ms_to_rfc3339),
        message_count: 0,
        total_tokens: 0,
        model: String::new(),
        workspace,
        mode: String::new(),
        path: normalize_windows_path(&row.rollout_path),
        invalid_reason: None,
    }
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
        });
        assert_eq!(record.title, "hello world");
        assert_eq!(record.workspace, r"C:\repo");
        assert_eq!(record.path, r"C:\rollout.jsonl");
    }
}
