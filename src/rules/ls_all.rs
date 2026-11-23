use super::prelude::*;

fn match_(command: &Command) -> bool {
    for_app(command, &["ls"]) && command.output().trim().is_empty()
}

fn get_new_command(command: &Command) -> Vec<String> {
    let mut parts = strings(&["ls", "-A"]);
    parts.extend_from_slice(&command.script_parts()[1..]);
    vec![parts.join(" ")]
}

pub const RULE: Rule = Rule::new("ls_all", match_, get_new_command);
