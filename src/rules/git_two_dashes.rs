use super::prelude::*;

fn match_(command: &Command) -> bool {
    git_support(command, |command| {
        command.output().contains("error: did you mean `") && command.output().contains("` (with two dashes ?)")
    })
}

fn get_new_command(command: &Command) -> Vec<String> {
    git_support(command, |command| {
        let to = command.output().split('`').nth(1).expect("the option is quoted");
        vec![replace_argument(&command.script, &to[1..], to)]
    })
}

pub const RULE: Rule = Rule::new("git_two_dashes", match_, get_new_command);
