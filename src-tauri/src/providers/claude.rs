//! Provides Claude JSONL session listing and resume planning.

use super::{status_for_command, AgentCheckContext, Provider, ResumeRequest, CLAUDE_CODE_COMMAND, CLAUDE_PREVIEW_COMMAND};
use crate::json_util::{content_to_text, number_at, string_at};
use crate::model::{LaunchArg, LaunchPlan, ProviderCapabilities, ProviderDescriptor, SessionRecord, ShellWrap};
use crate::paths::{default_claude_projects_dir, file_stem, normalize_windows_path, workspace_dir};
use crate::providers::deepseek::invalid_record;
use crate::time::system_time_to_rfc3339;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

pub struct ClaudeProvider;

impl Provider for ClaudeProvider {
    fn descriptor(&self) -> ProviderDescriptor {
        ProviderDescriptor {
            id: "claude".to_string(),
            display_name_key: "source_claude".to_string(),
            short_name: "Claude Code".to_string(),
            icon_key: "claude".to_string(),
            badge_key: "claude".to_string(),
            default_group_by: "workspace".to_string(),
            command_label: CLAUDE_PREVIEW_COMMAND.to_string(),
            badge_text: "C".to_string(),
            capabilities: ProviderCapabilities {
                quick_reply: false,
                launcher_toggle: false,
                favorite: true,
                open_session_folder: true,
                resume: true,
                copy_command: true,
            },
        }
    }

    fn list_sessions(&self, override_path: Option<PathBuf>) -> Result<Vec<SessionRecord>, String> {
        list_sessions(override_path.unwrap_or_else(default_claude_projects_dir))
    }

    fn check_agent(&self, _context: AgentCheckContext) -> crate::model::DeepseekStatus {
        status_for_command(CLAUDE_CODE_COMMAND)
    }

    fn plan_resume(&self, request: ResumeRequest) -> Result<LaunchPlan, String> {
        Ok(LaunchPlan {
            program: CLAUDE_CODE_COMMAND.to_string(),
            args: vec![
                LaunchArg { value: "--resume".to_string(), single_line: false, shell_quote: false },
                LaunchArg { value: request.session_id, single_line: false, shell_quote: false },
            ],
            cwd: Some(workspace_dir(request.workspace.map(|w| normalize_windows_path(&w))).to_string_lossy().to_string()),
            shell_wrap: ShellWrap::CmdStart,
            prefer_windows_terminal: true,
            error_command_label: CLAUDE_CODE_COMMAND.to_string(),
            use_call_operator: false,
        })
    }
}

fn list_sessions(dir: PathBuf) -> Result<Vec<SessionRecord>, String> {
    if !dir.exists() {
        return Err(format!(
            "Claude Code projects 目录不存在：{}。请确认已安装 Claude Code，或在设置中指定自定义目录。",
            dir.display()
        ));
    }

    let mut records = Vec::new();
    for project in fs::read_dir(&dir)
        .map_err(|error| format!("无法读取 Claude projects 目录 {}: {error}", dir.display()))?
    {
        let project = match project {
            Ok(project) => project,
            Err(error) => {
                records.push(invalid_record("claude", "", "", format!("读取 Claude 项目目录失败: {error}")));
                continue;
            }
        };

        let project_path = project.path();
        if project_path.is_dir() {
            collect_jsonl_sessions(&project_path, &mut records);
        }
    }

    records.sort_by(|left, right| right.updated_at.cmp(&left.updated_at));
    Ok(records)
}

fn collect_jsonl_sessions(dir: &Path, records: &mut Vec<SessionRecord>) {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(error) => {
            records.push(invalid_record(
                "claude",
                &file_stem(dir),
                &dir.to_string_lossy(),
                format!("读取 Claude 会话目录失败: {error}"),
            ));
            return;
        }
    };

    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(error) => {
                records.push(invalid_record("claude", "", "", format!("读取 Claude 会话文件失败: {error}")));
                continue;
            }
        };

        let path = entry.path();
        if path.is_dir() {
            if path.file_name().and_then(|name| name.to_str()) != Some("subagents") {
                collect_jsonl_sessions(&path, records);
            }
            continue;
        }
        if !path.is_file() || path.extension().and_then(|ext| ext.to_str()) != Some("jsonl") {
            continue;
        }

        match parse_session_file(&path) {
            Ok(record) => records.push(record),
            Err(error) => records.push(invalid_record(
                "claude",
                &file_stem(&path),
                &path.to_string_lossy(),
                error,
            )),
        }
    }
}

