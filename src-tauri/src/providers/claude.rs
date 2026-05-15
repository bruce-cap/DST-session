//! Provides Claude JSONL session listing and resume planning.

use super::{
    claude_command, launch_cwd, status_for_command, AgentCheckContext, Provider, ResumeRequest,
    CLAUDE_PREVIEW_COMMAND,
};
use crate::json_util::{content_to_text, number_at, string_at};
use crate::model::{
    LaunchArg, LaunchPlan, ProviderCapabilities, ProviderDescriptor, SessionRecord, ShellWrap,
};
use crate::paths::{default_claude_projects_dir, file_stem, normalize_windows_path};
use crate::providers::deepseek::invalid_record;
use crate::time::system_time_to_rfc3339;
use serde_json::Value;
use std::collections::BTreeMap;
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
                launcher_toggle: true,
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

    fn check_agent(&self, context: AgentCheckContext) -> crate::model::DeepseekStatus {
        let launcher = context.launcher.unwrap_or_else(|| "cmd".to_string());
        status_for_command(claude_command(&launcher))
    }

    fn plan_resume(&self, request: ResumeRequest) -> Result<LaunchPlan, String> {
        let launcher = request.launcher.unwrap_or_else(|| "cmd".to_string());
        let command = claude_command(&launcher);
        Ok(LaunchPlan {
            program: command.to_string(),
            args: vec![
                LaunchArg {
                    value: "--resume".to_string(),
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
    let root = root.unwrap_or_else(default_claude_projects_dir);
    if !root.exists() {
        return Err(format!(
            "Claude Code projects 目录不存在：{}。请确认已安装 Claude Code，或在设置中指定自定义目录。",
            root.display()
        ));
    }

    let mut records = Vec::new();
    for project in fs::read_dir(&root)
        .map_err(|error| format!("无法读取 Claude projects 目录 {}: {error}", root.display()))?
    {
        let project = project.map_err(|error| format!("读取 Claude 项目目录失败: {error}"))?;
        let project_path = project.path();
        if project_path.is_dir() {
            collect_jsonl_usage_records(&project_path, &mut records);
        }
    }
    Ok(records)
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
                records.push(invalid_record(
                    "claude",
                    "",
                    "",
                    format!("读取 Claude 项目目录失败: {error}"),
                ));
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
                records.push(invalid_record(
                    "claude",
                    "",
                    "",
                    format!("读取 Claude 会话文件失败: {error}"),
                ));
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
            Ok(Some(record)) => records.push(record),
            Ok(None) => {}
            Err(error) => records.push(invalid_record(
                "claude",
                &file_stem(&path),
                &path.to_string_lossy(),
                error,
            )),
        }
    }
}

fn collect_jsonl_usage_records(dir: &Path, records: &mut Vec<crate::usage::UsageRecord>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };

    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if path.is_dir() {
            collect_jsonl_usage_records(&path, records);
            continue;
        }
        if !path.is_file() || path.extension().and_then(|ext| ext.to_str()) != Some("jsonl") {
            continue;
        }
        if let Ok(mut file_records) = parse_usage_file(&path) {
            records.append(&mut file_records);
        }
    }
}

