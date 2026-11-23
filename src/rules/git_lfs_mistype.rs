use super::prelude::*;

/// Match a mistyped command
fn match_(command: &Command) -> bool {
    git_support(command, |command| {
        command.script.contains("lfs") && command.output().contains("Did you mean this?")
    })
}

fn get_new_command(command: &Command) -> Vec<String> {
    git_support(command, |command| {
        let broken_cmd = &findall(r#"Error: unknown command "([^"]*)" for "git-lfs""#, command.output())[0];
        let matched = get_all_matched_commands(command.output(), &["Did you mean", " for usage."]);
        replace_command(command, broken_cmd, &matched)
    })
}

pub const RULE: Rule = Rule::new("git_lfs_mistype", match_, get_new_command);
