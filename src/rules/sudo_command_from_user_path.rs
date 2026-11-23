use super::prelude::*;

fn get_command_name(command: &Command) -> Option<String> {
    find_first(r"sudo: (.*): command not found", command.output())
}

fn match_(command: &Command) -> bool {
    for_app(command, &["sudo"])
        && command.output().contains("command not found")
        && which(&get_command_name(command).expect("sudo names the command")).is_some()
}

fn get_new_command(command: &Command) -> Vec<String> {
    let Some(command_name) = get_command_name(command) else { return vec![] };
    vec![replace_argument(&command.script, &command_name, &format!("env \"PATH=$PATH\" {}", command_name))]
}

pub const RULE: Rule = Rule::new("sudo_command_from_user_path", match_, get_new_command);
