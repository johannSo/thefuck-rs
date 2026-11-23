use super::prelude::*;

fn match_(command: &Command) -> bool {
    command.script.starts_with("man") && command.output().to_lowercase().contains("command not found")
}

fn get_new_command(command: &Command) -> Vec<String> {
    vec![format!("man {}", &command.script[3..])]
}

pub const RULE: Rule = Rule { priority: 2000, ..Rule::new("man_no_space", match_, get_new_command) };
