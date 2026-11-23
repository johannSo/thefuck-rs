use super::prelude::*;

fn match_(command: &Command) -> bool {
    git_support(command, |command| command.script.contains("diff") && !command.script.contains("--staged"))
}

fn get_new_command(command: &Command) -> Vec<String> {
    git_support(command, |command| vec![replace_argument(&command.script, "diff", "diff --staged")])
}

pub const RULE: Rule = Rule::new("git_diff_staged", match_, get_new_command);
