use super::prelude::*;

const HOOKED_COMMANDS: &[&str] = &["am", "commit", "push"];

fn hooked_command(command: &Command) -> Option<&'static str> {
    HOOKED_COMMANDS
        .iter()
        .copied()
        .find(|hooked_command| command.script_parts().iter().any(|part| part == hooked_command))
}

fn match_(command: &Command) -> bool {
    git_support(command, |command| hooked_command(command).is_some())
}

fn get_new_command(command: &Command) -> Vec<String> {
    git_support(command, |command| {
        hooked_command(command)
            .map(|hooked| replace_argument(&command.script, hooked, &format!("{} --no-verify", hooked)))
            .into_iter()
            .collect()
    })
}

pub const RULE: Rule =
    Rule { priority: 1100, requires_output: false, ..Rule::new("git_hook_bypass", match_, get_new_command) };
