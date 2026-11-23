use super::prelude::*;

const INVALID_CHOICE: &str = "az.*: '(.*)' is not in the '.*' command group.";
const OPTIONS: &str = "(?m)^The most similar choice to '.*' is:\n\\s*(.*)$";

fn match_(command: &Command) -> bool {
    for_app(command, &["az"])
        && command.output().contains("is not in the")
        && command.output().contains("command group")
}

fn get_new_command(command: &Command) -> Vec<String> {
    let mistake = &findall(INVALID_CHOICE, command.output())[0];
    let options = findall(OPTIONS, command.output());
    options.iter().map(|option| replace_argument(&command.script, mistake, option)).collect()
}

pub const RULE: Rule = Rule::new("az_cli", match_, get_new_command);
