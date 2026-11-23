use super::prelude::*;

fn match_(command: &Command) -> bool {
    for_app(command, &["mvn"]) && command.output().contains("No goals have been specified for this build")
}

fn get_new_command(command: &Command) -> Vec<String> {
    vec![format!("{} clean package", command.script), format!("{} clean install", command.script)]
}

pub const RULE: Rule = Rule::new("mvn_no_command", match_, get_new_command);
