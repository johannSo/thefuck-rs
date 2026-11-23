use super::prelude::*;

fn match_(command: &Command) -> bool {
    git_support(command, |command| {
        command.script.contains(" git clone ") && command.output().contains("fatal: Too many arguments.")
    })
}

fn get_new_command(command: &Command) -> Vec<String> {
    git_support(command, |command| vec![command.script.replacen(" git clone ", " ", 1)])
}

pub const RULE: Rule = Rule::new("git_clone_git_clone", match_, get_new_command);
