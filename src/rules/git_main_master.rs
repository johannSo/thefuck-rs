use super::prelude::*;

fn match_(command: &Command) -> bool {
    git_support(command, |command| {
        command.output().contains("'master'") || command.output().contains("'main'")
    })
}

fn get_new_command(command: &Command) -> Vec<String> {
    git_support(command, |command| {
        if command.output().contains("'master'") {
            vec![command.script.replace("master", "main")]
        } else {
            vec![command.script.replace("main", "master")]
        }
    })
}

pub const RULE: Rule = Rule { priority: 1200, ..Rule::new("git_main_master", match_, get_new_command) };
