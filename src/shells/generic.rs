use std::collections::HashMap;
use std::fs;
use std::sync::OnceLock;

use crate::conf::settings;
use crate::logs::warn;
use crate::shlex;
use crate::utils::expanduser;

/// How to make the shell load the alias.
#[derive(Debug, Clone, PartialEq)]
pub struct ShellConfiguration {
    pub content: String,
    pub path: String,
    pub reload: String,
    pub can_configure_automatically: bool,
}

impl ShellConfiguration {
    pub(super) fn new(content: &str, path: &str, reload: &str) -> Self {
        ShellConfiguration {
            content: content.to_string(),
            path: path.to_string(),
            reload: reload.to_string(),
            can_configure_automatically: expanduser(path).exists(),
        }
    }
}

pub trait Shell: Send + Sync {
    fn friendly_name(&self) -> &'static str;

    fn get_aliases(&self) -> HashMap<String, String> {
        HashMap::new()
    }

    fn expand_aliases(&self, command_script: &str) -> String {
        let aliases = self.get_aliases();
        let binary = command_script.split(' ').next().unwrap_or("");
        match aliases.get(binary) {
            Some(expansion) => command_script.replacen(binary, expansion, 1),
            None => command_script.to_string(),
        }
    }

    /// Prepares command before running in app.
    #[allow(clippy::wrong_self_convention)]
    fn from_shell(&self, command_script: &str) -> String {
        self.expand_aliases(command_script)
    }

    /// Prepares command for running in shell.
    fn to_shell(&self, command_script: &str) -> String {
        command_script.to_string()
    }

    fn app_alias(&self, alias_name: &str) -> String {
        format!(
            "alias {0}='eval \"$(TF_ALIAS={0} thefuck \"$(fc -ln -1)\")\"'",
            alias_name
        )
    }

    fn instant_mode_alias(&self, alias_name: &str) -> String {
        warn("Instant mode not supported by your shell");
        self.app_alias(alias_name)
    }

    fn history_file_name(&self) -> String {
        String::new()
    }

    fn history_line(&self, _command_script: &str) -> String {
        String::new()
    }

    fn script_from_history(&self, line: &str) -> String {
        line.to_string()
    }

    /// Returns list of history entries.
    fn get_history(&self) -> Vec<String> {
        static HISTORY: OnceLock<Vec<String>> = OnceLock::new();
        HISTORY
            .get_or_init(|| {
                let Ok(content) = fs::read(self.history_file_name()) else {
                    return vec![];
                };
                let content = String::from_utf8_lossy(&content);
                let lines: Vec<&str> = content.split_inclusive('\n').collect();
                let skip = match settings().history_limit {
                    Some(limit) if limit > 0 => lines.len().saturating_sub(limit),
                    _ => 0,
                };
                lines[skip..]
                    .iter()
                    .map(|line| self.script_from_history(line).trim().to_string())
                    .filter(|prepared| !prepared.is_empty())
                    .collect()
            })
            .clone()
    }

    fn and_(&self, commands: &[&str]) -> String {
        commands.join(" && ")
    }

    fn or_(&self, commands: &[&str]) -> String {
        commands.join(" || ")
    }

    fn how_to_configure(&self) -> Option<ShellConfiguration> {
        None
    }

    /// Split the command using shell-like syntax.
    fn split_command(&self, command: &str) -> Vec<String> {
        // Keeps escaped spaces escaped in the parts.
        match shlex::split(&command.replace("\\ ", "??")) {
            Ok(parts) => parts.iter().map(|part| part.replace("??", "\\ ")).collect(),
            Err(_) => command.split(' ').map(String::from).collect(),
        }
    }

    /// Return a shell-escaped version of the string s.
    fn quote(&self, s: &str) -> String {
        shlex::quote(s)
    }

    /// Adds fixed command to shell history.
    ///
    /// In most of shells we change history on shell-level, but not
    /// all shells support it (Fish).
    fn put_to_history(&self, _command: &str) {}

    /// Returns shells builtin commands.
    fn get_builtin_commands(&self) -> &'static [&'static str] {
        &[
            "alias", "bg", "bind", "break", "builtin", "case", "cd", "command", "compgen",
            "complete", "continue", "declare", "dirs", "disown", "echo", "enable", "eval", "exec",
            "exit", "export", "fc", "fg", "getopts", "hash", "help", "history", "if", "jobs",
            "kill", "let", "local", "logout", "popd", "printf", "pushd", "pwd", "read", "readonly",
            "return", "set", "shift", "shopt", "source", "suspend", "test", "times", "trap",
            "type", "typeset", "ulimit", "umask", "unalias", "unset", "until", "wait", "while",
        ]
    }

    /// Returns the version of the current shell
    fn version(&self) -> String {
        String::new()
    }

    /// Returns the name and version of the current shell
    fn info(&self) -> String {
        format!("{} {}", self.friendly_name(), self.version())
            .trim_end()
            .to_string()
    }
}

pub struct Generic;

impl Shell for Generic {
    fn friendly_name(&self) -> &'static str {
        "Generic Shell"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn joins_commands() {
        assert_eq!(Generic.and_(&["ls", "cd"]), "ls && cd");
        assert_eq!(Generic.or_(&["ls", "cd"]), "ls || cd");
    }

    #[test]
    fn splits_commands() {
        assert_eq!(Generic.split_command("ls -la 'a b'"), ["ls", "-la", "a b"]);
        assert_eq!(Generic.split_command(r"cd my\ dir"), ["cd", r"my\ dir"]);
        assert_eq!(
            Generic.split_command("echo 'unbalanced one"),
            ["echo", "'unbalanced", "one"]
        );
    }

    #[test]
    fn generates_alias() {
        let alias = Generic.app_alias("fuck");
        assert!(alias.contains("alias fuck"));
        assert!(alias.contains("TF_ALIAS=fuck"));
        assert!(alias.contains("thefuck"));
    }

    #[test]
    fn has_no_configuration() {
        assert_eq!(Generic.how_to_configure(), None);
        assert_eq!(Generic.info(), "Generic Shell");
        assert_eq!(Generic.from_shell("ls -la"), "ls -la");
    }
}
