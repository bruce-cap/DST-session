use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

const DEEPSEEK_CMD_COMMAND: &str = "deepseek.cmd";
const DEEPSEEK_PS1_COMMAND: &str = "deepseek.ps1";
const CLAUDE_CODE_COMMAND: &str = "claude.cmd";
const CODEX_COMMAND: &str = "codex.ps1";

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SessionRecord {
    source: String,
    id: String,
    short_id: String,
    title: String,
    preview: String,
    created_at: Option<String>,
    updated_at: Option<String>,
    message_count: u64,
    total_tokens: u64,
    model: String,
    workspace: String,
    mode: String,
    path: String,
    invalid_reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AppState {
    favorites: Vec<String>,
    launch_mode: String,
    #[serde(default = "default_deepseek_launcher")]
    deepseek_launcher: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct DeepseekStatus {
    available: bool,
    version: String,
    message: String,
}

#[tauri::command]
fn list_sessions(
    source: Option<String>,
    sessions_dir: Option<String>,
) -> Result<Vec<SessionRecord>, String> {
    match source.as_deref().unwrap_or("deepseek") {
        "claude" => list_claude_sessions(
            sessions_dir
                .map(PathBuf::from)
                .unwrap_or_else(default_claude_projects_dir),
        ),
        "codex" => list_codex_sessions(
            sessions_dir
                .map(PathBuf::from)
                .unwrap_or_else(default_codex_db_path),
        ),
        _ => list_deepseek_sessions(
            sessions_dir
                .map(PathBuf::from)
                .unwrap_or_else(default_sessions_dir),
        ),
    }
}

fn list_deepseek_sessions(dir: PathBuf) -> Result<Vec<SessionRecord>, String> {
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

fn list_claude_sessions(dir: PathBuf) -> Result<Vec<SessionRecord>, String> {
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
        if !project_path.is_dir() {
            continue;
        }

        let files = match fs::read_dir(&project_path) {
            Ok(files) => files,
            Err(error) => {
                records.push(invalid_record(
                    "claude",
                    &file_stem(&project_path),
                    &project_path.to_string_lossy(),
                    format!("读取 Claude 会话目录失败: {error}"),
                ));
                continue;
            }
        };

        for file in files {
            let file = match file {
                Ok(file) => file,
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

            let path = file.path();
            if !path.is_file() || path.extension().and_then(|ext| ext.to_str()) != Some("jsonl") {
                continue;
            }

            match parse_claude_session_file(&path) {
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

    records.sort_by(|left, right| right.updated_at.cmp(&left.updated_at));
    Ok(records)
}

fn list_codex_sessions(db_path: PathBuf) -> Result<Vec<SessionRecord>, String> {
    if !db_path.exists() {
        return Err(format!(
            "Codex 数据库不存在：{}。请确认已安装 Codex CLI 并至少运行过一次，或在设置中指定自定义路径。",
            db_path.display()
        ));
    }

    // Codex may be running; open read-only so we never block the writer.
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
        if preview.is_empty() {
            "(untitled)".to_string()
        } else {
            preview.clone()
        }
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

/// Strip the `\\?\` extended-length prefix Windows sometimes uses.
fn normalize_windows_path(path: &str) -> String {
    path.strip_prefix(r"\\?\")
        .unwrap_or(path)
        .to_string()
}

fn ms_to_rfc3339(millis: i64) -> String {
    system_time_to_rfc3339(UNIX_EPOCH + std::time::Duration::from_millis(millis.max(0) as u64))
}

#[tauri::command]
fn get_app_state() -> Result<AppState, String> {
    read_app_state()
}

#[tauri::command]
fn set_favorite(session_id: String, favorite: bool) -> Result<AppState, String> {
    let mut state = read_app_state()?;
    let mut favorites: BTreeSet<String> = state.favorites.into_iter().collect();

    if favorite {
        favorites.insert(session_id);
    } else {
        favorites.remove(&session_id);
    }

    state.favorites = favorites.into_iter().collect();
    write_app_state(&state)?;
    Ok(state)
}

#[tauri::command]
fn set_deepseek_launcher(launcher: String) -> Result<AppState, String> {
    let launcher = normalize_deepseek_launcher(Some(launcher));
    let mut state = read_app_state()?;
    state.deepseek_launcher = launcher;
    write_app_state(&state)?;
    Ok(state)
}

#[tauri::command]
fn check_agent(source: Option<String>, deepseek_launcher: Option<String>) -> DeepseekStatus {
    let command = match source.as_deref().unwrap_or("deepseek") {
        "claude" => CLAUDE_CODE_COMMAND,
        "codex" => CODEX_COMMAND,
        _ => deepseek_command(&normalize_deepseek_launcher(deepseek_launcher)),
    };

    match command_output(command, "--version") {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            let version = if stdout.is_empty() { stderr } else { stdout };
            DeepseekStatus {
                available: true,
                version: version.clone(),
                message: if version.is_empty() {
                    format!("{command} 可用")
                } else {
                    version
                },
            }
        }
        Ok(output) => DeepseekStatus {
            available: false,
            version: String::new(),
            message: format!("{command} --version 退出码异常: {}", output.status),
        },
        Err(error) => DeepseekStatus {
            available: false,
            version: String::new(),
            message: format!("未找到 {command} 命令: {error}"),
        },
    }
}

#[tauri::command]
fn open_session_folder(path: String) -> Result<(), String> {
    let file_path = PathBuf::from(path);
    let folder = file_path.parent().unwrap_or_else(|| Path::new("."));

    #[cfg(target_os = "windows")]
    let result = Command::new("explorer").arg(folder).spawn();

    #[cfg(target_os = "macos")]
    let result = Command::new("open").arg(folder).spawn();

    #[cfg(all(unix, not(target_os = "macos")))]
    let result = Command::new("xdg-open").arg(folder).spawn();

    result
        .map(|_| ())
        .map_err(|error| format!("打开目录失败 {}: {error}", folder.display()))
}

#[tauri::command]
fn resume_session(
    source: Option<String>,
    session_id: String,
    workspace: Option<String>,
    launch_mode: Option<String>,
    deepseek_launcher: Option<String>,
    prompt: Option<String>,
) -> Result<(), String> {
    let mode = launch_mode.unwrap_or_else(|| "new_terminal".to_string());
    if mode != "new_terminal" {
        return Err("V0.1 暂只支持打开新的系统终端。".to_string());
    }

    let cwd = workspace_dir(workspace.map(|w| normalize_windows_path(&w)));
    match source.as_deref().unwrap_or("deepseek") {
        "claude" => launch_resume(CLAUDE_CODE_COMMAND, "--resume", &session_id, &cwd),
        "codex" => launch_codex_resume(&session_id, prompt.as_deref(), &cwd),
        _ => launch_resume(
            deepseek_command(&normalize_deepseek_launcher(deepseek_launcher)),
            "resume",
            &session_id,
            &cwd,
        ),
    }
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
        workspace: string_at(metadata, "workspace").unwrap_or_default(),
        mode: string_at(metadata, "mode").unwrap_or_default(),
        path: path.to_string_lossy().to_string(),
        invalid_reason: None,
    })
}

fn parse_claude_session_file(path: &Path) -> Result<SessionRecord, String> {
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

fn invalid_record(source: &str, id: &str, path: &str, reason: String) -> SessionRecord {
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

fn content_to_text(content: Option<&Value>) -> String {
    match content {
        Some(Value::String(text)) => compact(text),
        Some(Value::Array(items)) => compact(
            &items
                .iter()
                .filter_map(|item| {
                    if let Some(text) = item.as_str() {
                        return Some(text.to_string());
                    }
                    if item.get("type").and_then(Value::as_str) == Some("text") {
                        return item.get("text").and_then(Value::as_str).map(str::to_string);
                    }
                    item.get("content")
                        .and_then(Value::as_str)
                        .map(str::to_string)
                })
                .collect::<Vec<_>>()
                .join(" "),
        ),
        Some(Value::Object(map)) => map
            .get("content")
            .and_then(Value::as_str)
            .map(compact)
            .unwrap_or_default(),
        _ => String::new(),
    }
}

fn read_app_state() -> Result<AppState, String> {
    let path = app_state_path();
    if !path.exists() {
        return Ok(default_app_state());
    }

    let content = fs::read_to_string(&path)
        .map_err(|error| format!("读取状态文件失败 {}: {error}", path.display()))?;
    serde_json::from_str(&content)
        .map_err(|error| format!("状态文件格式错误 {}: {error}", path.display()))
}

fn write_app_state(state: &AppState) -> Result<(), String> {
    let path = app_state_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("创建状态目录失败 {}: {error}", parent.display()))?;
    }

    let content =
        serde_json::to_string_pretty(state).map_err(|error| format!("序列化状态失败: {error}"))?;
    fs::write(&path, content)
        .map_err(|error| format!("写入状态文件失败 {}: {error}", path.display()))
}

fn default_app_state() -> AppState {
    AppState {
        favorites: Vec::new(),
        launch_mode: "new_terminal".to_string(),
        deepseek_launcher: default_deepseek_launcher(),
    }
}

fn default_deepseek_launcher() -> String {
    "cmd".to_string()
}

fn normalize_deepseek_launcher(value: Option<String>) -> String {
    match value.as_deref() {
        Some("ps1") => "ps1".to_string(),
        _ => default_deepseek_launcher(),
    }
}

fn deepseek_command(launcher: &str) -> &'static str {
    if launcher == "ps1" {
        DEEPSEEK_PS1_COMMAND
    } else {
        DEEPSEEK_CMD_COMMAND
    }
}

fn default_sessions_dir() -> PathBuf {
    home_dir().join(".deepseek").join("sessions")
}

fn default_claude_projects_dir() -> PathBuf {
    home_dir().join(".claude").join("projects")
}

fn default_codex_db_path() -> PathBuf {
    home_dir().join(".codex").join("state_5.sqlite")
}

fn app_state_path() -> PathBuf {
    home_dir()
        .join(".deepseek-session-manager")
        .join("state.json")
}

fn home_dir() -> PathBuf {
    dirs::home_dir().unwrap_or_else(|| PathBuf::from("."))
}

fn workspace_dir(workspace: Option<String>) -> PathBuf {
    let Some(workspace) = workspace else {
        return home_dir();
    };

    let path = PathBuf::from(workspace);
    if path.is_dir() {
        path
    } else {
        home_dir()
    }
}

fn launch_codex_resume(
    session_id: &str,
    prompt: Option<&str>,
    cwd: &Path,
) -> Result<(), String> {
    let prompt = prompt
        .map(sanitize_single_line)
        .filter(|value| !value.is_empty());

    #[cfg(target_os = "windows")]
    {
        let cwd_text = cwd.to_string_lossy().to_string();
        let cwd_text = cwd_text
            .strip_prefix(r"\\?\")
            .unwrap_or(&cwd_text)
            .to_string();

        let script = match prompt.as_deref() {
            // PowerShell's call operator `&` ensures the .ps1 is invoked as
            // a script, not parsed as a literal string.
            Some(p) => format!(
                "& {} resume {} {}",
                CODEX_COMMAND,
                powershell_quote(session_id),
                powershell_quote(p)
            ),
            None => format!(
                "& {} resume {}",
                CODEX_COMMAND,
                powershell_quote(session_id)
            ),
        };

        match Command::new("wt")
            .args([
                "-d",
                &cwd_text,
                "powershell.exe",
                "-NoExit",
                "-ExecutionPolicy",
                "Bypass",
                "-Command",
                &script,
            ])
            .spawn()
        {
            Ok(_) => Ok(()),
            Err(_) => Command::new("powershell.exe")
                .args(["-NoExit", "-ExecutionPolicy", "Bypass", "-Command", &script])
                .current_dir(cwd)
                .spawn()
                .map(|_| ())
                .map_err(|error| format!("启动 {CODEX_COMMAND} 失败: {error}")),
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        let mut cmd = Command::new(CODEX_COMMAND);
        cmd.arg("resume").arg(session_id);
        if let Some(p) = prompt {
            cmd.arg(p);
        }
        cmd.current_dir(cwd)
            .spawn()
            .map(|_| ())
            .map_err(|error| format!("启动 {CODEX_COMMAND} 失败: {error}"))
    }
}

/// Collapse all whitespace into single spaces. Mirrors the frontend
/// `normalizeSingleLine` so the prompt preview matches what actually runs.
fn sanitize_single_line(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn launch_resume(
    command: &str,
    resume_arg: &str,
    session_id: &str,
    cwd: &Path,
) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let cwd_text = cwd.to_string_lossy().to_string();
        if command.ends_with(".ps1") {
            return launch_resume_powershell(command, resume_arg, session_id, cwd, &cwd_text);
        }

        match Command::new("wt")
            .args([
                "-d", &cwd_text, "cmd", "/K", command, resume_arg, session_id,
            ])
            .spawn()
        {
            Ok(_) => return Ok(()),
            Err(_) => Command::new("cmd")
                .args([
                    "/C", "start", command, "cmd", "/K", command, resume_arg, session_id,
                ])
                .current_dir(cwd)
                .spawn()
                .map(|_| ())
                .map_err(|error| format!("启动 {command} 失败: {error}")),
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        Command::new(command)
            .args([resume_arg, session_id])
            .current_dir(cwd)
            .spawn()
            .map(|_| ())
            .map_err(|error| format!("启动 {command} 失败: {error}"))
    }
}

#[cfg(target_os = "windows")]
fn command_output(command: &str, arg: &str) -> std::io::Result<std::process::Output> {
    if command.ends_with(".ps1") {
        Command::new("powershell.exe")
            .args([
                "-NoProfile",
                "-ExecutionPolicy",
                "Bypass",
                "-Command",
                &format!("& {command} {arg}"),
            ])
            .output()
    } else {
        Command::new(command).arg(arg).output()
    }
}

#[cfg(not(target_os = "windows"))]
fn command_output(command: &str, arg: &str) -> std::io::Result<std::process::Output> {
    Command::new(command).arg(arg).output()
}

#[cfg(target_os = "windows")]
fn launch_resume_powershell(
    command: &str,
    resume_arg: &str,
    session_id: &str,
    cwd: &Path,
    cwd_text: &str,
) -> Result<(), String> {
    let script = format!(
        "{} {} {}",
        command,
        powershell_quote(resume_arg),
        powershell_quote(session_id)
    );

    match Command::new("wt")
        .args([
            "-d",
            cwd_text,
            "powershell.exe",
            "-NoExit",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            &script,
        ])
        .spawn()
    {
        Ok(_) => Ok(()),
        Err(_) => Command::new("powershell.exe")
            .args(["-NoExit", "-ExecutionPolicy", "Bypass", "-Command", &script])
            .current_dir(cwd)
            .spawn()
            .map(|_| ())
            .map_err(|error| format!("启动 {command} 失败: {error}")),
    }
}

#[cfg(target_os = "windows")]
fn powershell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

fn string_at(value: &Value, key: &str) -> Option<String> {
    value.get(key).and_then(Value::as_str).map(str::to_string)
}

fn number_at(value: &Value, key: &str) -> Option<u64> {
    value.get(key).and_then(Value::as_u64)
}

fn compact(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn file_stem(path: &Path) -> String {
    path.file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("(unknown)")
        .to_string()
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

fn decode_claude_project_dir(name: &str) -> String {
    // Claude Code encodes project paths by replacing separators with '-'.
    // Absolute paths start with a leading '-' marker.
    // Windows: "-C--Users-Cap-Desktop-proj" -> "C:\Users\Cap\Desktop\proj"
    // POSIX:   "-home-cap-proj"             -> "/home/cap/proj"
    // Note: original paths containing literal '-' are ambiguous and cannot be
    // perfectly recovered; this decoder still prefers the `cwd` field from
    // the JSONL content when available.
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

fn system_time_to_rfc3339(value: SystemTime) -> String {
    let duration = value.duration_since(UNIX_EPOCH).unwrap_or_default();
    let total_secs = duration.as_secs() as i64;

    let days = total_secs.div_euclid(86_400);
    let secs_of_day = total_secs.rem_euclid(86_400);
    let hour = (secs_of_day / 3600) as u32;
    let minute = ((secs_of_day % 3600) / 60) as u32;
    let second = (secs_of_day % 60) as u32;

    let (year, month, day) = civil_from_days(days);
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        year, month, day, hour, minute, second
    )
}

/// Convert days since 1970-01-01 to (year, month, day). Algorithm from
/// Howard Hinnant's date library, valid for the full proleptic Gregorian range.
fn civil_from_days(days: i64) -> (i32, u32, u32) {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64; // [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365; // [0, 399]
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32; // [1, 31]
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32; // [1, 12]
    let y = if m <= 2 { y + 1 } else { y };
    (y as i32, m, d)
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            list_sessions,
            get_app_state,
            set_favorite,
            set_deepseek_launcher,
            check_agent,
            open_session_folder,
            resume_session
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