fn parse_usage_file(path: &Path) -> Result<Vec<crate::usage::UsageRecord>, String> {
    let content = fs::read_to_string(path)
        .map_err(|error| format!("读取 Claude usage 文件失败 {}: {error}", path.display()))?;
    let mut created_at: Option<String> = None;
    let mut fallback_at: Option<String> = None;
    let mut message_count = 0_u64;
    let mut assistant_model_token_totals = BTreeMap::<String, TokenSplit>::new();
    let mut result_model_token_totals = BTreeMap::<String, TokenSplit>::new();

    for line in content.lines().filter(|line| !line.trim().is_empty()) {
        let Ok(json) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        if let Some(timestamp) = string_at(&json, "timestamp") {
            if created_at.is_none() {
                created_at = Some(timestamp.clone());
            }
            fallback_at = Some(timestamp);
        }
        match string_at(&json, "type").as_deref() {
            Some("user") => message_count += 1,
            Some("assistant") => {
                message_count += 1;
                if let Some(message) = json.get("message") {
                    add_model_tokens(
                        &mut assistant_model_token_totals,
                        string_at(message, "model").unwrap_or_default(),
                        claude_usage_split(message.get("usage")),
                    );
                }
            }
            Some("result") => {
                for (model, tokens) in claude_model_usage_tokens(json.get("modelUsage")) {
                    add_model_tokens(&mut result_model_token_totals, model, tokens);
                }
            }
            _ => {}
        }
    }

    if fallback_at.is_none() {
        fallback_at = fs::metadata(path)
            .ok()
            .and_then(|metadata| metadata.modified().ok())
            .map(system_time_to_rfc3339);
    }

    let model_token_totals = if assistant_model_token_totals.is_empty() {
        result_model_token_totals
    } else {
        assistant_model_token_totals
    };
    let multi_model = model_token_totals.len() > 1;
    Ok(model_token_totals
        .into_iter()
        .map(|(model, tokens)| {
            let path_id = path.to_string_lossy();
            crate::usage::UsageRecord {
                source: "claude".to_string(),
                usage_id: if multi_model {
                    format!("{path_id}#model:{model}")
                } else {
                    path_id.to_string()
                },
                created_at: created_at.clone(),
                fallback_at: fallback_at.clone(),
                model,
                input_tokens: tokens.input,
                output_tokens: tokens.output,
                total_tokens: tokens.total(),
                message_count,
            }
        })
        .collect())
}

fn parse_session_file(path: &Path) -> Result<Option<SessionRecord>, String> {
    let content = fs::read_to_string(path)
        .map_err(|error| format!("读取 Claude session 文件失败 {}: {error}", path.display()))?;
    let mut id = file_stem(path);
    let mut custom_title: Option<String> = None;
    let mut ai_title: Option<String> = None;
    let mut preview = String::new();
    let mut created_at: Option<String> = None;
    let mut updated_at: Option<String> = None;
    let mut message_count = 0_u64;
    let mut total_tokens = 0_u64;
    let mut model = String::new();
    let mut model_token_totals = BTreeMap::<String, TokenSplit>::new();
    let mut workspace = String::new();
    let mut mode = String::new();
    let mut has_real_activity = false;

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
            workspace = string_at(&json, "cwd")
                .map(|value| normalize_windows_path(&value))
                .unwrap_or_default();
        }
        if mode.is_empty() {
            mode = string_at(&json, "permissionMode").unwrap_or_default();
        }

        match string_at(&json, "type").as_deref() {
            Some("custom-title") => {
                if let Some(title) = string_at(&json, "customTitle").and_then(non_empty_compact) {
                    custom_title = Some(title);
                }
            }
            Some("ai-title") => {
                if let Some(title) = string_at(&json, "aiTitle").and_then(non_empty_compact) {
                    ai_title = Some(title);
                }
            }
            Some("user") => {
                has_real_activity = true;
                message_count += 1;
                if !json
                    .get("isCompactSummary")
                    .and_then(Value::as_bool)
                    .unwrap_or(false)
                {
                    let text = clean_claude_user_text(&claude_message_text(json.get("message")));
                    if !is_noise_user_text(&text) && preview.is_empty() {
                        preview = text;
                    }
                }
            }
            Some("assistant") => {
                has_real_activity = true;
                message_count += 1;
                if let Some(message) = json.get("message") {
                    let message_model = string_at(message, "model").unwrap_or_default();
                    if model.is_empty() && !message_model.is_empty() {
                        model = message_model.clone();
                    }
                    let tokens = claude_usage_split(message.get("usage"));
                    total_tokens += tokens.total();
                    add_model_tokens(&mut model_token_totals, message_model, tokens);
                }
            }
            Some("result") => {
                has_real_activity = true;
                for (usage_model, tokens) in claude_model_usage_tokens(json.get("modelUsage")) {
                    if model.is_empty() && !usage_model.is_empty() {
                        model = usage_model.clone();
                    }
                    total_tokens += tokens.total();
                    add_model_tokens(&mut model_token_totals, usage_model, tokens);
                }
            }
            _ => {}
        }
    }

    if !has_real_activity {
        return Ok(None);
    }

    let title = custom_title
        .or(ai_title)
        .or_else(|| non_empty_compact(preview.clone()))
        .unwrap_or_else(|| "(untitled)".to_string());
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
            .map(|value| normalize_windows_path(&value))
            .unwrap_or_default();
    }
    if let Some((dominant_model, _)) = model_token_totals
        .iter()
        .max_by(|left, right| left.1.total().cmp(&right.1.total()).then_with(|| right.0.cmp(left.0)))
    {
        model = dominant_model.clone();
    }

    Ok(Some(SessionRecord {
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
    }))
}

