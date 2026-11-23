use super::prelude::*;

fn match_(command: &Command) -> bool {
    is_app(command, &["cargo"], 1)
        && command.output().to_lowercase().contains("no such subcommand")
        && command.output().contains("Did you mean")
}

fn get_new_command(command: &Command) -> Vec<String> {
    let broken = &command.script_parts()[1];
    let fix = &findall(r"Did you mean `([^`]*)`", command.output())[0];

    vec![replace_argument(&command.script, broken, fix)]
}

pub const RULE: Rule = Rule::new("cargo_no_command", match_, get_new_command);
