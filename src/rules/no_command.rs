use super::prelude::*;
use crate::specific::subcommands::fix_subcommand;

fn match_(command: &Command) -> bool {
    sudo_support(command, |command| {
        let executable = &command.script_parts()[0];
        which(executable).is_none()
            && (command.output().contains("not found") || command.output().contains("is not recognized as"))
            && !get_close_matches(executable, get_all_executables(), 0.6).is_empty()
    })
}

fn get_used_executables(command: &Command) -> Vec<String> {
    get_valid_history_without_current(command)
        .iter()
        .map(|script| script.split(' ').next().unwrap_or("").to_string())
        .collect()
}

fn get_new_command(command: &Command) -> Vec<String> {
    sudo_support(command, |command| {
        let old_command = &command.script_parts()[0];

        // One from history:
        let already_used = get_closest(old_command, &get_used_executables(command), 0.6, false);

        // Other from all executables:
        let mut new_cmds: Vec<String> = already_used.into_iter().collect();
        for cmd in get_close_matches(old_command, get_all_executables(), 0.6) {
            if !new_cmds.contains(&cmd) {
                new_cmds.push(cmd);
            }
        }

        // A mistyped subcommand is fixed too, `gti atus` becomes `git status`
        // right away instead of `git atus` that needs fixing once more.
        let mut new_scripts: Vec<String> = Vec::new();
        for cmd in &new_cmds {
            let new_command = command.with_script(command.script.replacen(old_command.as_str(), cmd, 1));
            for script in fix_subcommand(&new_command) {
                if !new_scripts.contains(&script) {
                    new_scripts.push(script);
                }
            }
        }
        new_scripts
    })
}

pub const RULE: Rule = Rule { priority: 3000, ..Rule::new("no_command", match_, get_new_command) };
