//! Provides DeepSeek JSON session listing and resume planning.

use super::{
    deepseek_command, launch_cwd, status_for_command, AgentCheckContext, Provider, ResumeRequest,
};
use crate::json_util::{content_to_text, number_at, string_at};
use crate::model::{
    LaunchArg, LaunchPlan, ProviderCapabilities, ProviderDescriptor, SessionRecord, ShellWrap,
};
use crate::paths::{default_sessions_dir, file_stem, normalize_windows_path};
use crate::state::normalize_deepseek_launcher;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

pub struct DeepseekProvider;

impl Provider for DeepseekProvider {
    fn descriptor(&self) -> ProviderDescriptor {
        ProviderDescriptor {
            id: "deepseek".to_string(),
            display_name_key: "source_deepseek".to_string(),
            short_name: "DeepSeek TUI".to_string(),
            icon_key: "deepseek".to_string(),
            badge_key: "deepseek".to_string(),
            default_group_by: "workspace".to_string(),
            command_label: "deepseek.cmd".to_string(),
            badge_text: String::new(),
            capabilities: ProviderCapabilities {
                quick_reply: false,
                launcher_toggle: true,
                favorite: true,
                open_session_folder: true,
                resume: true,
                copy_command: true,
            },
        }
    }

    fn list_sessions(&self, override_path: Option<PathBuf>) -> Result<Vec<SessionRecord>, String> {
        list_sessions(override_path.unwrap_or_else(default_sessions_dir))
    }

    fn check_agent(&self, context: AgentCheckContext) -> crate::model::DeepseekStatus {
        let launcher = normalize_deepseek_launcher(context.launcher);
        status_for_command(deepseek_command(&launcher))
    }

    fn plan_resume(&self, request: ResumeRequest) -> Result<LaunchPlan, String> {
        let launcher = normalize_deepseek_launcher(request.launcher);
        let command = deepseek_command(&launcher);
        Ok(LaunchPlan {
            program: command.to_string(),
            args: vec![
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
            ],
            cwd: launch_cwd(request.workspace.map(|w| normalize_windows_path(&w))),
            shell_wrap: if launcher == "ps1" {
                ShellWrap::PowerShellScript
            } else {
                ShellWrap::CmdStart
            },
            prefer_windows_terminal: true,
            error_command_label: command.to_string(),
            use_call_operator: false,
        })
    }
}

pub fn list_usage_records(root: Option<PathBuf>) -> Result<Vec<crate::usage::UsageRecord>, String> {
    let dir = root.unwrap_or_else(default_sessions_dir);
    if !dir.exists() {
        return Err(format!(
            "DeepSeek sessions 目录不存在：{}。请确认已安装 DeepSeek TUI，或在设置中指定自定义目录。",
            dir.display()
        ));
    }

    let mut records = Vec::new();
    let entries = fs::read_dir(&dir)
        .map_err(|error| format!("无法读取 sessions 目录 {}: {error}", dir.display()))?;
    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|ext| ext.to_str()) == Some("json") {
            if let Ok(record) = read_deepseek_metadata_usage(&path) {
                records.push(record);
            }
        }
    }
    Ok(records)
}

fn list_sessions(dir: PathBuf) -> Result<Vec<SessionRecord>, String> {
    if !dir.exists() {
        return Err(format!(
            "DeepSeek sessions 目录不存在：{}。请确认已安装 DeepSeek TUI，或在设置中指定自定义目录。",
            dir.display()
        ));
    }

    let mut records = Vec::new();
    let entries = fs::read_dir(&dir)
        .map_err(|error| format!("无法读取 sessions 目录 {}: {error}", dir.display()))?;

    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(error) => {
                records.push(invalid_record(
                    "deepseek",
                    "",
                    "",
                    format!("读取目录项失败: {error}"),
                ));
                continue;
            }
        };

        let path = entry.path();
        if !path.is_file() || path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }

        match parse_session_file(&path) {
            Ok(record) => records.push(record),
            Err(error) => records.push(invalid_record(
                "deepseek",
                &file_stem(&path),
                &path.to_string_lossy(),
                error,
            )),
        }
    }

    records.sort_by(|left, right| right.updated_at.cmp(&left.updated_at));
    Ok(records)
}

