//! Helpers shared by the rules and the rest of the application.

use std::collections::{HashMap, HashSet};
use std::ffi::CString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command as Process, Stdio};
use std::sync::{Mutex, OnceLock};
use std::time::UNIX_EPOCH;

use regex::Regex;
use serde_json::{json, Value};

use crate::conf::settings;
use crate::difflib;
use crate::logs;
use crate::shells::shell;
use crate::types::Command;

/// Returns the home directory of the current user.
pub fn home_dir() -> PathBuf {
    std::env::var_os("HOME")
        .filter(|home| !home.is_empty())
        .map_or_else(|| PathBuf::from("/"), PathBuf::from)
}

/// Replaces the leading `~` with the home directory.
pub fn expanduser(path: &str) -> PathBuf {
    if path == "~" {
        home_dir()
    } else if let Some(rest) = path.strip_prefix("~/") {
        home_dir().join(rest)
    } else {
        PathBuf::from(path)
    }
}

/// Compiles the regex once and returns it; for regexes used by rules.
///
/// Panics on an invalid pattern, which is a bug in the rule.
pub fn regex(pattern: &str) -> Regex {
    static CACHE: Mutex<Option<HashMap<String, Regex>>> = Mutex::new(None);
    let mut cache = CACHE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    cache
        .get_or_insert_with(HashMap::new)
        .entry(pattern.to_string())
        .or_insert_with(|| {
            Regex::new(pattern).unwrap_or_else(|error| panic!("invalid regex: {}", error))
        })
        .clone()
}

/// Returns the first group of all matches of the pattern, like `re.findall`
/// does for patterns with one group.
pub fn findall(pattern: &str, text: &str) -> Vec<String> {
    regex(pattern)
        .captures_iter(text)
        .map(|captures| {
            captures
                .get(1)
                .map_or("", |group| group.as_str())
                .to_string()
        })
        .collect()
}

/// Returns the first group of the first match of the pattern.
pub fn find_first(pattern: &str, text: &str) -> Option<String> {
    regex(pattern).captures(text).map(|captures| {
        captures
            .get(1)
            .map_or("", |group| group.as_str())
            .to_string()
    })
}

/// Returns `true` if the current user is allowed to execute the file.
pub fn is_executable(path: &Path) -> bool {
    let Ok(c_path) = CString::new(path.as_os_str().as_encoded_bytes()) else {
        return false;
    };
    // SAFETY: `c_path` is a valid NUL-terminated string.
    unsafe { libc::access(c_path.as_ptr(), libc::X_OK) == 0 }
}

fn is_executable_file(path: &Path) -> bool {
    path.is_file() && is_executable(path)
}

/// Returns `program` path or `None`.
pub fn which(program: &str) -> Option<String> {
    static CACHE: Mutex<Option<HashMap<String, Option<String>>>> = Mutex::new(None);
    let mut cache = CACHE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    cache
        .get_or_insert_with(HashMap::new)
        .entry(program.to_string())
        .or_insert_with(|| {
            if program.is_empty() {
                None
            } else if program.contains('/') {
                is_executable_file(Path::new(program)).then(|| program.to_string())
            } else {
                let path = std::env::var_os("PATH")?;
                std::env::split_paths(&path)
                    .map(|dir| dir.join(program))
                    .find(|candidate| is_executable_file(candidate))
                    .map(|found| found.to_string_lossy().into_owned())
            }
        })
        .clone()
}

/// Output of a finished helper process.
#[derive(Debug, Default)]
pub struct Captured {
    pub stdout: String,
    pub stderr: String,
    pub success: bool,
    pub code: Option<i32>,
}

impl Captured {
    /// Returns lines of stdout, each still ends with its line break.
    pub fn stdout_lines(&self) -> Vec<&str> {
        self.stdout.split_inclusive('\n').collect()
    }

    /// Returns lines of stderr, each still ends with its line break.
    pub fn stderr_lines(&self) -> Vec<&str> {
        self.stderr.split_inclusive('\n').collect()
    }
}

