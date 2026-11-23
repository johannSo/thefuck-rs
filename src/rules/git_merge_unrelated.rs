use super::prelude::*;

fn match_(command: &Command) -> bool {
    git_support(command, |command| {
        command.script.contains("merge")
            && command.output().contains("fatal: refusing to merge unrelated histories")
    })
}

fn get_new_command(command: &Command) -> Vec<String> {
    git_support(command, |command| vec![format!("{} --allow-unrelated-histories", command.script)])
}

pub const RULE: Rule = Rule::new("git_merge_unrelated", match_, get_new_command);
