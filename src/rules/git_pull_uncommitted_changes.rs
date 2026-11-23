use super::prelude::*;

fn match_(command: &Command) -> bool {
    git_support(command, |command| {
        command.script.contains("pull")
            && (command.output().contains("You have unstaged changes")
                || command.output().contains("contains uncommitted changes"))
    })
}

fn get_new_command(command: &Command) -> Vec<String> {
    git_support(command, |_| vec![and_(&["git stash", "git pull", "git stash pop"])])
}

pub const RULE: Rule = Rule::new("git_pull_uncommitted_changes", match_, get_new_command);