/// Runs a program and captures its output. The result is empty when the
/// program can't be started.
pub fn run(program: &str, args: &[&str]) -> Captured {
    let output = Process::new(program)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output();
    match output {
        Ok(output) => Captured {
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
            success: output.status.success(),
            code: output.status.code(),
        },
        Err(error) => {
            logs::debug(&format!("Can't run {}: {}", program, error));
            Captured::default()
        }
    }
}

/// Returns closest match or just first from possibilities.
pub fn get_closest<S: AsRef<str>>(
    word: &str,
    possibilities: &[S],
    cutoff: f64,
    fallback_to_first: bool,
) -> Option<String> {
    difflib::get_close_matches(word, possibilities, 1, cutoff)
        .into_iter()
        .next()
        .or_else(|| {
            if fallback_to_first {
                possibilities
                    .first()
                    .map(|first| first.as_ref().to_string())
            } else {
                None
            }
        })
}

/// Returns at most `settings.num_close_matches` close matches.
pub fn get_close_matches<S: AsRef<str>>(
    word: &str,
    possibilities: &[S],
    cutoff: f64,
) -> Vec<String> {
    difflib::get_close_matches(word, possibilities, settings().num_close_matches, cutoff)
}

fn include_path_in_search(path: &str) -> bool {
    !settings()
        .excluded_search_path_prefixes
        .iter()
        .any(|prefix| path.starts_with(prefix.as_str()))
}

/// Returns names of everything callable from the shell: files from `PATH`
/// and shell aliases.
pub fn get_all_executables() -> &'static [String] {
    static EXECUTABLES: OnceLock<Vec<String>> = OnceLock::new();
    EXECUTABLES.get_or_init(|| {
        let tf_alias = get_alias();
        let tf_entry_points = ["thefuck", "fuck"];

        let path = std::env::var("PATH").unwrap_or_default();
        let mut executables: Vec<String> = path
            .split(':')
            .filter(|dir| include_path_in_search(dir))
            .filter_map(|dir| fs::read_dir(if dir.is_empty() { "." } else { dir }).ok())
            .flat_map(|entries| entries.flatten())
            .filter(|entry| !entry.path().is_dir())
            .map(|entry| entry.file_name().to_string_lossy().into_owned())
            .filter(|name| !tf_entry_points.contains(&name.as_str()))
            .collect();
        executables.extend(
            shell()
                .get_aliases()
                .into_keys()
                .filter(|alias| *alias != tf_alias),
        );
        executables
    })
}

/// Replaces command line argument.
pub fn replace_argument(script: &str, from: &str, to: &str) -> String {
    let (body, line_break) = match script.strip_suffix('\n') {
        Some(body) => (body, "\n"),
        None => (script, ""),
    };
    let suffix = format!(" {}", from);
    for (body, line_break) in [(script, ""), (body, line_break)] {
        if let Some(start) = body.strip_suffix(suffix.as_str()) {
            return format!("{} {}{}", start, to, line_break);
        }
    }
    script.replacen(&format!(" {} ", from), &format!(" {} ", to), 1)
}

/// Returns all non-empty lines that go after the first line containing one of
/// the separators, without the lines containing separators themselves.
pub fn get_all_matched_commands(stderr: &str, separators: &[&str]) -> Vec<String> {
    let mut should_yield = false;
    let mut matched = Vec::new();
    for line in stderr.split('\n') {
        if separators.iter().any(|separator| line.contains(separator)) {
            should_yield = true;
        } else if should_yield && !line.is_empty() {
            matched.push(line.trim().to_string());
        }
    }
    matched
}

/// Helper for *_no_command rules.
pub fn replace_command<S: AsRef<str>>(
    command: &Command,
    broken: &str,
    matched: &[S],
) -> Vec<String> {
    get_close_matches(broken, matched, 0.1)
        .iter()
        .map(|new_cmd| replace_argument(&command.script, broken, new_cmd.trim()))
        .collect()
}

/// Returns `true` if command is call to one of passed app names and has more
/// than `at_least` parts.
pub fn is_app(command: &Command, app_names: &[&str], at_least: usize) -> bool {
    let parts = command.script_parts();
    if parts.len() > at_least {
        let basename = parts[0].rsplit('/').next().unwrap_or("");
        app_names.contains(&basename)
    } else {
        false
    }
}

