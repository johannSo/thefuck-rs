use super::prelude::*;

fn match_(command: &Command) -> bool {
    git_support(command, |command| {
        command.script.contains("help") && command.output().contains(" is aliased to ")
    })
}

fn get_new_command(command: &Command) -> Vec<String> {
    git_support(command, |command| {
        let quoted = command.output().splitn(3, '`').nth(2).expect("the expansion is quoted");
        let aliased = quoted.split('\'').next().unwrap_or("").split(' ').next().unwrap_or("");
        vec![format!("git help {}", aliased)]
    })
}

pub const RULE: Rule = Rule::new("git_help_aliased", match_, get_new_command);
