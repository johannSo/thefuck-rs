use super::prelude::*;

const WARNING_REGEX: &str = "Warning: .+ is already installed and up-to-date";
const MESSAGE_REGEX: &str = "To reinstall .+?, run `brew reinstall [^`]+`";

fn match_(command: &Command) -> bool {
    is_app(command, &["brew"], 2)
        && command.script.contains("install")
        && regex(WARNING_REGEX).is_match(command.output())
        && regex(MESSAGE_REGEX).is_match(command.output())
}

fn get_new_command(command: &Command) -> Vec<String> {
    vec![command.script.replace("install", "reinstall")]
}

pub const RULE: Rule = Rule::new("brew_reinstall", match_, get_new_command);
