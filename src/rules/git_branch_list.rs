use super::prelude::*;

fn match_(command: &Command) -> bool {
    git_support(command, |command| {
        // If there's no script parts, the rule doesn't match
        command.script_parts().get(1..).is_some_and(|rest| rest == ["branch", "list"])
    })
}

fn get_new_command(command: &Command) -> Vec<String> {
    git_support(command, |_| vec![and_(&["git branch --delete list", "git branch"])])
}

pub const RULE: Rule = Rule::new("git_branch_list", match_, get_new_command);
