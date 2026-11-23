use super::prelude::*;

fn match_(command: &Command) -> bool {
    for_app(command, &["docker"])
        && command.script.contains("docker")
        && command.output().contains("access denied")
        && command.output().contains("may require 'docker login'")
}

fn get_new_command(command: &Command) -> Vec<String> {
    vec![and_(&["docker login", &command.script])]
}

pub const RULE: Rule = Rule::new("docker_login", match_, get_new_command);
