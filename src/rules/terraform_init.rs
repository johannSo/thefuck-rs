use super::prelude::*;

fn match_(command: &Command) -> bool {
    let output = command.output().to_lowercase();
    for_app(command, &["terraform"])
        && (output.contains("this module is not yet installed") || output.contains("initialization required"))
}

fn get_new_command(command: &Command) -> Vec<String> {
    vec![and_(&["terraform init", &command.script])]
}

pub const RULE: Rule = Rule::new("terraform_init", match_, get_new_command);
