use super::prelude::*;

fn get_executable(script_part: &str) -> Option<&'static String> {
    get_all_executables()
        .iter()
        .find(|executable| executable.chars().count() > 1 && script_part.starts_with(executable.as_str()))
}

fn match_(command: &Command) -> bool {
    let first = &command.script_parts()[0];
    !get_all_executables().contains(first) && get_executable(first).is_some()
}

fn get_new_command(command: &Command) -> Vec<String> {
    let Some(executable) = get_executable(&command.script_parts()[0]) else { return vec![] };
    vec![command.script.replacen(executable.as_str(), &format!("{} ", executable), 1)]
}

pub const RULE: Rule = Rule { priority: 4000, ..Rule::new("missing_space_before_subcommand", match_, get_new_command) };
