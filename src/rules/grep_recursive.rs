use super::prelude::*;

fn match_(command: &Command) -> bool {
    for_app(command, &["grep"]) && command.output().to_lowercase().contains("is a directory")
}

fn get_new_command(command: &Command) -> Vec<String> {
    vec![format!("grep -r {}", command.script.chars().skip(5).collect::<String>())]
}

pub const RULE: Rule = Rule::new("grep_recursive", match_, get_new_command);
