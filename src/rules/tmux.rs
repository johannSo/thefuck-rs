use super::prelude::*;

fn match_(command: &Command) -> bool {
    for_app(command, &["tmux"])
        && command.output().contains("ambiguous command:")
        && command.output().contains("could be:")
}

fn get_new_command(command: &Command) -> Vec<String> {
    let cmd = regex(r"^ambiguous command: (.*), could be: (.*)")
        .captures(command.output())
        .expect("the output starts with the error");

    let old_cmd = &cmd[1];
    let suggestions: Vec<&str> = cmd[2].split(',').map(str::trim).collect();

    replace_command(command, old_cmd, &suggestions)
}

pub const RULE: Rule = Rule::new("tmux", match_, get_new_command);
