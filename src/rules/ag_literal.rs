use super::prelude::*;

fn match_(command: &Command) -> bool {
    for_app(command, &["ag"]) && command.output().ends_with("run ag with -Q\n")
}

fn get_new_command(command: &Command) -> Vec<String> {
    vec![command.script.replacen("ag", "ag -Q", 1)]
}

pub const RULE: Rule = Rule::new("ag_literal", match_, get_new_command);
