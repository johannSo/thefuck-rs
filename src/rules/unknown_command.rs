use super::prelude::*;

const BROKEN: &str = r"([^:]*): Unknown command.*";
const SUGGESTION: &str = r"Did you mean ([^?]*)?";

fn match_(command: &Command) -> bool {
    regex(BROKEN).is_match(command.output()) && regex(SUGGESTION).is_match(command.output())
}

fn get_new_command(command: &Command) -> Vec<String> {
    let broken_cmd = &findall(BROKEN, command.output())[0];
    let matched = findall(SUGGESTION, command.output());
    replace_command(command, broken_cmd, &matched)
}

pub const RULE: Rule = Rule::new("unknown_command", match_, get_new_command);
