//! Executes launch plans and opens local session folders.

#[cfg(target_os = "windows")]
mod windows;

#[cfg(not(target_os = "windows"))]
mod posix;

use crate::model::LaunchPlan;
use std::path::{Path, PathBuf};

pub fn execute_plan(plan: LaunchPlan) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        windows::execute_plan(plan)
    }

    #[cfg(not(target_os = "windows"))]
    {
        posix::execute_plan(plan)
    }
}

pub fn open_folder(path: PathBuf) -> Result<(), String> {
    let folder = path.parent().unwrap_or_else(|| Path::new("."));

    #[cfg(target_os = "windows")]
    let result = std::process::Command::new("explorer").arg(folder).spawn();

    #[cfg(target_os = "macos")]
    let result = std::process::Command::new("open").arg(folder).spawn();

    #[cfg(all(unix, not(target_os = "macos")))]
    let result = std::process::Command::new("xdg-open").arg(folder).spawn();

    result
        .map(|_| ())
        .map_err(|error| format!("打开目录失败 {}: {error}", folder.display()))
}
