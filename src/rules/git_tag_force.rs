use super::prelude::*;

fn match_(command: &Command) -> bool {
    git_support(command, |command| {
        command.script_parts().iter().any(|part| part == "tag") && command.output().contains("already exists")
    })
}

fn get_new_command(command: &Command) -> Vec<String> {
    git_support(command, |command| vec![replace_argument(&command.script, "tag", "tag --force")])
}

pub const RULE: Rule = Rule::new("git_tag_force", match_, get_new_command);
