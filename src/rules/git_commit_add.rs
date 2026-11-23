use super::prelude::*;

fn match_(command: &Command) -> bool {
    git_support(command, |command| {
        command.script_parts().iter().any(|part| part == "commit")
            && command.output().contains("no changes added to commit")
    })
}

fn get_new_command(command: &Command) -> Vec<String> {
    git_support(command, |command| {
        ["-a", "-p"]
            .iter()
            .map(|opt| replace_argument(&command.script, "commit", &format!("commit {}", opt)))
            .collect()
    })
}

pub const RULE: Rule = Rule::new("git_commit_add", match_, get_new_command);
