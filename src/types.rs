//! The core types: the command to fix, the rules and the fixed commands.

use std::cell::RefCell;
use std::panic::{self, AssertUnwindSafe};
use std::sync::{Once, OnceLock};

use crate::conf::settings;
use crate::consts::{ALL_ENABLED, DEFAULT_PRIORITY};
use crate::logs;
use crate::output_readers::get_output;
use crate::shells::shell;
use crate::utils::{format_raw_script, get_alias};

/// Command that should be fixed.
#[derive(Debug, Clone)]
pub struct Command {
    pub script: String,
    /// Combined stdout and stderr; `None` when it couldn't be obtained.
    pub output: Option<String>,
    script_parts: OnceLock<Vec<String>>,
}

impl PartialEq for Command {
    fn eq(&self, other: &Self) -> bool {
        (&self.script, &self.output) == (&other.script, &other.output)
    }
}

impl Command {
    pub fn new(script: impl Into<String>, output: Option<&str>) -> Self {
        Command {
            script: script.into(),
            output: output.map(String::from),
            script_parts: OnceLock::new(),
        }
    }

    /// Returns the output, empty if there's none.
    pub fn output(&self) -> &str {
        self.output.as_deref().unwrap_or("")
    }

    /// Returns the script split in the way the shell does it.
    pub fn script_parts(&self) -> &[String] {
        self.script_parts
            .get_or_init(|| shell().split_command(&self.script))
    }

    /// Returns new command with replaced script.
    pub fn with_script(&self, script: impl Into<String>) -> Self {
        Command {
            script: script.into(),
            output: self.output.clone(),
            script_parts: OnceLock::new(),
        }
    }

    /// Creates instance of `Command` from a list of script parts, obtaining
    /// the output of the script. Returns `None` for an empty script.
    pub fn from_raw_script(raw_script: &[String]) -> Option<Self> {
        let script = format_raw_script(raw_script);
        if script.is_empty() {
            return None;
        }

        let expanded = shell().from_shell(&script);
        let output = get_output(&script, &expanded);
        Some(Command {
            script: expanded,
            output,
            script_parts: OnceLock::new(),
        })
    }
}

pub type MatchFn = fn(&Command) -> bool;
pub type GetNewCommandFn = fn(&Command) -> Vec<String>;
/// Called with the old command and the fixed script before the latter is run.
pub type SideEffectFn = fn(&Command, &str);

/// Rule for fixing commands.
#[derive(Clone, Copy)]
pub struct Rule {
    pub name: &'static str,
    pub match_: MatchFn,
    pub get_new_command: GetNewCommandFn,
    /// Evaluated lazily, as it often depends on installed applications.
    pub enabled_by_default: fn() -> bool,
    pub side_effect: Option<SideEffectFn>,
    pub priority: i64,
    pub requires_output: bool,
}

pub fn always() -> bool {
    true
}

pub fn never() -> bool {
    false
}

thread_local! {
    /// Set while a rule runs: `Some` with the description of its panic, if any.
    static RULE_PANIC: RefCell<Option<String>> = const { RefCell::new(None) };
    static IN_RULE: RefCell<bool> = const { RefCell::new(false) };
}

/// Runs code of a rule, converting a panic in it to an error.
///
/// Rules are written against output of various applications, and they trust
/// it a lot. A rule confused by unexpected output shouldn't take the whole
/// application down, it just doesn't apply.
fn guarded<T>(f: impl FnOnce() -> T) -> Result<T, String> {
    static HOOK: Once = Once::new();
    HOOK.call_once(|| {
        let default_hook = panic::take_hook();
        panic::set_hook(Box::new(move |info| {
            if IN_RULE.with(|in_rule| *in_rule.borrow()) {
                RULE_PANIC.with(|panic| *panic.borrow_mut() = Some(info.to_string()));
            } else {
                default_hook(info);
            }
        }));
    });

    IN_RULE.with(|in_rule| *in_rule.borrow_mut() = true);
    let result = panic::catch_unwind(AssertUnwindSafe(f));
    IN_RULE.with(|in_rule| *in_rule.borrow_mut() = false);
    result.map_err(|_| {
        RULE_PANIC
            .with(|panic| panic.borrow_mut().take())
            .unwrap_or_default()
    })
}

impl Rule {
    /// Creates a rule that is enabled by default, has the default priority,
    /// requires output and doesn't have side effects.
    pub const fn new(
        name: &'static str,
        match_: MatchFn,
        get_new_command: GetNewCommandFn,
    ) -> Self {
        Rule {
            name,
            match_,
            get_new_command,
            enabled_by_default: always,
            side_effect: None,
            priority: DEFAULT_PRIORITY,
            requires_output: true,
        }
    }

