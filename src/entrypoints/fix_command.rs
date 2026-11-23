use crate::argument_parser::Args;
use crate::conf::{self, settings};
use crate::consts::DIFF_WITH_ALIAS;
use crate::corrector::get_corrected_commands;
use crate::difflib::SequenceMatcher;
use crate::logs;
use crate::types::Command;
use crate::ui::{read_action, select_command};
use crate::utils::{get_alias, get_all_executables};

/// Picks the command to fix from the last history entries: the latest one
/// that doesn't look like a call of The Fuck itself.
fn command_from_history(history: &str, alias: &str, executables: &[String]) -> Vec<String> {
    history
        .split('\n')
        .rev()
        .find(|command| {
            let diff = SequenceMatcher::new(alias, command).ratio();
            diff < DIFF_WITH_ALIAS || executables.iter().any(|executable| executable == command)
        })
        .map_or_else(Vec::new, |command| vec![command.to_string()])
}

fn get_raw_command(known_args: &Args) -> Vec<String> {
    if let Some(force_command) = &known_args.force_command {
        return vec![force_command.clone()];
    }
    match std::env::var("TF_HISTORY") {
        Ok(history) if !history.is_empty() => {
            command_from_history(&history, &get_alias(), get_all_executables())
        }
        _ => known_args.command.clone(),
    }
}

/// Fixes previous command. Used when `thefuck` called without arguments.
pub fn fix_command(known_args: &Args) {
    conf::init(Some(known_args));
    let fixed = logs::debug_time("Total", || {
        logs::debug(&format!("Run with settings: {:#?}", settings()));
        let raw_command = get_raw_command(known_args);

        let Some(command) = Command::from_raw_script(&raw_command) else {
            logs::debug("Empty command, nothing to do");
            return true;
        };

        let corrected_commands = get_corrected_commands(&command);
        match select_command(corrected_commands, read_action) {
            Some(selected_command) => {
                selected_command.run(&command);
                true
            }
            None => false,
        }
    });

    if !fixed {
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::command_from_history;

    #[test]
    fn takes_the_last_command_that_is_not_the_alias() {
        let executables = ["ls".to_string(), "fucc".to_string()];
        assert_eq!(
            command_from_history("git br\nfuck", "fuck", &executables),
            ["git br"]
        );
        assert_eq!(
            command_from_history("git br\nfcuk\nfuck", "fuck", &executables),
            ["git br"]
        );
        assert_eq!(
            command_from_history("git br\nfucc\nfuck", "fuck", &executables),
            ["fucc"]
        );
        assert_eq!(
            command_from_history("\tgit br\n\tls", "fuck", &executables),
            ["\tls"]
        );
        assert!(command_from_history("fuck\nfuck", "fuck", &executables).is_empty());
    }
}
