//! Defines provider registration and dispatch for session sources.

pub mod claude;
pub mod codex;
pub mod deepseek;

use crate::model::{DeepseekStatus, LaunchPlan, ProviderDescriptor, SessionRecord};
use std::collections::BTreeMap;
use std::path::PathBuf;

pub const DEEPSEEK_CMD_COMMAND: &str = "deepseek.cmd";
pub const DEEPSEEK_PS1_COMMAND: &str = "deepseek.ps1";
pub const CLAUDE_CMD_COMMAND: &str = "claude.cmd";
pub const CLAUDE_PS1_COMMAND: &str = "claude.ps1";
pub const CLAUDE_PREVIEW_COMMAND: &str = "claude";
pub const CODEX_CMD_COMMAND: &str = "codex.cmd";
pub const CODEX_PS1_COMMAND: &str = "codex.ps1";
pub const DEFAULT_SOURCE: &str = "deepseek";

pub trait Provider: Send + Sync {
    fn descriptor(&self) -> ProviderDescriptor;
    fn list_sessions(&self, override_path: Option<PathBuf>) -> Result<Vec<SessionRecord>, String>;
    fn check_agent(&self, context: AgentCheckContext) -> DeepseekStatus;
    fn plan_resume(&self, request: ResumeRequest) -> Result<LaunchPlan, String>;
}

pub struct ProviderRegistry {
    providers: BTreeMap<String, Box<dyn Provider>>,
}

#[derive(Debug, Clone)]
pub struct AgentCheckContext {
    pub launcher: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ResumeRequest {
    pub session_id: String,
    pub workspace: Option<String>,
    pub launcher: Option<String>,
    pub prompt: Option<String>,
}

impl ProviderRegistry {
    pub fn bootstrap() -> Self {
        let mut providers: BTreeMap<String, Box<dyn Provider>> = BTreeMap::new();
        providers.insert(
            DEFAULT_SOURCE.to_string(),
            Box::new(deepseek::DeepseekProvider),
        );
        providers.insert("claude".to_string(), Box::new(claude::ClaudeProvider));
        providers.insert("codex".to_string(), Box::new(codex::CodexProvider));
        Self { providers }
    }

    pub fn descriptors(&self) -> Vec<ProviderDescriptor> {
        self.providers
            .values()
            .map(|provider| provider.descriptor())
            .collect()
    }

    pub fn resolve_or_default(&self, source: Option<&str>) -> &dyn Provider {
        source
            .and_then(|value| self.providers.get(value))
            .or_else(|| self.providers.get(DEFAULT_SOURCE))
            .map(Box::as_ref)
            .expect("default provider must be registered")
    }
}

pub fn deepseek_command(launcher: &str) -> &'static str {
    if launcher == "ps1" {
        DEEPSEEK_PS1_COMMAND
    } else {
        DEEPSEEK_CMD_COMMAND
    }
}

pub fn claude_command(launcher: &str) -> &'static str {
    if launcher == "ps1" {
        CLAUDE_PS1_COMMAND
    } else {
        CLAUDE_CMD_COMMAND
    }
}

pub fn codex_command(launcher: &str) -> &'static str {
    if launcher == "cmd" {
        CODEX_CMD_COMMAND
    } else {
        CODEX_PS1_COMMAND
    }
}

pub fn launch_cwd(workspace: Option<String>) -> Option<String> {
    Some(
        crate::paths::workspace_dir(workspace)
            .to_string_lossy()
            .to_string(),
    )
}

pub fn status_for_command(command: &str) -> DeepseekStatus {
    match crate::shell::command_output(command, "--version") {
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
