use super::git_rm_local_modifications::get_new_command;
use super::prelude::*;

fn match_(command: &Command) -> bool {
    git_support(command, |command| {
        command.script.contains(" rm ")
            && command.output().contains("error: the following file has changes staged in the index")
            && command.output().contains("use --cached to keep the file, or -f to force removal")
    })
}

pub const RULE: Rule = Rule::new("git_rm_staged", match_, get_new_command);
