use super::prelude::*;

const CEDILLA: char = 'ç';

fn match_(command: &Command) -> bool {
    command.script.ends_with(CEDILLA)
}

fn get_new_command(command: &Command) -> Vec<String> {
    vec![command.script.strip_suffix(CEDILLA).unwrap_or(&command.script).to_string()]
}

pub const RULE: Rule = Rule::new("remove_trailing_cedilla", match_, get_new_command);
