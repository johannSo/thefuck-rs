use super::prelude::*;

fn match_(command: &Command) -> bool {
    command.script == "cargo"
}

fn get_new_command(_command: &Command) -> Vec<String> {
    one("cargo build")
}

pub const RULE: Rule = Rule::new("cargo", match_, get_new_command);
