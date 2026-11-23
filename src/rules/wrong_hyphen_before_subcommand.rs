use super::prelude::*;

fn match_(command: &Command) -> bool {
    sudo_support(command, |command| {
        let first_part = &command.script_parts()[0];
        if !first_part.contains('-') || get_all_executables().contains(first_part) {
            return false;
        }
        let cmd = first_part.split('-').next().unwrap_or("");
        get_all_executables().iter().any(|executable| executable == cmd)
    })
}

fn get_new_command(command: &Command) -> Vec<String> {
    sudo_support(command, |command| vec![command.script.replacen('-', " ", 1)])
}

pub const RULE: Rule = Rule {
    priority: 4500,
    requires_output: false,
    ..Rule::new("wrong_hyphen_before_subcommand", match_, get_new_command)
};