fn claude_message_text(message: Option<&Value>) -> String {
    let Some(message) = message else {
        return String::new();
    };
    content_to_text(message.get("content"))
}

fn non_empty_compact(value: String) -> Option<String> {
    let value = crate::json_util::compact(&value);
    (!value.is_empty()).then_some(value)
}

fn clean_claude_user_text(text: &str) -> String {
    let mut text = text.to_string();
    for tag in [
        "system-reminder",
        "ide_opened_file",
        "local-command-caveat",
        "local-command-stdout",
        "command-name",
        "command-message",
        "command-args",
    ] {
        text = strip_tagged_blocks(&text, tag);
    }
    crate::json_util::compact(&text)
}

fn strip_tagged_blocks(text: &str, tag: &str) -> String {
    let open = format!("<{tag}>");
    let close = format!("</{tag}>");
    let mut remaining = text;
    let mut output = String::new();

    while let Some(start) = remaining.find(&open) {
        output.push_str(&remaining[..start]);
        let after_open = &remaining[start + open.len()..];
        let Some(end) = after_open.find(&close) else {
            remaining = "";
            break;
        };
        remaining = &after_open[end + close.len()..];
    }

    output.push_str(remaining);
    output
}

fn is_noise_user_text(text: &str) -> bool {
    if text.is_empty() {
        return true;
    }
    if text.starts_with('/') && text.split_whitespace().count() == 1 {
        return true;
    }
    [
        "Base directory for this skill:",
        "Launching skill:",
        "Tool loaded.",
        "Todos have been modified successfully.",
    ]
    .iter()
    .any(|prefix| text.starts_with(prefix))
}

#[derive(Debug, Clone, Copy, Default)]
struct TokenSplit {
    input: u64,
    output: u64,
}

impl TokenSplit {
    fn total(self) -> u64 {
        self.input + self.output
    }
}

fn claude_usage_split(usage: Option<&Value>) -> TokenSplit {
    let Some(usage) = usage else {
        return TokenSplit::default();
    };

    TokenSplit {
        input: number_at(usage, "input_tokens").unwrap_or(0),
        output: number_at(usage, "output_tokens").unwrap_or(0),
    }
}

fn claude_model_usage_tokens(model_usage: Option<&Value>) -> Vec<(String, TokenSplit)> {
    let Some(Value::Object(models)) = model_usage else {
        return Vec::new();
    };

    models
        .iter()
        .filter_map(|(model, usage)| {
            let tokens = TokenSplit {
                input: number_at(usage, "inputTokens").unwrap_or(0),
                output: number_at(usage, "outputTokens").unwrap_or(0),
            };
            (tokens.total() > 0).then(|| (model.clone(), tokens))
        })
        .collect()
}

fn add_model_tokens(model_totals: &mut BTreeMap<String, TokenSplit>, model: String, tokens: TokenSplit) {
    if model.is_empty() || tokens.total() == 0 {
        return;
    }
    let entry = model_totals.entry(model).or_default();
    entry.input += tokens.input;
    entry.output += tokens.output;
}

