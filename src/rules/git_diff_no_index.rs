use super::prelude::*;

fn match_(command: &Command) -> bool {
    git_support(command, |command| {
        let files = command.script_parts().iter().skip(2).filter(|arg| !arg.starts_with('-'));
        command.script.contains("diff") && !command.script.contains("--no-index") && files.count() == 2
    })
}

fn get_new_command(command: &Command) -> Vec<String> {
    git_support(command, |command| vec![replace_argument(&command.script, "diff", "diff --no-index")])
}

pub const RULE: Rule = Rule::new("git_diff_no_index", match_, get_new_command);
