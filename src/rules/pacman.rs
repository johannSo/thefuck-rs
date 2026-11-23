use super::prelude::*;
use crate::specific::archlinux;

fn match_(command: &Command) -> bool {
    command.output().contains("not found") && !get_pkgfile(&command.script).is_empty()
}

fn get_new_command(command: &Command) -> Vec<String> {
    let Some(pacman) = archlinux::pacman() else { return vec![] };
    get_pkgfile(&command.script)
        .iter()
        .map(|package| and_(&[&format!("{} -S {}", pacman, package), &command.script]))
        .collect()
}

pub const RULE: Rule = Rule { enabled_by_default: archlinux_env, ..Rule::new("pacman", match_, get_new_command) };
