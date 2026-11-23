use super::prelude::*;

fn match_(command: &Command) -> bool {
    git_support(command, |command| regex(r"src refspec \w+ does not match any").is_match(command.output()))
}

fn get_new_command(command: &Command) -> Vec<String> {
    vec![and_(&["git commit -m \"Initial commit\"", &command.script])]
}

pub const RULE: Rule = Rule::new("git_push_without_commits", match_, get_new_command);