fn read_deepseek_metadata_usage(path: &Path) -> Result<crate::usage::UsageRecord, String> {
    let content = fs::read_to_string(path)
        .map_err(|error| format!("读取 DeepSeek usage 文件失败 {}: {error}", path.display()))?;
    let json: Value = serde_json::from_str(&content)
        .map_err(|error| format!("JSON 解析失败 {}: {error}", path.display()))?;
    let metadata = json.get("metadata").unwrap_or(&Value::Null);
    let model = string_at(metadata, "model").unwrap_or_default();

    Ok(crate::usage::UsageRecord {
        source: "deepseek".to_string(),
        usage_id: string_at(metadata, "id").unwrap_or_else(|| file_stem(path)),
        created_at: string_at(metadata, "created_at"),
        fallback_at: string_at(metadata, "updated_at"),
        model: if model.trim().is_empty() {
            "unknown".to_string()
        } else {
            model
        },
        total_tokens: number_at(metadata, "total_tokens").unwrap_or(0),
        message_count: number_at(metadata, "message_count").unwrap_or(0),
    })
}

fn parse_session_file(path: &Path) -> Result<SessionRecord, String> {
    let content = fs::read_to_string(path)
        .map_err(|error| format!("读取 session 文件失败 {}: {error}", path.display()))?;
    let json: Value = serde_json::from_str(&content)
        .map_err(|error| format!("JSON 解析失败 {}: {error}", path.display()))?;
    let metadata = json.get("metadata").unwrap_or(&Value::Null);
    let id = string_at(metadata, "id").unwrap_or_else(|| file_stem(path));
    let preview = first_user_text(json.get("messages").and_then(Value::as_array));
    let title = string_at(metadata, "title")
        .filter(|title| !title.trim().is_empty())
        .unwrap_or_else(|| {
            if preview.is_empty() {
                "(untitled)".to_string()
            } else {
                preview.clone()
            }
        });

    Ok(SessionRecord {
        source: "deepseek".to_string(),
        short_id: id.chars().take(8).collect(),
        id,
        title,
        preview,
        created_at: string_at(metadata, "created_at"),
        updated_at: string_at(metadata, "updated_at"),
        message_count: number_at(metadata, "message_count").unwrap_or_else(|| {
            json.get("messages")
                .and_then(Value::as_array)
                .map_or(0, |m| m.len() as u64)
        }),
        total_tokens: number_at(metadata, "total_tokens").unwrap_or(0),
        model: string_at(metadata, "model").unwrap_or_default(),
        workspace: string_at(metadata, "workspace")
            .map(|value| normalize_windows_path(&value))
            .unwrap_or_default(),
        mode: string_at(metadata, "mode").unwrap_or_default(),
        path: path.to_string_lossy().to_string(),
        invalid_reason: None,
    })
}

pub fn invalid_record(source: &str, id: &str, path: &str, reason: String) -> SessionRecord {
    let id = if id.is_empty() {
        "(invalid)".to_string()
    } else {
        id.to_string()
    };

    SessionRecord {
        source: source.to_string(),
        short_id: id.chars().take(8).collect(),
        id: id.clone(),
        title: format!("无法解析: {id}"),
        preview: String::new(),
        created_at: None,
        updated_at: None,
        message_count: 0,
        total_tokens: 0,
        model: String::new(),
        workspace: String::new(),
        mode: String::new(),
        path: path.to_string(),
        invalid_reason: Some(reason),
    }
}

fn first_user_text(messages: Option<&Vec<Value>>) -> String {
    let Some(messages) = messages else {
        return String::new();
    };

    for message in messages {
        if message.get("role").and_then(Value::as_str) == Some("user") {
            let text = content_to_text(message.get("content"));
            if !text.is_empty() {
                return text;
            }
        }
    }

    String::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn list_usage_records_reads_metadata_only() {
        let dir = std::env::temp_dir().join(format!(
            "dst-session-deepseek-usage-test-{}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("session-file.json");
        fs::write(
            &path,
            json!({
                "metadata": {
                    "id": "session-1",
                    "created_at": "2026-05-10T01:00:00Z",
                    "updated_at": "2026-05-11T01:00:00Z",
                    "model": "deepseek-v3",
                    "total_tokens": 321,
                    "message_count": 7
                },
                "messages": [
                    {"role": "assistant", "usage": {"total_tokens": 99999}}
                ]
            })
            .to_string(),
        )
        .unwrap();

        let records = list_usage_records(Some(dir.clone())).unwrap();

        fs::remove_file(&path).ok();
        fs::remove_dir(&dir).ok();

        assert_eq!(records.len(), 1);
        assert_eq!(records[0].source, "deepseek");
        assert_eq!(records[0].usage_id, "session-1");
        assert_eq!(records[0].created_at.as_deref(), Some("2026-05-10T01:00:00Z"));
        assert_eq!(records[0].fallback_at.as_deref(), Some("2026-05-11T01:00:00Z"));
        assert_eq!(records[0].model, "deepseek-v3");
        assert_eq!(records[0].total_tokens, 321);
        assert_eq!(records[0].message_count, 7);
    }
}
