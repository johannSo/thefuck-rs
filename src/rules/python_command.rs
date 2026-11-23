use super::prelude::*;

fn match_(command: &Command) -> bool {
    sudo_support(command, |command| {
        command.script_parts().first().is_some_and(|script| script.ends_with(".py"))
            && (command.output().contains("Permission denied") || command.output().contains("command not found"))
    })
}

fn get_new_command(command: &Command) -> Vec<String> {
    sudo_support(command, |command| vec![format!("python {}", command.script)])
}

pub const RULE: Rule = Rule::new("python_command", match_, get_new_command);
