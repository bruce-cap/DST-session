//! Provides shell quoting, normalization, and command output helpers.

use std::io;
use std::process::{Command, Output, Stdio};
use std::thread;
use std::time::{Duration, Instant};

const COMMAND_CHECK_TIMEOUT: Duration = Duration::from_secs(3);

pub fn sanitize_single_line(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub fn powershell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

#[cfg(target_os = "windows")]
pub fn command_output(command: &str, arg: &str) -> std::io::Result<Output> {
    if command.ends_with(".ps1") {
        let mut powershell = Command::new("powershell.exe");
        powershell.args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            &format!("& {command} {arg}"),
        ]);
        hide_console_window(&mut powershell);
        output_with_timeout(powershell, COMMAND_CHECK_TIMEOUT)
    } else {
        let mut check = Command::new(command);
        check.arg(arg);
        hide_console_window(&mut check);
        output_with_timeout(check, COMMAND_CHECK_TIMEOUT)
    }
}

#[cfg(not(target_os = "windows"))]
pub fn command_output(command: &str, arg: &str) -> std::io::Result<Output> {
    let mut check = Command::new(command);
    check.arg(arg);
    output_with_timeout(check, COMMAND_CHECK_TIMEOUT)
}

#[cfg(target_os = "windows")]
fn hide_console_window(command: &mut Command) {
    use std::os::windows::process::CommandExt;

    // Prevent transient cmd/powershell windows when the GUI app performs
    // background availability checks such as `codex.ps1 --version`.
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    command.creation_flags(CREATE_NO_WINDOW);
}

fn output_with_timeout(mut command: Command, timeout: Duration) -> io::Result<Output> {
    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let started_at = Instant::now();

    loop {
        if child.try_wait()?.is_some() {
            return child.wait_with_output();
        }

        if started_at.elapsed() >= timeout {
            let _ = child.kill();
            let _ = child.wait_with_output();
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                format!("command check timed out after {}s", timeout.as_secs()),
            ));
        }

        thread::sleep(Duration::from_millis(50));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_single_line_is_idempotent() {
        let once = sanitize_single_line("a\n  b\t c");
        let twice = sanitize_single_line(&once);
        assert_eq!(once, twice);
    }

    #[test]
    fn powershell_quote_wraps_and_doubles_inner_quotes() {
        let quoted = powershell_quote("a'b");
        assert_eq!(quoted, "'a''b'");
        let inner = &quoted[1..quoted.len() - 1];
        let mut chars = inner.chars().peekable();
        while let Some(ch) = chars.next() {
            if ch == '\'' {
                assert_eq!(chars.next(), Some('\''));
            }
        }
    }
}
