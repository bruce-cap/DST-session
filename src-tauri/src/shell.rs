//! Provides shell quoting, normalization, and command output helpers.

use std::process::{Command, Output};

pub fn sanitize_single_line(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub fn powershell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

#[cfg(target_os = "windows")]
pub fn command_output(command: &str, arg: &str) -> std::io::Result<Output> {
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
pub fn command_output(command: &str, arg: &str) -> std::io::Result<Output> {
    Command::new(command).arg(arg).output()
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
