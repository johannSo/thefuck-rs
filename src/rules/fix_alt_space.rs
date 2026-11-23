use super::prelude::*;

/// The non-breaking space, typed with alt+space on some keyboard layouts.
const ALT_SPACE: char = '\u{a0}';

fn match_(command: &Command) -> bool {
    sudo_support(command, |command| {
        command.output().to_lowercase().contains("command not found") && command.script.contains(ALT_SPACE)
    })
}

fn get_new_command(command: &Command) -> Vec<String> {
    sudo_support(command, |command| vec![command.script.replace(ALT_SPACE, " ")])
}

pub const RULE: Rule = Rule::new("fix_alt_space", match_, get_new_command);
