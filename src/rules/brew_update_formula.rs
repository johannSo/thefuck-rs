use super::prelude::*;

fn match_(command: &Command) -> bool {
    is_app(command, &["brew"], 2)
        && command.script.contains("update")
        && command.output().contains("Error: This command updates brew itself")
        && command.output().contains("Use `brew upgrade")
}

fn get_new_command(command: &Command) -> Vec<String> {
    vec![command.script.replace("update", "upgrade")]
}

pub const RULE: Rule = Rule::new("brew_update_formula", match_, get_new_command);
