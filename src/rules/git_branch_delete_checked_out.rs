use super::prelude::*;

fn match_(command: &Command) -> bool {
    git_support(command, |command| {
        (command.script.contains("branch -d") || command.script.contains("branch -D"))
            && command.output().contains("error: Cannot delete branch '")
            && command.output().contains("' checked out at '")
    })
}

fn get_new_command(command: &Command) -> Vec<String> {
    git_support(command, |command| {
        vec![and_(&["git checkout master", &replace_argument(&command.script, "-d", "-D")])]
    })
}

pub const RULE: Rule = Rule::new("git_branch_delete_checked_out", match_, get_new_command);
