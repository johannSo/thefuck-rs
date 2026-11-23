use super::prelude::*;

fn match_(command: &Command) -> bool {
    sudo_support(command, |command| {
        command.script.contains("mkdir") && command.output().contains("No such file or directory")
    })
}

fn get_new_command(command: &Command) -> Vec<String> {
    sudo_support(command, |command| {
        vec![regex(r"\bmkdir (.*)").replace_all(&command.script, "mkdir -p ${1}").into_owned()]
    })
}

pub const RULE: Rule = Rule::new("mkdir_p", match_, get_new_command);
