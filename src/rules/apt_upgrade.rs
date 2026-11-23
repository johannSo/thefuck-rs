use super::prelude::*;

fn match_(command: &Command) -> bool {
    sudo_support(command, |command| {
        for_app(command, &["apt"])
            && command.script == "apt list --upgradable"
            && command.output().trim().split('\n').count() > 1
    })
}

fn get_new_command(command: &Command) -> Vec<String> {
    sudo_support(command, |_| one("apt upgrade"))
}

pub const RULE: Rule = Rule { enabled_by_default: apt_available, ..Rule::new("apt_upgrade", match_, get_new_command) };
