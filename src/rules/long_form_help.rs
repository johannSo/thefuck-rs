use super::prelude::*;

const HELP_REGEX: &str = r"(?:Run|Try) '([^']+)'(?: or '[^']+')? for (?:details|more information).";

fn match_(command: &Command) -> bool {
    regex(&format!("(?i){}", HELP_REGEX)).is_match(command.output()) || command.output().contains("--help")
}

fn get_new_command(command: &Command) -> Vec<String> {
    if regex(HELP_REGEX).is_match(command.output()) {
        return find_first(&format!("(?i){}", HELP_REGEX), command.output()).into_iter().collect();
    }

    vec![replace_argument(&command.script, "-h", "--help")]
}

pub const RULE: Rule = Rule { priority: 5000, ..Rule::new("long_form_help", match_, get_new_command) };
