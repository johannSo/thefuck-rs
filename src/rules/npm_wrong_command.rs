use super::prelude::*;

fn get_wrong_command(script_parts: &[String]) -> Option<&String> {
    script_parts[1..].iter().find(|part| !part.starts_with('-'))
}

fn match_(command: &Command) -> bool {
    sudo_support(command, |command| {
        for_app(command, &["npm"])
            && command.script_parts()[0] == "npm"
            && command.output().contains("where <command> is one of:")
            && get_wrong_command(command.script_parts()).is_some()
    })
}

fn get_available_commands(stdout: &str) -> Vec<String> {
    let mut commands_listing = false;
    let mut commands = Vec::new();
    for line in stdout.split('\n') {
        if line.starts_with("where <command> is one of:") {
            commands_listing = true;
        } else if commands_listing {
            if line.is_empty() {
                break;
            }

            commands.extend(line.split(", ").map(str::trim).filter(|command| !command.is_empty()).map(String::from));
        }
    }
    commands
}

fn get_new_command(command: &Command) -> Vec<String> {
    let npm_commands = get_available_commands(command.output());
    let Some(wrong_command) = get_wrong_command(command.script_parts()) else { return vec![] };
    get_closest(wrong_command, &npm_commands, 0.6, true)
        .map(|fixed| replace_argument(&command.script, wrong_command, &fixed))
        .into_iter()
        .collect()
}

pub const RULE: Rule =
    Rule { enabled_by_default: npm_available, ..Rule::new("npm_wrong_command", match_, get_new_command) };
