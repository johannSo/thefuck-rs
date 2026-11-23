use super::prelude::*;

fn match_(command: &Command) -> bool {
    let split_command = command.script_parts();

    split_command.len() >= 2 && split_command[0] == split_command[1]
}

fn get_new_command(command: &Command) -> Vec<String> {
    vec![command.script_parts()[1..].join(" ")]
}

// it should be rare enough to actually have to type twice the same word, so
// this rule can have a higher priority to come before things like "cd cd foo"
pub const RULE: Rule = Rule { priority: 900, ..Rule::new("dry", match_, get_new_command) };
