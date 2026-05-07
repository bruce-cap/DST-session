use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const DEEPSEEK_TUI_COMMAND: &str = "deepseek-tui.cmd";

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SessionRecord {
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
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct DeepseekStatus {
    available: bool,
    version: String,
    message: String,
}

#[tauri::command]
fn list_sessions(sessions_dir: Option<String>) -> Result<Vec<SessionRecord>, String> {
    let dir = sessions_dir
        .map(PathBuf::from)
        .unwrap_or_else(default_sessions_dir);

    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut records = Vec::new();
    let entries = fs::read_dir(&dir)
        .map_err(|error| format!("无法读取 sessions 目录 {}: {error}", dir.display()))?;

    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(error) => {
                records.push(invalid_record("", "", format!("读取目录项失败: {error}")));
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
                &file_stem(&path),
                &path.to_string_lossy(),
                error,
            )),
        }
    }

    records.sort_by(|left, right| right.updated_at.cmp(&left.updated_at));
    Ok(records)
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
fn check_deepseek() -> DeepseekStatus {
    match Command::new(DEEPSEEK_TUI_COMMAND).arg("--version").output() {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            let version = if stdout.is_empty() { stderr } else { stdout };
            DeepseekStatus {
                available: true,
                version: version.clone(),
                message: if version.is_empty() {
                    format!("{DEEPSEEK_TUI_COMMAND} 可用")
                } else {
                    version
                },
            }
        }
        Ok(output) => DeepseekStatus {
            available: false,
            version: String::new(),
            message: format!("{DEEPSEEK_TUI_COMMAND} --version 退出码异常: {}", output.status),
        },
        Err(error) => DeepseekStatus {
            available: false,
            version: String::new(),
            message: format!("未找到 {DEEPSEEK_TUI_COMMAND} 命令: {error}"),
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
    session_id: String,
    workspace: Option<String>,
    launch_mode: Option<String>,
) -> Result<(), String> {
    let mode = launch_mode.unwrap_or_else(|| "new_terminal".to_string());
    if mode != "new_terminal" {
        return Err("V0.1 暂只支持打开新的系统终端。".to_string());
    }

    let cwd = workspace_dir(workspace);
    launch_deepseek_resume(&session_id, &cwd)
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
        short_id: id.chars().take(8).collect(),
        id,
        title,
        preview,
        created_at: string_at(metadata, "created_at"),
        updated_at: string_at(metadata, "updated_at"),
        message_count: number_at(metadata, "message_count")
            .unwrap_or_else(|| json.get("messages").and_then(Value::as_array).map_or(0, |m| m.len() as u64)),
        total_tokens: number_at(metadata, "total_tokens").unwrap_or(0),
        model: string_at(metadata, "model").unwrap_or_default(),
        workspace: string_at(metadata, "workspace").unwrap_or_default(),
        mode: string_at(metadata, "mode").unwrap_or_default(),
        path: path.to_string_lossy().to_string(),
        invalid_reason: None,
    })
}

fn invalid_record(id: &str, path: &str, reason: String) -> SessionRecord {
    let id = if id.is_empty() {
        "(invalid)".to_string()
    } else {
        id.to_string()
    };

    SessionRecord {
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
                    item.get("content").and_then(Value::as_str).map(str::to_string)
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

    let content = serde_json::to_string_pretty(state)
        .map_err(|error| format!("序列化状态失败: {error}"))?;
    fs::write(&path, content)
        .map_err(|error| format!("写入状态文件失败 {}: {error}", path.display()))
}

fn default_app_state() -> AppState {
    AppState {
        favorites: Vec::new(),
        launch_mode: "new_terminal".to_string(),
    }
}

fn default_sessions_dir() -> PathBuf {
    home_dir().join(".deepseek").join("sessions")
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

fn launch_deepseek_resume(session_id: &str, cwd: &Path) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let cwd_text = cwd.to_string_lossy().to_string();
        let resume_command = format!("{DEEPSEEK_TUI_COMMAND} resume '{}'", powershell_escape(session_id));

        match Command::new("wt")
            .args(["-d", &cwd_text, "powershell", "-NoExit", "-Command", &resume_command])
            .spawn()
        {
            Ok(_) => return Ok(()),
            Err(_) => {
                let fallback = format!(
                    "Set-Location -LiteralPath '{}'; {DEEPSEEK_TUI_COMMAND} resume '{}'",
                    powershell_escape(&cwd_text),
                    powershell_escape(session_id)
                );
                Command::new("cmd")
                    .args(["/C", "start", "DeepSeek", "powershell", "-NoExit", "-Command", &fallback])
                    .spawn()
                    .map(|_| ())
                    .map_err(|error| format!("启动 PowerShell 失败: {error}"))
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        Command::new(DEEPSEEK_TUI_COMMAND)
            .args(["resume", session_id])
            .current_dir(cwd)
            .spawn()
            .map(|_| ())
            .map_err(|error| format!("启动 {DEEPSEEK_TUI_COMMAND} 失败: {error}"))
    }
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

fn powershell_escape(value: &str) -> String {
    value.replace('\'', "''")
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            list_sessions,
            get_app_state,
            set_favorite,
            check_deepseek,
            open_session_folder,
            resume_session
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
