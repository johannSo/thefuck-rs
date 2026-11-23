use std::path::Path;

use super::prelude::*;

fn match_(command: &Command) -> bool {
    for_app(command, &["gradle"])
        && which(&command.script_parts()[0]).is_none()
        && command.output().contains("not found")
        && Path::new("gradlew").is_file()
}

fn get_new_command(command: &Command) -> Vec<String> {
    vec![format!("./gradlew {}", command.script_parts()[1..].join(" "))]
}

pub const RULE: Rule = Rule::new("gradle_wrapper", match_, get_new_command);
