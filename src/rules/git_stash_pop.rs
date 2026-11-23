use super::prelude::*;

fn match_(command: &Command) -> bool {
    git_support(command, |command| {
        command.script.contains("stash")
            && command.script.contains("pop")
            && command
                .output()
                .contains("Your local changes to the following files would be overwritten by merge")
    })
}

fn get_new_command(command: &Command) -> Vec<String> {
    git_support(command, |_| vec![and_(&["git add --update", "git stash pop", "git reset ."])])
}

// make it come before the other applicable rules
pub const RULE: Rule = Rule { priority: 900, ..Rule::new("git_stash_pop", match_, get_new_command) };