    /// Returns `true` when rule enabled.
    pub fn is_enabled(&self) -> bool {
        let rules = &settings().rules;
        rules.iter().any(|rule| rule == self.name)
            || (rules.iter().any(|rule| rule == ALL_ENABLED) && (self.enabled_by_default)())
    }

    /// Returns `true` if rule matches the command.
    pub fn is_match(&self, command: &Command) -> bool {
        if command.output.is_none() && self.requires_output {
            return false;
        }

        let matched = logs::debug_time(&format!("Trying rule: {};", self.name), || {
            guarded(|| (self.match_)(command))
        });
        matched.unwrap_or_else(|details| {
            logs::rule_failed(self.name, &details);
            false
        })
    }

    /// Returns corrected commands.
    pub fn get_corrected_commands(&self, command: &Command) -> Vec<CorrectedCommand> {
        let new_commands = guarded(|| (self.get_new_command)(command)).unwrap_or_else(|details| {
            logs::rule_failed(self.name, &details);
            vec![]
        });
        new_commands
            .into_iter()
            .enumerate()
            .map(|(n, script)| CorrectedCommand {
                script,
                side_effect: self.side_effect,
                priority: (n as i64 + 1) * self.priority,
            })
            .collect()
    }
}

/// Corrected by rule command.
#[derive(Clone, Debug)]
pub struct CorrectedCommand {
    pub script: String,
    pub side_effect: Option<SideEffectFn>,
    pub priority: i64,
}

/// Ignores `priority` field.
impl PartialEq for CorrectedCommand {
    fn eq(&self, other: &Self) -> bool {
        let address = |side_effect: Option<SideEffectFn>| side_effect.map(|f| f as usize);
        self.script == other.script && address(self.side_effect) == address(other.side_effect)
    }
}

impl CorrectedCommand {
    /// Returns fixed commands script.
    ///
    /// If `settings.repeat` is `true`, appends command with second attempt
    /// of running fuck in case fixed command fails again.
    fn get_script(&self) -> String {
        if settings().repeat {
            let repeat_fuck = format!(
                "{} --repeat {}--force-command {}",
                get_alias(),
                if settings().debug { "--debug " } else { "" },
                shell().quote(&self.script)
            );
            shell().or_(&[&self.script, &repeat_fuck])
        } else {
            self.script.clone()
        }
    }

    /// Runs command from rule for passed command: actually prints it, the
    /// shell alias takes care of the rest.
    pub fn run(&self, old_cmd: &Command) {
        if let Some(side_effect) = self.side_effect {
            if let Err(details) = guarded(|| side_effect(old_cmd, &self.script)) {
                logs::exception("Side effect failed", &details);
            }
        }
        if settings().alter_history {
            shell().put_to_history(&self.script);
        }

        print!("{}", self.get_script());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn matches(_: &Command) -> bool {
        true
    }

    fn panics(command: &Command) -> bool {
        command.script_parts()[10] == "x"
    }

    fn new_commands(command: &Command) -> Vec<String> {
        vec![
            format!("{} --first", command.script),
            format!("{} --second", command.script),
        ]
    }

    #[test]
    fn rule_needs_output_unless_told_otherwise() {
        let rule = Rule::new("test", matches, new_commands);
        assert!(rule.is_match(&Command::new("ls", Some(""))));
        assert!(!rule.is_match(&Command::new("ls", None)));
        assert!(Rule {
            requires_output: false,
            ..rule
        }
        .is_match(&Command::new("ls", None)));
    }

    #[test]
    fn failed_rule_does_not_match() {
        let rule = Rule::new("test", panics, new_commands);
        assert!(!rule.is_match(&Command::new("ls", Some(""))));
    }

    #[test]
    fn corrected_commands_get_growing_priority() {
        let rule = Rule {
            priority: 100,
            ..Rule::new("test", matches, new_commands)
        };
        let corrected = rule.get_corrected_commands(&Command::new("ls", Some("")));
        let summary: Vec<_> = corrected
            .iter()
            .map(|c| (c.script.as_str(), c.priority))
            .collect();
        assert_eq!(summary, [("ls --first", 100), ("ls --second", 200)]);
    }

    #[test]
    fn rule_is_enabled_by_default_or_by_name() {
        let rule = Rule::new("test", matches, new_commands);
        assert!(rule.is_enabled());
        assert!(!Rule {
            enabled_by_default: never,
            ..rule
        }
        .is_enabled());
    }

    #[test]
    fn script_is_split_lazily() {
        let command = Command::new("git commit -m 'a b'", Some(""));
        assert_eq!(command.script_parts(), ["git", "commit", "-m", "a b"]);
        assert_eq!(command.with_script("ls -la").script_parts(), ["ls", "-la"]);
    }
}
