use super::prelude::*;

fn match_(command: &Command) -> bool {
    git_support(command, |command| {
        command.script.contains("push")
            && command.output().contains("! [rejected]")
            && command.output().contains("failed to push some refs to")
            && command
                .output()
                .contains("Updates were rejected because the tip of your current branch is behind")
    })
}

fn get_new_command(command: &Command) -> Vec<String> {
    git_support(command, |command| vec![replace_argument(&command.script, "push", "push --force-with-lease")])
}

pub const RULE: Rule = Rule { enabled_by_default: never, ..Rule::new("git_push_force", match_, get_new_command) };