pub fn decode_claude_project_dir(name: &str) -> String {
    let trimmed = name.strip_prefix('-').unwrap_or(name);

    if let Some((drive, rest)) = trimmed.split_once("--") {
        if drive.len() == 1 && drive.chars().next().unwrap().is_ascii_alphabetic() {
            return format!(
                "{}:\\{}",
                drive.to_ascii_uppercase(),
                rest.replace('-', "\\")
            );
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
    use serde_json::json;

    #[test]
    fn decode_claude_project_dir_never_panics_for_edge_inputs() {
        assert_eq!(decode_claude_project_dir(""), "");
        assert_eq!(decode_claude_project_dir("---"), "///");
        assert_eq!(decode_claude_project_dir("plain"), "plain");
    }

    #[test]
    fn claude_usage_tokens_excludes_cache_tokens_from_total() {
        let usage = json!({
            "input_tokens": 100,
            "output_tokens": 20,
            "cache_read_input_tokens": 1_000,
            "cache_creation_input_tokens": 500
        });

        assert_eq!(claude_usage_split(Some(&usage)).total(), 120);
    }

    #[test]
    fn parse_usage_file_prefers_assistant_usage_over_result_aggregate() {
        let dir = std::env::temp_dir().join(format!(
            "dst-session-claude-usage-dedupe-test-{}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("session.jsonl");
        let lines = [
            json!({
                "type": "assistant",
                "timestamp": "2026-05-11T10:01:00Z",
                "message": {
                    "model": "claude-opus-4-6",
                    "usage": { "input_tokens": 100, "output_tokens": 20 }
                }
            })
            .to_string(),
            json!({
                "type": "result",
                "timestamp": "2026-05-11T10:02:00Z",
                "modelUsage": {
                    "claude-opus-4-6": { "inputTokens": 100, "outputTokens": 20 }
                }
            })
            .to_string(),
        ];
        fs::write(&path, lines.join("\n")).unwrap();

        let records = parse_usage_file(&path).unwrap();

        fs::remove_file(&path).ok();
        fs::remove_dir(&dir).ok();

        assert_eq!(records.len(), 1);
        assert_eq!(records[0].model, "claude-opus-4-6");
        assert_eq!(records[0].total_tokens, 120);
    }

    #[test]
    fn list_usage_records_includes_subagents_and_groups_tokens_by_model() {
        let root = std::env::temp_dir().join(format!(
            "dst-session-claude-usage-test-{}",
            std::process::id()
        ));
        let project = root.join("project");
        let subagents = project.join("subagents");
        fs::create_dir_all(&subagents).unwrap();
        let main_path = project.join("main.jsonl");
        let subagent_path = subagents.join("worker.jsonl");
        let main_lines = [
            json!({
                "type": "user",
                "timestamp": "2026-05-11T10:00:00Z",
                "message": { "content": "hello" }
            })
            .to_string(),
            json!({
                "type": "assistant",
                "timestamp": "2026-05-11T10:01:00Z",
                "message": {
                    "model": "claude-opus-4-6",
                    "usage": {
                        "input_tokens": 100,
                        "output_tokens": 20,
                        "cache_read_input_tokens": 1_000,
                        "cache_creation_input_tokens": 500
                    }
                }
            })
            .to_string(),
            json!({
                "type": "assistant",
                "timestamp": "2026-05-11T10:02:00Z",
                "message": {
                    "model": "claude-sonnet-4-6",
                    "usage": {
                        "input_tokens": 30,
                        "output_tokens": 5
                    }
                }
            })
            .to_string(),
        ];
        let subagent_lines = [json!({
            "type": "assistant",
            "timestamp": "2026-05-12T10:00:00Z",
            "message": {
                "model": "claude-haiku-4-5",
                "usage": {
                    "input_tokens": 7,
                    "output_tokens": 3
                }
            }
        })
        .to_string()];
        fs::write(&main_path, main_lines.join("\n")).unwrap();
        fs::write(&subagent_path, subagent_lines.join("\n")).unwrap();

        let mut records = list_usage_records(Some(root.clone())).unwrap();
        records.sort_by(|left, right| left.usage_id.cmp(&right.usage_id));

        fs::remove_file(&main_path).ok();
        fs::remove_file(&subagent_path).ok();
        fs::remove_dir(&subagents).ok();
        fs::remove_dir(&project).ok();
        fs::remove_dir(&root).ok();

        assert_eq!(records.len(), 3);
        assert!(records
            .iter()
            .any(|record| record.usage_id.contains("subagents")
                && !record.usage_id.contains("#model:")));
        assert!(
            records
                .iter()
                .filter(|record| record.usage_id.contains("main.jsonl#model:"))
                .count()
                == 2
        );
        assert!(records.iter().any(|record| {
            record.model == "claude-opus-4-6"
                && record.total_tokens == 120
                && record.message_count == 3
                && record.created_at.as_deref() == Some("2026-05-11T10:00:00Z")
                && record.fallback_at.as_deref() == Some("2026-05-11T10:02:00Z")
        }));
        assert!(records
            .iter()
            .any(|record| { record.model == "claude-sonnet-4-6" && record.total_tokens == 35 }));
        assert!(records
            .iter()
            .any(|record| { record.model == "claude-haiku-4-5" && record.total_tokens == 10 }));
    }

    #[test]
    fn list_sessions_skips_permission_mode_only_placeholder_files() {
        let root = std::env::temp_dir().join(format!(
            "dst-session-claude-sessions-test-{}",
            std::process::id()
        ));
        let project = root.join("project");
        let subagents = project.join("subagents");
        fs::create_dir_all(&subagents).unwrap();
        let valid_path = project.join("valid.jsonl");
        let placeholder_path = project.join("placeholder.jsonl");
        let subagent_path = subagents.join("worker.jsonl");

        fs::write(
            &valid_path,
            [
                json!({
                    "type": "permission-mode",
                    "permissionMode": "default",
                    "sessionId": "valid-session"
                })
                .to_string(),
                json!({
                    "type": "user",
                    "sessionId": "valid-session",
                    "timestamp": "2026-05-12T10:00:00Z",
                    "message": { "content": "hello" }
                })
                .to_string(),
            ]
            .join("\n"),
        )
        .unwrap();
        fs::write(
            &placeholder_path,
            json!({
                "type": "permission-mode",
                "permissionMode": "default",
                "sessionId": "placeholder-session"
            })
            .to_string(),
        )
        .unwrap();
        fs::write(
            &subagent_path,
            json!({
                "type": "user",
                "sessionId": "subagent-session",
                "timestamp": "2026-05-12T10:01:00Z",
                "message": { "content": "subagent" }
            })
            .to_string(),
        )
        .unwrap();

        let records = list_sessions(root.clone()).unwrap();

        fs::remove_file(&valid_path).ok();
        fs::remove_file(&placeholder_path).ok();
        fs::remove_file(&subagent_path).ok();
        fs::remove_dir(&subagents).ok();
        fs::remove_dir(&project).ok();
        fs::remove_dir(&root).ok();

        assert_eq!(records.len(), 1);
        assert_eq!(records[0].id, "valid-session");
        assert_eq!(records[0].message_count, 1);
    }

    #[test]
    fn parse_session_file_sums_assistant_and_result_usage_without_cache() {
        let dir =
            std::env::temp_dir().join(format!("dst-session-claude-test-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("session.jsonl");
        let lines = [
            json!({
                "type": "user",
                "sessionId": "session-1",
                "timestamp": "2026-05-10T23:30:00.000Z",
                "cwd": r"C:\repo",
                "permissionMode": "default",
                "message": { "content": "hello" }
            })
            .to_string(),
            json!({
                "type": "assistant",
                "timestamp": "2026-05-10T23:31:00.000Z",
                "message": {
                    "id": "resp-1",
                    "model": "claude-opus-4-6",
                    "usage": {
                        "input_tokens": 100,
                        "output_tokens": 20,
                        "cache_read_input_tokens": 1_000,
                        "cache_creation_input_tokens": 500
                    }
                }
            })
            .to_string(),
            json!({
                "type": "result",
                "timestamp": "2026-05-10T23:32:00.000Z",
                "modelUsage": {
                    "gpt-5.5": {
                        "inputTokens": 30,
                        "outputTokens": 4,
                        "cacheReadInputTokens": 2_000,
                        "cacheCreationInputTokens": 200
                    },
                    "claude-opus-4-6": {
                        "inputTokens": 10,
                        "outputTokens": 1,
                        "cacheReadInputTokens": 3_000,
                        "cacheCreationInputTokens": 300
                    }
                }
            })
            .to_string(),
        ];
        fs::write(&path, lines.join("\n")).unwrap();

        let record = parse_session_file(&path).unwrap().expect("valid session record");

        fs::remove_file(&path).ok();
        fs::remove_dir(&dir).ok();

        assert_eq!(record.id, "session-1");
        assert_eq!(
            record.created_at.as_deref(),
            Some("2026-05-10T23:30:00.000Z")
        );
        assert_eq!(
            record.updated_at.as_deref(),
            Some("2026-05-10T23:32:00.000Z")
        );
        assert_eq!(record.message_count, 2);
        assert_eq!(record.total_tokens, 165);
        assert_eq!(record.model, "claude-opus-4-6");
    }


    #[test]
    fn parse_session_file_prefers_custom_title_then_ai_title() {
        let dir = std::env::temp_dir().join(format!(
            "dst-session-claude-title-priority-test-{}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("session.jsonl");
        let lines = [
            json!({
                "type": "user",
                "sessionId": "session-title-priority",
                "timestamp": "2026-05-12T10:00:00.000Z",
                "message": { "content": "真实用户问题" }
            })
            .to_string(),
            json!({
                "type": "ai-title",
                "sessionId": "session-title-priority",
                "aiTitle": "AI generated title"
            })
            .to_string(),
            json!({
                "type": "custom-title",
                "sessionId": "session-title-priority",
                "customTitle": "User renamed title"
            })
            .to_string(),
        ];
        fs::write(&path, lines.join("\n")).unwrap();

        let record = parse_session_file(&path)
            .unwrap()
            .expect("valid session record");

        fs::remove_file(&path).ok();
        fs::remove_dir(&dir).ok();

        assert_eq!(record.title, "User renamed title");
        assert_eq!(record.preview, "真实用户问题");
    }

    #[test]
    fn parse_session_file_uses_latest_ai_title_without_custom_title() {
        let dir = std::env::temp_dir().join(format!(
            "dst-session-claude-ai-title-test-{}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("session.jsonl");
        let lines = [
            json!({
                "type": "ai-title",
                "sessionId": "session-ai-title",
                "aiTitle": "Old title"
            })
            .to_string(),
            json!({
                "type": "user",
                "sessionId": "session-ai-title",
                "timestamp": "2026-05-12T10:00:00.000Z",
                "message": { "content": "真实用户问题" }
            })
            .to_string(),
            json!({
                "type": "ai-title",
                "sessionId": "session-ai-title",
                "aiTitle": "New title"
            })
            .to_string(),
        ];
        fs::write(&path, lines.join("\n")).unwrap();

        let record = parse_session_file(&path)
            .unwrap()
            .expect("valid session record");

        fs::remove_file(&path).ok();
        fs::remove_dir(&dir).ok();

        assert_eq!(record.title, "New title");
        assert_eq!(record.preview, "真实用户问题");
    }

    #[test]
    fn parse_session_file_cleans_context_tags_for_preview_fallback_title() {
        let dir = std::env::temp_dir().join(format!(
            "dst-session-claude-title-clean-test-{}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("session.jsonl");
        let lines = [
            json!({
                "type": "user",
                "sessionId": "session-clean-title",
                "timestamp": "2026-05-12T10:00:00.000Z",
                "message": {
                    "content": "<ide_opened_file>The user opened a file.</ide_opened_file> 写一版中文 CLAUDE.md"
                }
            })
            .to_string(),
            json!({
                "type": "assistant",
                "timestamp": "2026-05-12T10:01:00.000Z",
                "message": {
                    "model": "claude-opus-4-6",
                    "usage": { "input_tokens": 10, "output_tokens": 2 }
                }
            })
            .to_string(),
        ];
        fs::write(&path, lines.join("\n")).unwrap();

        let record = parse_session_file(&path)
            .unwrap()
            .expect("valid session record");

        fs::remove_file(&path).ok();
        fs::remove_dir(&dir).ok();

        assert_eq!(record.title, "写一版中文 CLAUDE.md");
        assert_eq!(record.preview, "写一版中文 CLAUDE.md");
    }

    #[test]
    fn parse_session_file_skips_noisy_user_text_for_title_and_preview() {
        let dir = std::env::temp_dir().join(format!(
            "dst-session-claude-title-noise-test-{}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("session.jsonl");
        let lines = [
            json!({
                "type": "user",
                "sessionId": "session-noise-title",
                "timestamp": "2026-05-12T10:00:00.000Z",
                "message": { "content": "<local-command-caveat>ignore this</local-command-caveat>" }
            })
            .to_string(),
            json!({
                "type": "user",
                "sessionId": "session-noise-title",
                "timestamp": "2026-05-12T10:00:01.000Z",
                "message": { "content": "/plugin" }
            })
            .to_string(),
            json!({
                "type": "user",
                "sessionId": "session-noise-title",
                "timestamp": "2026-05-12T10:00:02.000Z",
                "isCompactSummary": true,
                "message": { "content": "This session is being continued from a previous conversation." }
            })
            .to_string(),
            json!({
                "type": "user",
                "sessionId": "session-noise-title",
                "timestamp": "2026-05-12T10:00:03.000Z",
                "message": { "content": "帮我实现标题读取" }
            })
            .to_string(),
        ];
        fs::write(&path, lines.join("\n")).unwrap();

        let record = parse_session_file(&path)
            .unwrap()
            .expect("valid session record");

        fs::remove_file(&path).ok();
        fs::remove_dir(&dir).ok();

        assert_eq!(record.title, "帮我实现标题读取");
        assert_eq!(record.preview, "帮我实现标题读取");
        assert_eq!(record.message_count, 4);
    }

    #[test]
    fn parse_session_file_counts_user_tool_result_messages() {
        let dir = std::env::temp_dir().join(format!(
            "dst-session-claude-tool-result-test-{}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("session.jsonl");
        let lines = [
            json!({
                "type": "user",
                "sessionId": "session-tool-result",
                "timestamp": "2026-05-12T10:00:00.000Z",
                "message": {
                    "content": [{
                        "type": "tool_result",
                        "content": [{
                            "type": "text",
                            "text": "tool result only"
                        }]
                    }]
                }
            })
            .to_string(),
            json!({
                "type": "assistant",
                "timestamp": "2026-05-12T10:01:00.000Z",
                "message": {
                    "model": "claude-opus-4-6",
                    "usage": {
                        "input_tokens": 10,
                        "output_tokens": 2
                    }
                }
            })
            .to_string(),
        ];
        fs::write(&path, lines.join("\n")).unwrap();

        let record = parse_session_file(&path)
            .unwrap()
            .expect("valid session record");

        fs::remove_file(&path).ok();
        fs::remove_dir(&dir).ok();

        assert_eq!(record.id, "session-tool-result");
        assert_eq!(record.message_count, 2);
        assert_eq!(record.total_tokens, 12);
    }
}