fn parse_session_file(path: &Path) -> Result<SessionRecord, String> {
    let content = fs::read_to_string(path)
        .map_err(|error| format!("读取 Claude session 文件失败 {}: {error}", path.display()))?;
    let mut id = file_stem(path);
    let mut title = String::new();
    let mut preview = String::new();
    let mut created_at: Option<String> = None;
    let mut updated_at: Option<String> = None;
    let mut message_count = 0_u64;
    let mut total_tokens = 0_u64;
    let mut model = String::new();
    let mut workspace = String::new();
    let mut mode = String::new();

    for line in content.lines().filter(|line| !line.trim().is_empty()) {
        let Ok(json) = serde_json::from_str::<Value>(line) else {
            continue;
        };

        if let Some(session_id) = string_at(&json, "sessionId") {
            id = session_id;
        }
        if let Some(timestamp) = string_at(&json, "timestamp") {
            if created_at.is_none() {
                created_at = Some(timestamp.clone());
            }
            updated_at = Some(timestamp);
        }
        if workspace.is_empty() {
            workspace = string_at(&json, "cwd").unwrap_or_default();
        }
        if mode.is_empty() {
            mode = string_at(&json, "permissionMode").unwrap_or_default();
        }

        match string_at(&json, "type").as_deref() {
            Some("user") => {
                let text = claude_message_text(json.get("message"));
                if !text.is_empty() {
                    message_count += 1;
                    if preview.is_empty() {
                        preview = text.clone();
                    }
                    if title.is_empty() {
                        title = text;
                    }
                }
            }
            Some("assistant") => {
                message_count += 1;
                if let Some(message) = json.get("message") {
                    if model.is_empty() {
                        model = string_at(message, "model").unwrap_or_default();
                    }
                    total_tokens += claude_usage_tokens(message.get("usage"));
                }
            }
            _ => {}
        }
    }

    if title.is_empty() {
        title = "(untitled)".to_string();
    }
    if updated_at.is_none() {
        updated_at = fs::metadata(path)
            .ok()
            .and_then(|metadata| metadata.modified().ok())
            .map(system_time_to_rfc3339);
    }
    if workspace.is_empty() {
        workspace = path
            .parent()
            .and_then(Path::file_name)
            .and_then(|name| name.to_str())
            .map(decode_claude_project_dir)
            .unwrap_or_default();
    }

    Ok(SessionRecord {
        source: "claude".to_string(),
        short_id: id.chars().take(8).collect(),
        id,
        title,
        preview,
        created_at,
        updated_at,
        message_count,
        total_tokens,
        model,
        workspace,
        mode,
        path: path.to_string_lossy().to_string(),
        invalid_reason: None,
    })
}

fn claude_message_text(message: Option<&Value>) -> String {
    let Some(message) = message else {
        return String::new();
    };
    content_to_text(message.get("content"))
}

fn claude_usage_tokens(usage: Option<&Value>) -> u64 {
    let Some(usage) = usage else {
        return 0;
    };

    number_at(usage, "input_tokens").unwrap_or(0)
        + number_at(usage, "output_tokens").unwrap_or(0)
        + number_at(usage, "cache_creation_input_tokens").unwrap_or(0)
        + number_at(usage, "cache_read_input_tokens").unwrap_or(0)
}

pub fn decode_claude_project_dir(name: &str) -> String {
    let trimmed = name.strip_prefix('-').unwrap_or(name);

    if let Some((drive, rest)) = trimmed.split_once("--") {
        if drive.len() == 1 && drive.chars().next().unwrap().is_ascii_alphabetic() {
            return format!("{}:\\{}", drive.to_ascii_uppercase(), rest.replace('-', "\\"));
        }
    }

    if name.starts_with('-') {
        format!("/{}", trimmed.replace('-', "/"))
    } else {
        name.replace('-', "/")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_claude_project_dir_never_panics_for_edge_inputs() {
        assert_eq!(decode_claude_project_dir(""), "");
        assert_eq!(decode_claude_project_dir("---"), "///");
        assert_eq!(decode_claude_project_dir("plain"), "plain");
    }
}
