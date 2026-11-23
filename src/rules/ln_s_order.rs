use std::path::Path;

use super::prelude::*;

/// When arguments order is wrong first argument will be destination.
fn get_destination(script_parts: &[String]) -> Option<&String> {
    script_parts
        .iter()
        .find(|part| !["ln", "-s", "--symbolic"].contains(&part.as_str()) && Path::new(part).exists())
}

fn match_(command: &Command) -> bool {
    sudo_support(command, |command| {
        let parts = command.script_parts();
        parts[0] == "ln"
            && parts.iter().any(|part| part == "-s" || part == "--symbolic")
            && command.output().contains("File exists")
            && get_destination(parts).is_some()
    })
}

fn get_new_command(command: &Command) -> Vec<String> {
    sudo_support(command, |command| {
        let Some(destination) = get_destination(command.script_parts()).cloned() else { return vec![] };
        let mut parts = command.script_parts().to_vec();
        if let Some(position) = parts.iter().position(|part| *part == destination) {
            parts.remove(position);
        }
        parts.push(destination);
        vec![parts.join(" ")]
    })
}

pub const RULE: Rule = Rule::new("ln_s_order", match_, get_new_command);
