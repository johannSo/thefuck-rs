use super::prelude::*;

fn match_(command: &Command) -> bool {
    sudo_support(command, |command| {
        let output = command.output().to_lowercase();
        for_app(command, &["cp"]) && (output.contains("omitting directory") || output.contains("is a directory"))
    })
}

fn get_new_command(command: &Command) -> Vec<String> {
    sudo_support(command, |command| vec![regex("^cp").replace(&command.script, "cp -a").into_owned()])
}

pub const RULE: Rule = Rule::new("cp_omitting_directory", match_, get_new_command);
