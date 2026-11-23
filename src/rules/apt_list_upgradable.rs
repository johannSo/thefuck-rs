use super::prelude::*;

fn match_(command: &Command) -> bool {
    sudo_support(command, |command| {
        for_app(command, &["apt"]) && command.output().contains("apt list --upgradable")
    })
}

fn get_new_command(command: &Command) -> Vec<String> {
    sudo_support(command, |_| one("apt list --upgradable"))
}

pub const RULE: Rule =
    Rule { enabled_by_default: apt_available, ..Rule::new("apt_list_upgradable", match_, get_new_command) };
