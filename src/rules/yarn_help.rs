use super::prelude::*;
use crate::system::open_command;

fn match_(command: &Command) -> bool {
    is_app(command, &["yarn"], 2)
        && command.script_parts()[1] == "help"
        && command.output().contains("for documentation about this command.")
}

fn get_new_command(command: &Command) -> Vec<String> {
    let url = &findall(r"Visit ([^ ]*) for documentation about this command.", command.output())[0];

    vec![open_command(url)]
}

pub const RULE: Rule = Rule::new("yarn_help", match_, get_new_command);
