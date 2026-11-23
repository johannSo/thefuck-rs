use super::prelude::*;

fn match_(command: &Command) -> bool {
    let parts = command.script_parts();
    is_app(command, &["php"], 2) && parts.iter().any(|part| part == "-s") && parts[parts.len() - 1] != "-s"
}

fn get_new_command(command: &Command) -> Vec<String> {
    vec![replace_argument(&command.script, "-s", "-S")]
}

pub const RULE: Rule = Rule::new("php_s", match_, get_new_command);
