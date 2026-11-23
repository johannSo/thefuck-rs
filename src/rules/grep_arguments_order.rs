use std::path::Path;

use super::prelude::*;

fn get_actual_file(parts: &[String]) -> Option<&String> {
    parts[1..].iter().find(|part| Path::new(part).is_file() || Path::new(part).is_dir())
}

fn match_(command: &Command) -> bool {
    for_app(command, &["grep", "egrep"])
        && command.output().contains(": No such file or directory")
        && get_actual_file(command.script_parts()).is_some()
}

fn get_new_command(command: &Command) -> Vec<String> {
    let Some(actual_file) = get_actual_file(command.script_parts()).cloned() else { return vec![] };
    let mut parts = command.script_parts().to_vec();
    // moves actual file to the end of the script
    if let Some(position) = parts.iter().position(|part| *part == actual_file) {
        parts.remove(position);
    }
    parts.push(actual_file);
    vec![parts.join(" ")]
}

pub const RULE: Rule = Rule::new("grep_arguments_order", match_, get_new_command);
