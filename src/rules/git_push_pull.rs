use super::prelude::*;

fn match_(command: &Command) -> bool {
    git_support(command, |command| {
        let output = command.output();
        command.script.contains("push")
            && output.contains("! [rejected]")
            && output.contains("failed to push some refs to")
            && (output.contains("Updates were rejected because the tip of your current branch is behind")
                || output.contains("Updates were rejected because the remote contains work that you do"))
    })
}

fn get_new_command(command: &Command) -> Vec<String> {
    git_support(command, |command| {
        vec![and_(&[&replace_argument(&command.script, "push", "pull"), &command.script])]
    })
}

pub const RULE: Rule = Rule::new("git_push_pull", match_, get_new_command);
