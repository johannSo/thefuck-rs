use super::prelude::*;

fn match_(command: &Command) -> bool {
    git_support(command, |command| {
        command.script.contains("set-url") && command.output().contains("fatal: No such remote")
    })
}

fn get_new_command(command: &Command) -> Vec<String> {
    vec![replace_argument(&command.script, "set-url", "add")]
}

pub const RULE: Rule = Rule::new("git_remote_seturl_add", match_, get_new_command);
