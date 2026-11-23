use super::prelude::*;

fn match_(command: &Command) -> bool {
    for_app(command, &["touch"]) && command.output().contains("No such file or directory")
}

fn get_new_command(command: &Command) -> Vec<String> {
    let path = &findall(r"touch: (?:cannot touch ')?(.+)/.+'?:", command.output())[0];
    vec![and_(&[&format!("mkdir -p {}", path), &command.script])]
}

pub const RULE: Rule = Rule::new("touch", match_, get_new_command);
