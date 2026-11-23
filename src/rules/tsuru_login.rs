use super::prelude::*;

fn match_(command: &Command) -> bool {
    for_app(command, &["tsuru"])
        && command.output().contains("not authenticated")
        && command.output().contains("session has expired")
}

fn get_new_command(command: &Command) -> Vec<String> {
    vec![and_(&["tsuru login", &command.script])]
}

pub const RULE: Rule = Rule::new("tsuru_login", match_, get_new_command);
