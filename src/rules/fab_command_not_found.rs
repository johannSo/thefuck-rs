use super::prelude::*;

fn match_(command: &Command) -> bool {
    for_app(command, &["fab"]) && command.output().contains("Warning: Command(s) not found:")
}

// We need different behavior then in get_all_matched_commands.
fn get_between(content: &str, start: &str, end: Option<&str>) -> Vec<String> {
    let mut should_yield = false;
    let mut found = Vec::new();
    for line in content.split('\n') {
        if line.contains(start) {
            should_yield = true;
            continue;
        }

        if end.is_some_and(|end| line.contains(end)) {
            break;
        }

        if should_yield && !line.is_empty() {
            found.push(line.trim().split(' ').next().unwrap_or("").to_string());
        }
    }
    found
}

fn get_new_command(command: &Command) -> Vec<String> {
    let not_found_commands =
        get_between(command.output(), "Warning: Command(s) not found:", Some("Available commands:"));
    let possible_commands = get_between(command.output(), "Available commands:", None);

    let mut script = command.script.clone();
    for not_found in not_found_commands {
        let Some(fix) = get_closest(&not_found, &possible_commands, 0.6, true) else { continue };
        script = script.replace(&format!(" {}", not_found), &format!(" {}", fix));
    }

    vec![script]
}

pub const RULE: Rule = Rule::new("fab_command_not_found", match_, get_new_command);
