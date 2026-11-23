use super::prelude::*;

fn match_(command: &Command) -> bool {
    for_app(command, &["ls"]) && !command.script.contains("ls -")
}

fn get_new_command(command: &Command) -> Vec<String> {
    let mut parts = command.script_parts().to_vec();
    parts[0] = "ls -lah".to_string();
    vec![parts.join(" ")]
}

pub const RULE: Rule = Rule::new("ls_lah", match_, get_new_command);
