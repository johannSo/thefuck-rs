use super::prelude::*;

fn match_(command: &Command) -> bool {
    git_support(command, |command| {
        command.script.contains("branch -d")
            && command.output().contains("If you are sure you want to delete it")
    })
}

fn get_new_command(command: &Command) -> Vec<String> {
    git_support(command, |command| vec![replace_argument(&command.script, "-d", "-D")])
}

pub const RULE: Rule = Rule::new("git_branch_delete", match_, get_new_command);
