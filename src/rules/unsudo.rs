use super::prelude::*;

const PATTERNS: &[&str] = &["you cannot perform this operation as root"];

fn match_(command: &Command) -> bool {
    if command.script_parts().first().is_some_and(|first| first != "sudo") {
        return false;
    }

    let output = command.output().to_lowercase();
    PATTERNS.iter().any(|pattern| output.contains(pattern))
}

fn get_new_command(command: &Command) -> Vec<String> {
    vec![command.script_parts()[1..].join(" ")]
}

pub const RULE: Rule = Rule::new("unsudo", match_, get_new_command);
