use std::path::Path;

use super::prelude::*;

fn get_missing_file(command: &Command) -> Option<String> {
    let pathspec = findall(
        r"error: pathspec '([^']*)' did not match any file\(s\) known to git.",
        command.output(),
    )[0]
    .clone();
    Path::new(&pathspec).exists().then_some(pathspec)
}

fn match_(command: &Command) -> bool {
    git_support(command, |command| {
        command.output().contains("did not match any file(s) known to git.")
            && get_missing_file(command).is_some()
    })
}

fn get_new_command(command: &Command) -> Vec<String> {
    git_support(command, |command| {
        let Some(missing_file) = get_missing_file(command) else { return vec![] };
        vec![and_(&[&format!("git add -- {}", missing_file), &command.script])]
    })
}

pub const RULE: Rule = Rule::new("git_add", match_, get_new_command);
