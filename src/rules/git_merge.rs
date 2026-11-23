use super::prelude::*;

fn match_(command: &Command) -> bool {
    git_support(command, |command| {
        command.script.contains("merge")
            && command.output().contains(" - not something we can merge")
            && command.output().contains("Did you mean this?")
    })
}

fn get_new_command(command: &Command) -> Vec<String> {
    git_support(command, |command| {
        let unknown_branch = &findall(r"merge: (.+) - not something we can merge", command.output())[0];
        let remote_branch = &findall(r"Did you mean this\?\n\t([^\n]+)", command.output())[0];

        vec![replace_argument(&command.script, unknown_branch, remote_branch)]
    })
}

pub const RULE: Rule = Rule::new("git_merge", match_, get_new_command);
