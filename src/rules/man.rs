use super::prelude::*;

fn match_(command: &Command) -> bool {
    is_app(command, &["man"], 1)
}

fn get_new_command(command: &Command) -> Vec<String> {
    if command.script.contains('3') {
        return vec![command.script.replace('3', "2")];
    }
    if command.script.contains('2') {
        return vec![command.script.replace('2', "3")];
    }

    let parts = command.script_parts();
    let last_arg = &parts[parts.len() - 1];
    let help_command = format!("{} --help", last_arg);

    // If there are no man pages for last_arg, suggest `last_arg --help` instead.
    // Otherwise, suggest `--help` after suggesting other man page sections.
    if command.output().trim() == format!("No manual entry for {}", last_arg) {
        return vec![help_command];
    }

    let mut split_cmd2 = parts.to_vec();
    let mut split_cmd3 = split_cmd2.clone();

    split_cmd2.insert(1, " 2 ".to_string());
    split_cmd3.insert(1, " 3 ".to_string());

    vec![split_cmd3.concat(), split_cmd2.concat(), help_command]
}

pub const RULE: Rule = Rule::new("man", match_, get_new_command);
