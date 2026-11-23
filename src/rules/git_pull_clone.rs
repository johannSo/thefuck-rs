use super::prelude::*;

fn match_(command: &Command) -> bool {
    git_support(command, |command| {
        command.output().contains("fatal: Not a git repository")
            && command
                .output()
                .contains("Stopping at filesystem boundary (GIT_DISCOVERY_ACROSS_FILESYSTEM not set).")
    })
}

fn get_new_command(command: &Command) -> Vec<String> {
    git_support(command, |command| vec![replace_argument(&command.script, "pull", "clone")])
}

pub const RULE: Rule = Rule::new("git_pull_clone", match_, get_new_command);