/// Returns `true` if command is call to one of passed app names.
pub fn for_app(command: &Command, app_names: &[&str]) -> bool {
    is_app(command, app_names, 0)
}

fn mtime(path: &Path) -> String {
    fs::metadata(path)
        .and_then(|metadata| metadata.modified())
        .ok()
        .and_then(|modified| modified.duration_since(UNIX_EPOCH).ok())
        .map_or_else(
            || "0".to_string(),
            |since_epoch| since_epoch.as_secs_f64().to_string(),
        )
}

fn cache_path() -> PathBuf {
    let cache_dir = match std::env::var("XDG_CACHE_HOME") {
        Ok(dir) if !dir.is_empty() => PathBuf::from(dir),
        _ => expanduser("~/.cache"),
    };
    cache_dir.join("thefuck.json")
}

/// Caches the result of `compute` in a file between runs.
///
/// Cache will be expired when modification date of files from `depends_on`
/// will be changed.
pub fn cache(name: &str, depends_on: &[&str], compute: impl FnOnce() -> Value) -> Value {
    static DB: Mutex<Option<serde_json::Map<String, Value>>> = Mutex::new(None);

    if cfg!(test) {
        return compute();
    }

    let depends_on: Vec<PathBuf> = depends_on
        .iter()
        .map(|path| std::path::absolute(expanduser(path)).unwrap_or_else(|_| PathBuf::from(path)))
        .collect();
    let key = format!("{}:{:?}", name, depends_on);
    let etag = depends_on
        .iter()
        .map(|path| mtime(path))
        .collect::<Vec<_>>()
        .join(".");

    let mut db = DB.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    let db = db.get_or_insert_with(|| {
        fs::read_to_string(cache_path())
            .ok()
            .and_then(|content| serde_json::from_str(&content).ok())
            .unwrap_or_default()
    });

    if let Some(entry) = db.get(&key) {
        if entry["etag"] == json!(etag) {
            return entry["value"].clone();
        }
    }

    let value = compute();
    db.insert(key, json!({"etag": etag, "value": value}));
    let path = cache_path();
    let saved = path
        .parent()
        .map_or(Ok(()), fs::create_dir_all)
        .and_then(|_| fs::write(&path, Value::Object(db.clone()).to_string()));
    if let Err(error) = saved {
        logs::debug(&format!("Can't save cache: {}", error));
    }
    value
}

/// [`cache`] for lists of strings.
pub fn cache_strings(
    name: &str,
    depends_on: &[&str],
    compute: impl FnOnce() -> Vec<String>,
) -> Vec<String> {
    let value = cache(name, depends_on, || json!(compute()));
    serde_json::from_value(value).unwrap_or_default()
}

/// [`cache_strings`] for output of an application: expires when the
/// application is updated. Nothing is cached for applications that aren't
/// installed.
pub fn cache_for_app(app: &str, name: &str, compute: impl FnOnce() -> Vec<String>) -> Vec<String> {
    match which(app) {
        Some(path) => cache_strings(name, &[&path], compute),
        None => compute(),
    }
}

/// Returns the absolute path without `.` and `..` components. The file
/// system isn't accessed, so symlinks aren't resolved.
pub fn abspath(path: &str) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in std::path::absolute(path)
        .unwrap_or_else(|_| PathBuf::from(path))
        .components()
    {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                normalized.pop();
            }
            other => normalized.push(other),
        }
    }
    normalized
}

/// Removes files extracted from an archive to the current directory.
pub fn remove_extracted_files<'a>(files: impl Iterator<Item = &'a str>) {
    let Ok(cwd) = std::env::current_dir() else {
        return;
    };
    for file in files {
        if !abspath(file)
            .to_string_lossy()
            .starts_with(cwd.to_string_lossy().as_ref())
        {
            // it's unsafe to overwrite files outside of the current directory
            continue;
        }

        // does not try to remove directories as we cannot know if they
        // already existed before
        let _ = fs::remove_file(file);
    }
}

/// Returns name of the alias used for calling The Fuck.
pub fn get_alias() -> String {
    std::env::var("TF_ALIAS").unwrap_or_else(|_| "fuck".to_string())
}

