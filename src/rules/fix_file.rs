use std::path::Path;

use super::prelude::*;
use crate::output_readers::rerun::SHELL;

// order is important: only the first match is considered
const PATTERNS: &[&str] = &[
    // js, node:
    "^    at {file}:{line}:{col}",
    // cargo:
    "^   {file}:{line}:{col}",
    // python, thefuck:
    "^  File \"{file}\", line {line}",
    // awk:
    "^awk: {file}:{line}:",
    // git
    "^fatal: bad config file line {line} in {file}",
    // llc:
    "^llc: {file}:{line}:{col}:",
    // lua:
    "^lua: {file}:{line}:",
    // fish:
    "^{file} \\(line {line}\\):",
    // bash, sh, ssh:
    "^{file}: line {line}: ",
    // cargo, clang, gcc, go, pep8, rustc:
    "^{file}:{line}:{col}",
    // ghc, make, ruby, zsh:
    "^{file}:{line}:",
    // perl:
    "at {file} line {line}",
];

// for the sake of readability do not use named groups above
fn make_pattern(pattern: &str) -> String {
    let pattern = pattern
        .replace("{file}", "(?P<file>[^:\n]+)")
        .replace("{line}", "(?P<line>[0-9]+)")
        .replace("{col}", "(?P<col>[0-9]+)");
    format!("(?m){}", pattern)
}

/// Returns the file and the line mentioned in the output.
fn search(output: &str) -> Option<(String, String)> {
    for pattern in PATTERNS {
        if let Some(m) = regex(&make_pattern(pattern)).captures(output) {
            // Errors of the shell used for getting the output start with its
            // path, but nobody wants to edit the shell.
            if &m["file"] != SHELL && Path::new(&m["file"]).is_file() {
                return Some((m["file"].to_string(), m["line"].to_string()));
            }
        }
    }
    None
}

fn match_(command: &Command) -> bool {
    if std::env::var_os("EDITOR").is_none() {
        return false;
    }

    search(command.output()).is_some()
}

fn get_new_command(command: &Command) -> Vec<String> {
    let Some((file, line)) = search(command.output()) else { return vec![] };
    let editor = std::env::var("EDITOR").unwrap_or_default();

    // Note: there does not seem to be a standard for columns, so they are just
    // ignored
    let editor_call = format!("{} {} +{}", editor, file, line);
    vec![and_(&[&editor_call, &command.script])]
}

pub const RULE: Rule = Rule::new("fix_file", match_, get_new_command);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_existing_file_in_output() {
        let file = file!();
        let outputs = [
            format!("    at {}:12:34\n", file),
            format!("Traceback:\n  File \"{}\", line 12, in <module>\n", file),
            format!("fatal: bad config file line 12 in {}\n", file),
            format!("{} (line 12): wrong\n", file),
            format!("{}: line 12: foo: command not found\n", file),
            format!("error\n{}:12:34: error: expected ';'\n", file),
            format!("{}:12: parse error\n", file),
            format!("syntax error at {} line 12, near \"foo\"\n", file),
        ];
        for output in outputs {
            assert_eq!(search(&output), Some((file.to_string(), "12".to_string())), "{}", output);
        }
        assert_eq!(search("    at /nonexistent/file.js:12:34\n"), None);
        assert_eq!(search("nothing to see here"), None);
        assert_eq!(search("/bin/sh: line 1: gti: command not found\n"), None);
    }
}
