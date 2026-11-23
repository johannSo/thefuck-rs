use std::path::Path;

use super::prelude::*;

fn match_(command: &Command) -> bool {
    sudo_support(command, |command| {
        command.script_parts().first().is_some_and(|script| Path::new(script).exists())
            && command.output().contains("command not found")
    })
}

fn get_new_command(command: &Command) -> Vec<String> {
    sudo_support(command, |command| vec![format!("./{}", command.script)])
}

pub const RULE: Rule = Rule::new("has_exists_script", match_, get_new_command);
