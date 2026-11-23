use super::prelude::*;

fn match_(command: &Command) -> bool {
    for_app(command, &["apt-get"]) && command.script.starts_with("apt-get search")
}

fn get_new_command(command: &Command) -> Vec<String> {
    vec![regex("^apt-get").replace(&command.script, "apt-cache").into_owned()]
}

pub const RULE: Rule =
    Rule { enabled_by_default: apt_available, ..Rule::new("apt_get_search", match_, get_new_command) };
