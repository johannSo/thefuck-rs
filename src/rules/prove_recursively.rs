use std::path::Path;

use super::prelude::*;

fn is_recursive(part: &str) -> bool {
    part == "--recurse" || (!part.starts_with("--") && part.starts_with('-') && part.contains('r'))
}

fn isdir(part: &str) -> bool {
    !part.starts_with('-') && Path::new(part).is_dir()
}

fn match_(command: &Command) -> bool {
    for_app(command, &["prove"])
        && command.output().contains("NOTESTS")
        && !command.script_parts()[1..].iter().any(|part| is_recursive(part))
        && command.script_parts()[1..].iter().any(|part| isdir(part))
}

fn get_new_command(command: &Command) -> Vec<String> {
    let mut parts = command.script_parts().to_vec();
    parts.insert(1, "-r".to_string());
    vec![parts.join(" ")]
}

pub const RULE: Rule = Rule::new("prove_recursively", match_, get_new_command);
