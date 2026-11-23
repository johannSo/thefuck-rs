use super::prelude::*;

/// Match a mistyped command
fn match_(command: &Command) -> bool {
    for_app(command, &["conda"]) && command.output().contains("Did you mean 'conda")
}

fn get_new_command(command: &Command) -> Vec<String> {
    let matched = findall(r"'conda ([^']*)'", command.output());
    let broken_cmd = &matched[0];
    let correct_cmd = &matched[1];
    replace_command(command, broken_cmd, &[correct_cmd])
}

pub const RULE: Rule = Rule::new("conda_mistype", match_, get_new_command);
