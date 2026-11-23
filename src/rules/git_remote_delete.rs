use super::prelude::*;

fn match_(command: &Command) -> bool {
    git_support(command, |command| command.script.contains("remote delete"))
}

fn get_new_command(command: &Command) -> Vec<String> {
    git_support(command, |command| vec![command.script.replacen("delete", "remove", 1)])
}

pub const RULE: Rule = Rule::new("git_remote_delete", match_, get_new_command);