/// Returns all lines from history except that comes before `fuck`.
fn not_corrected(history: &[String], tf_alias: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut previous: Option<&String> = None;
    for line in history {
        if let Some(previous) = previous {
            if line != tf_alias {
                result.push(previous.clone());
            }
        }
        previous = Some(line);
    }
    result.extend(history.last().cloned());
    result
}

/// Returns history without the current command, calls of The Fuck, commands
/// fixed by it and without lines that don't look like calls of something
/// executable.
pub fn get_valid_history_without_current(command: &Command) -> Vec<String> {
    static HISTORY: OnceLock<Vec<String>> = OnceLock::new();
    let tf_alias = get_alias();
    HISTORY
        .get_or_init(|| {
            let executables: HashSet<&str> = get_all_executables()
                .iter()
                .map(String::as_str)
                .chain(shell().get_builtin_commands().iter().copied())
                .collect();
            not_corrected(&shell().get_history(), &tf_alias)
                .into_iter()
                .filter(|line| {
                    !line.starts_with(&tf_alias)
                        && executables.contains(line.split(' ').next().unwrap_or(""))
                })
                .collect()
        })
        .iter()
        .filter(|line| **line != command.script)
        .cloned()
        .collect()
}

/// Creates single script from a list of script parts.
pub fn format_raw_script(raw_script: &[String]) -> String {
    raw_script.join(" ").trim_start().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replaces_arguments() {
        assert_eq!(
            replace_argument("git brnch", "brnch", "branch"),
            "git branch"
        );
        assert_eq!(
            replace_argument("git brnch -a", "brnch", "branch"),
            "git branch -a"
        );
        assert_eq!(
            replace_argument("git add add", "add", "add --force"),
            "git add add --force"
        );
        assert_eq!(replace_argument("git a.b", "a.b", "c"), "git c");
        assert_eq!(replace_argument("gitbrnch", "brnch", "branch"), "gitbrnch");
    }

    #[test]
    fn finds_matched_commands() {
        let stderr = "git: 'cs' is not a git command. See 'git --help'.\n\n\
                      The most similar command is\n\tcommit\n\tcss\n";
        assert_eq!(
            get_all_matched_commands(stderr, &["The most similar command"]),
            ["commit", "css"]
        );
        assert!(get_all_matched_commands(stderr, &["Did you mean"]).is_empty());
    }

    #[test]
    fn finds_closest() {
        assert_eq!(
            get_closest("ch", &["checkout", "branch"], 0.6, true).unwrap(),
            "checkout"
        );
        assert_eq!(
            get_closest("st", &["status", "reset"], 0.6, true).unwrap(),
            "status"
        );
        assert_eq!(get_closest("st", &["status", "reset"], 0.6, false), None);
        assert_eq!(get_closest::<&str>("st", &[], 0.6, true), None);
    }

    #[test]
    fn recognizes_apps() {
        let command = |script: &str| Command::new(script, Some(""));
        assert!(is_app(&command("git co"), &["git", "hub"], 0));
        assert!(is_app(&command("/usr/bin/hub diff"), &["git", "hub"], 0));
        assert!(!is_app(&command("gitk"), &["git"], 0));
        assert!(!is_app(&command("git"), &["git"], 1));
        assert!(is_app(&command("git push"), &["git"], 1));
    }

    #[test]
    fn skips_commands_before_the_alias() {
        let history: Vec<String> = ["ls cat", "fuck", "git log", "ls"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        assert_eq!(not_corrected(&history, "fuck"), ["fuck", "git log", "ls"]);
    }

    #[test]
    fn normalizes_paths() {
        assert_eq!(abspath("/a/b/../c/./d"), PathBuf::from("/a/c/d"));
        assert_eq!(abspath("/../a"), PathBuf::from("/a"));
        assert_eq!(
            abspath("a/../b"),
            std::env::current_dir().unwrap().join("b")
        );
    }

    #[test]
    fn regex_helpers_work() {
        assert_eq!(findall(r"'(\w+)'", "'a' and 'b'"), ["a", "b"]);
        assert_eq!(find_first(r"'(\w+)'", "'a' and 'b'").unwrap(), "a");
        assert_eq!(find_first(r"'(\w+)'", "nothing"), None);
    }
}
