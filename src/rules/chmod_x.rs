use std::path::Path;

use super::prelude::*;

fn match_(command: &Command) -> bool {
    command.script.starts_with("./")
        && command.output().to_lowercase().contains("permission denied")
        && Path::new(&command.script_parts()[0]).exists()
        && !is_executable(Path::new(&command.script_parts()[0]))
}

fn get_new_command(command: &Command) -> Vec<String> {
    vec![and_(&[&format!("chmod +x {}", &command.script_parts()[0][2..]), &command.script])]
}

pub const RULE: Rule = Rule::new("chmod_x", match_, get_new_command);
