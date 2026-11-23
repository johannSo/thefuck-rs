use super::prelude::*;

fn match_(command: &Command) -> bool {
    git_support(command, |command| {
        command.script.contains("pull") && command.output().contains("set-upstream")
    })
}

fn get_new_command(command: &Command) -> Vec<String> {
    git_support(command, |command| {
        let lines: Vec<&str> = command.output().split('\n').collect();
        let line = lines[lines.len() - 3].trim();
        let branch = line.split(' ').next_back().unwrap_or("");
        let set_upstream = line.replace("<remote>", "origin").replace("<branch>", branch);
        vec![and_(&[&set_upstream, &command.script])]
    })
}

pub const RULE: Rule = Rule::new("git_pull", match_, get_new_command);
