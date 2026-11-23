use super::prelude::*;

fn match_(command: &Command) -> bool {
    // catches "Please commit or stash them" and "Please, commit your changes or
    // stash them before you can switch branches."
    git_support(command, |command| command.output().contains("or stash them"))
}

fn get_new_command(command: &Command) -> Vec<String> {
    git_support(command, |command| vec![and_(&["git stash", &command.script])])
}

pub const RULE: Rule = Rule::new("git_stash", match_, get_new_command);
