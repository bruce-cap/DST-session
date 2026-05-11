//! Executes launch plans using Windows Terminal, PowerShell, and cmd.

use crate::model::{LaunchArg, LaunchPlan, ShellWrap};
use crate::paths::normalize_windows_path;
use crate::shell::{powershell_quote, sanitize_single_line};
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn execute_plan(plan: LaunchPlan) -> Result<(), String> {
    let cwd = plan.cwd.as_deref().map(PathBuf::from).unwrap_or_else(crate::paths::home_dir);
    let cwd_text = normalize_windows_path(&cwd.to_string_lossy());
    let args = normalize_args(&plan.args);

    if plan.prefer_windows_terminal {
        if spawn_wt(&plan, &args, &cwd_text).is_ok() {
            return Ok(());
        }
    }

    match plan.shell_wrap {
        ShellWrap::PowerShellScript => spawn_powershell(&plan, &args, &cwd),
        ShellWrap::CmdStart => spawn_cmd_start(&plan, &args, &cwd),
    }
}

fn spawn_wt(plan: &LaunchPlan, args: &[LaunchArg], cwd_text: &str) -> std::io::Result<std::process::Child> {
    match plan.shell_wrap {
        ShellWrap::PowerShellScript => Command::new("wt")
            .args([
                "-d",
                cwd_text,
                "powershell.exe",
                "-NoExit",
                "-ExecutionPolicy",
                "Bypass",
                "-Command",
                &powershell_script(plan, args),
            ])
            .spawn(),
        ShellWrap::CmdStart => {
            let mut command = Command::new("wt");
            command.args(["-d", cwd_text, "cmd", "/K", &plan.program]);
            command.args(args.iter().map(|arg| arg.value.as_str()));
            command.spawn()
        }
    }
}

fn spawn_powershell(plan: &LaunchPlan, args: &[LaunchArg], cwd: &Path) -> Result<(), String> {
    Command::new("powershell.exe")
        .args([
            "-NoExit",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            &powershell_script(plan, args),
        ])
        .current_dir(cwd)
        .spawn()
        .map(|_| ())
        .map_err(|error| format!("启动 {} 失败: {error}", plan.error_command_label))
}

fn spawn_cmd_start(plan: &LaunchPlan, args: &[LaunchArg], cwd: &Path) -> Result<(), String> {
    let mut command = Command::new("cmd");
    command.args(["/C", "start", &plan.program, "cmd", "/K", &plan.program]);
    command.args(args.iter().map(|arg| arg.value.as_str()));
    command
        .current_dir(cwd)
        .spawn()
        .map(|_| ())
        .map_err(|error| format!("启动 {} 失败: {error}", plan.error_command_label))
}

fn powershell_script(plan: &LaunchPlan, args: &[LaunchArg]) -> String {
    let mut parts = Vec::new();
    if plan.use_call_operator {
        parts.push("&".to_string());
    }
    parts.push(plan.program.clone());
    parts.extend(args.iter().map(|arg| {
        if arg.shell_quote {
            powershell_quote(&arg.value)
        } else {
            arg.value.clone()
        }
    }));
    parts.join(" ")
}

fn normalize_args(args: &[LaunchArg]) -> Vec<LaunchArg> {
    args.iter()
        .filter_map(|arg| {
            let value = if arg.single_line {
                sanitize_single_line(&arg.value)
            } else {
                arg.value.clone()
            };
            if arg.single_line && value.is_empty() {
                None
            } else {
                Some(LaunchArg { value, single_line: false, shell_quote: arg.shell_quote })
            }
        })
        .collect()
}
