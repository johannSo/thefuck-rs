use super::prelude::*;

fn match_(command: &Command) -> bool {
    !get_close_matches(&command.script, &get_valid_history_without_current(command), 0.6).is_empty()
}

fn get_new_command(command: &Command) -> Vec<String> {
    get_closest(&command.script, &get_valid_history_without_current(command), 0.6, true).into_iter().collect()
}

pub const RULE: Rule = Rule { priority: 9999, ..Rule::new("history", match_, get_new_command) };
