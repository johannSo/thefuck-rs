//! Finding fixes for a command.

use std::collections::VecDeque;

use crate::conf::settings;
use crate::logs;
use crate::rules;
use crate::types::{Command, CorrectedCommand, Rule};

/// Returns all enabled rules, the most important first.
pub fn get_rules() -> Vec<Rule> {
    let settings = settings();
    let mut enabled: Vec<Rule> = rules::all()
        .into_iter()
        .filter(|rule| {
            if settings
                .exclude_rules
                .iter()
                .any(|excluded| excluded == rule.name)
            {
                logs::debug(&format!("Ignoring excluded rule: {}", rule.name));
                false
            } else {
                true
            }
        })
        .map(|rule| Rule {
            priority: settings
                .priority
                .get(rule.name)
                .copied()
                .unwrap_or(rule.priority),
            ..rule
        })
        .filter(Rule::is_enabled)
        .collect();
    // The sort is stable, rules with the same priority stay in alphabetical order.
    enabled.sort_by_key(|rule| rule.priority);
    enabled
}

/// Lazily yields corrected commands in order of rules.
///
/// Matching a rule can be slow, some of them call external applications, so
/// rules are tried only when the next command is actually requested.
pub struct CorrectedCommands<'a> {
    command: &'a Command,
    rules: std::vec::IntoIter<Rule>,
    pending: VecDeque<CorrectedCommand>,
}

impl Iterator for CorrectedCommands<'_> {
    type Item = CorrectedCommand;

    fn next(&mut self) -> Option<CorrectedCommand> {
        loop {
            if let Some(corrected) = self.pending.pop_front() {
                return Some(corrected);
            }
            let rule = self.rules.next()?;
            if rule.is_match(self.command) {
                self.pending
                    .extend(rule.get_corrected_commands(self.command));
            }
        }
    }
}

/// Returns corrected commands for the command, not sorted and with duplicates.
pub fn get_corrected_commands(command: &Command) -> CorrectedCommands<'_> {
    get_corrected_commands_using(command, get_rules())
}

pub fn get_corrected_commands_using(command: &Command, rules: Vec<Rule>) -> CorrectedCommands<'_> {
    CorrectedCommands {
        command,
        rules: rules.into_iter(),
        pending: VecDeque::new(),
    }
}

/// Returns the commands going after the `first_command`: sorted by priority
/// and without duplicates.
pub fn organize_commands(
    first_command: &CorrectedCommand,
    corrected_commands: impl Iterator<Item = CorrectedCommand>,
) -> Vec<CorrectedCommand> {
    let mut sorted_commands: Vec<CorrectedCommand> = corrected_commands.collect();
    sorted_commands.sort_by_key(|command| command.priority);

    let mut without_duplicates: Vec<CorrectedCommand> = Vec::new();
    for command in sorted_commands {
        if command != *first_command && !without_duplicates.contains(&command) {
            without_duplicates.push(command);
        }
    }

    if settings().debug {
        let scripts: Vec<&str> = std::iter::once(first_command)
            .chain(&without_duplicates)
            .map(|command| command.script.as_str())
            .collect();
        logs::debug(&format!("Corrected commands: {}", scripts.join(", ")));
    }

    without_duplicates
}

#[cfg(test)]
mod tests {
    use super::*;

    fn corrected(script: &str, priority: i64) -> CorrectedCommand {
        CorrectedCommand {
            script: script.to_string(),
            side_effect: None,
            priority,
        }
    }

    #[test]
    fn organizes_commands() {
        let first = corrected("ls", 100);
        let rest = vec![
            corrected("cd", 100),
            corrected("ls", 100),
            corrected("ls -la", 9000),
            corrected("ls -lh", 200),
            corrected("cd", 50),
        ];
        let organized = organize_commands(&first, rest.into_iter());
        let summary: Vec<_> = organized
            .iter()
            .map(|c| (c.script.as_str(), c.priority))
            .collect();
        assert_eq!(summary, [("cd", 50), ("ls -lh", 200), ("ls -la", 9000)]);
    }

    #[test]
    fn rules_are_tried_lazily_and_in_order() {
        fn matches(_: &Command) -> bool {
            true
        }
        fn mismatches(_: &Command) -> bool {
            false
        }
        fn explodes(_: &Command) -> bool {
            panic!("should not be tried")
        }
        fn fix(command: &Command) -> Vec<String> {
            vec![
                format!("{} fixed", command.script),
                format!("{} again", command.script),
            ]
        }

        let command = Command::new("ls", Some(""));
        let rules = vec![
            Rule::new("a", mismatches, fix),
            Rule::new("b", matches, fix),
            Rule::new("c", explodes, fix),
        ];
        let mut commands = get_corrected_commands_using(&command, rules);
        assert_eq!(commands.next().unwrap().script, "ls fixed");
        assert_eq!(commands.next().unwrap().priority, 2000);
        assert!(commands.next().is_none());
    }

    #[test]
    fn rules_are_unique_and_sorted() {
        let all = rules::all();
        let names: Vec<&str> = all.iter().map(|rule| rule.name).collect();
        let mut sorted = names.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(names, sorted);

        let priorities: Vec<i64> = get_rules().iter().map(|rule| rule.priority).collect();
        assert!(priorities.windows(2).all(|pair| pair[0] <= pair[1]));
    }
}
