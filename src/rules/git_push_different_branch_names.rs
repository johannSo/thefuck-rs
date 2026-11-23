use super::prelude::*;

fn match_(command: &Command) -> bool {
    git_support(command, |command| {
        command.script.contains("push")
            && command.output().contains("The upstream branch of your current branch does not match")
    })
}

fn get_new_command(command: &Command) -> Vec<String> {
    git_support(command, |command| {
        vec![findall(r"(?m)^ +(git push [^\s]+ [^\s]+)", command.output())[0].clone()]
    })
}

pub const RULE: Rule = Rule::new("git_push_different_branch_names", match_, get_new_command);
