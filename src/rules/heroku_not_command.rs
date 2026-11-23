use super::prelude::*;

fn match_(command: &Command) -> bool {
    for_app(command, &["heroku"]) && command.output().contains("Run heroku _ to run")
}

fn get_new_command(command: &Command) -> Vec<String> {
    vec![findall("Run heroku _ to run ([^.]*)", command.output())[0].clone()]
}

pub const RULE: Rule = Rule::new("heroku_not_command", match_, get_new_command);
