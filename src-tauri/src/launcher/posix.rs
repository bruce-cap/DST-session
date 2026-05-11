//! Executes launch plans on POSIX platforms without terminal wrapping.

use crate::model::LaunchPlan;
use crate::shell::sanitize_single_line;
use std::path::PathBuf;
use std::process::Command;

pub fn execute_plan(plan: LaunchPlan) -> Result<(), String> {
    let cwd = plan
        .cwd
        .as_deref()
        .map(PathBuf::from)
        .unwrap_or_else(crate::paths::home_dir);
    let args = plan.args.into_iter().filter_map(|arg| {
        let value = if arg.single_line {
            sanitize_single_line(&arg.value)
        } else {
            arg.value
        };
        if value.is_empty() {
            None
        } else {
            Some(value)
        }
    });

    Command::new(&plan.program)
        .args(args)
        .current_dir(cwd)
        .spawn()
        .map(|_| ())
        .map_err(|error| format!("启动 {} 失败: {error}", plan.error_command_label))
}
