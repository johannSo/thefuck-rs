use super::prelude::*;

fn match_(command: &Command) -> bool {
    git_support(command, |command| {
        command.output().contains(" is not a git command. See 'git --help'.")
            && (command.output().contains("The most similar command") || command.output().contains("Did you mean"))
    })
}

fn get_new_command(command: &Command) -> Vec<String> {
    git_support(command, |command| {
        let broken_cmd = &findall(r"git: '([^']*)' is not a git command", command.output())[0];
        let matched = get_all_matched_commands(command.output(), &["The most similar command", "Did you mean"]);
        replace_command(command, broken_cmd, &matched)
    })
}

pub const RULE: Rule = Rule::new("git_not_command", match_, get_new_command);
