use super::prelude::*;

fn match_(command: &Command) -> bool {
    git_support(command, |command| {
        command.script_parts().iter().any(|part| part == "add")
            && command.output().contains("Use -f if you really want to add them.")
    })
}

fn get_new_command(command: &Command) -> Vec<String> {
    git_support(command, |command| vec![replace_argument(&command.script, "add", "add --force")])
}

pub const RULE: Rule = Rule::new("git_add_force", match_, get_new_command);
