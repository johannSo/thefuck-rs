use super::prelude::*;

const INVALID_CHOICE: &str = "Invalid choice: '(.*)', maybe you meant:";
const OPTIONS: &str = r"(?m)^\s*\*\s(.*)";

fn match_(command: &Command) -> bool {
    for_app(command, &["aws"]) && command.output().contains("usage:") && command.output().contains("maybe you meant:")
}

fn get_new_command(command: &Command) -> Vec<String> {
    let mistake = &findall(INVALID_CHOICE, command.output())[0];
    let options = findall(OPTIONS, command.output());
    options.iter().map(|option| replace_argument(&command.script, mistake, option)).collect()
}

pub const RULE: Rule = Rule::new("aws_cli", match_, get_new_command);
