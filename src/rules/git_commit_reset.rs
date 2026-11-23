use super::prelude::*;

fn match_(command: &Command) -> bool {
    git_support(command, |command| command.script_parts().iter().any(|part| part == "commit"))
}

fn get_new_command(command: &Command) -> Vec<String> {
    git_support(command, |_| one("git reset HEAD~"))
}

pub const RULE: Rule = Rule::new("git_commit_reset", match_, get_new_command);
