//! Obtains the output from the log written by the shell logger in the
//! experimental instant mode.

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

use super::render;
use crate::consts::{LOG_SIZE_IN_BYTES, USER_COMMAND_MARK};
use crate::logs;
use crate::shlex;

/// Groups lines of the log by the prompts: returns the lines with the typed
/// scripts together with everything shown until the next prompt.
fn group_by_calls<'a>(log: &[&'a str], ps1: &str) -> Vec<(&'a str, Vec<&'a str>)> {
    let ps1_newlines = ps1.matches("\\n").count() + ps1.matches('\n').count();
    let mut ps1_counter = 0;

    let mut groups = Vec::new();
    let mut script_line: Option<&str> = None;
    let mut lines: Vec<&str> = Vec::new();
    for line in log {
        if line.contains(USER_COMMAND_MARK) || ps1_counter > 0 {
            if let Some(script_line) = script_line {
                if ps1_counter == 0 {
                    groups.push((script_line, std::mem::take(&mut lines)));
                }
            }

            if ps1_newlines > 0 {
                if ps1_counter == 0 {
                    ps1_counter = ps1_newlines;
                } else {
                    ps1_counter -= 1;
                }
            }

            script_line = Some(line);
            lines = vec![line];
        } else if script_line.is_some() {
            lines.push(line);
        }
    }

    if let Some(script_line) = script_line {
        groups.push((script_line, lines));
    }
    groups
}

fn get_script_group_lines<'a>(
    grouped: Vec<(&'a str, Vec<&'a str>)>,
    script: &str,
) -> Option<Vec<&'a str>> {
    let parts =
        shlex::split(script).unwrap_or_else(|_| script.split(' ').map(String::from).collect());
    grouped
        .into_iter()
        .rev()
        .find(|(script_line, _)| parts.iter().all(|part| script_line.contains(part.as_str())))
        .map(|(_, lines)| lines)
}

fn get_output_from_log(script: &str, log: &str, ps1: &str) -> Option<String> {
    let data = log.trim_end_matches('\0');
    let lines: Vec<&str> = data.split('\n').collect();
    let grouped = group_by_calls(&lines, ps1);
    let script_lines = get_script_group_lines(grouped, script)?;
    Some(render(&script_lines))
}

fn read_log(path: &str) -> std::io::Result<String> {
    let mut log_file = File::open(path)?;
    // Skip old lines:
    let size = log_file.metadata()?.len();
    if size > LOG_SIZE_IN_BYTES as u64 {
        log_file.seek(SeekFrom::Start(size - LOG_SIZE_IN_BYTES as u64))?;
    }
    let mut data = Vec::new();
    log_file.read_to_end(&mut data)?;
    Ok(String::from_utf8_lossy(&data).into_owned())
}

/// Reads script output from log.
pub fn get_output(script: &str) -> Option<String> {
    let Ok(log_path) = std::env::var("THEFUCK_OUTPUT_LOG") else {
        logs::warn("Output log isn't specified");
        return None;
    };

    let ps1 = std::env::var("PS1").unwrap_or_default();
    if !ps1.contains(USER_COMMAND_MARK) {
        logs::warn(
            "PS1 doesn't contain user command mark, please ensure \
             that PS1 is not changed after The Fuck alias initialization",
        );
        return None;
    }

    logs::debug_time("Read output from log", || {
        let Ok(log) = read_log(&log_path) else {
            logs::warn("Can't read output log");
            return None;
        };
        let Some(output) = get_output_from_log(script, &log, &ps1) else {
            logs::warn("Script not found in output log");
            return None;
        };
        logs::debug(&format!("Received output: {}", output));
        Some(output)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn log() -> String {
        let mark = USER_COMMAND_MARK;
        format!(
            "{mark}$ ls\r\nfoo\r\n{mark}$ git brnch\r\ngit: 'brnch' is not a git command.\r\n\r\n\
             The most similar command is\r\n\tbranch\r\n{mark}$ fuck\r\n\0\0\0\0"
        )
    }

    #[test]
    fn finds_output_of_the_script() {
        let output = get_output_from_log("git brnch", &log(), "$ ").unwrap();
        assert!(output.ends_with(
            "$ git brnch\ngit: 'brnch' is not a git command.\n\nThe most similar command is\n        branch"
        ));
        assert!(get_output_from_log("ls", &log(), "$ ")
            .unwrap()
            .ends_with("$ ls\nfoo"));
        assert_eq!(get_output_from_log("cd", &log(), "$ "), None);
    }

    #[test]
    fn takes_the_last_call_of_the_script() {
        let mark = USER_COMMAND_MARK;
        let log = format!("{mark}$ ls\r\nold\r\n{mark}$ ls\r\nnew\r\n{mark}$ fuck");
        assert!(get_output_from_log("ls", &log, "$ ")
            .unwrap()
            .ends_with("$ ls\nnew"));
    }

    #[test]
    fn supports_multiline_prompt() {
        let mark = USER_COMMAND_MARK;
        let log = format!("{mark}~/dir\r\n$ ls\r\nfoo\r\n{mark}~/dir\r\n$ fuck\r\n");
        let output = get_output_from_log("ls", &log, "\\w\\n$ ").unwrap();
        assert_eq!(output, "$ ls\nfoo");
    }
}
