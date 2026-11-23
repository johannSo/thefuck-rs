use super::prelude::*;

const REGEX: &str = r#"Run "(.*)" instead"#;

fn match_(command: &Command) -> bool {
    is_app(command, &["yarn"], 1) && regex(REGEX).is_match(command.output())
}

fn get_new_command(command: &Command) -> Vec<String> {
    vec![findall(REGEX, command.output())[0].clone()]
}

pub const RULE: Rule = Rule::new("yarn_command_replaced", match_, get_new_command);
