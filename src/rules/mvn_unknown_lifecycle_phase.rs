use super::prelude::*;

fn get_failed_lifecycle(command: &Command) -> Option<String> {
    find_first(r#"\[ERROR\] Unknown lifecycle phase "(.+)""#, command.output())
}

fn get_available_lifecycles(command: &Command) -> Option<String> {
    find_first(r"Available lifecycle phases are: (.+) -> \[Help 1\]", command.output())
}

fn match_(command: &Command) -> bool {
    for_app(command, &["mvn"]) && get_available_lifecycles(command).is_some() && get_failed_lifecycle(command).is_some()
}

fn get_new_command(command: &Command) -> Vec<String> {
    match (get_failed_lifecycle(command), get_available_lifecycles(command)) {
        (Some(failed_lifecycle), Some(available_lifecycles)) => {
            let available: Vec<&str> = available_lifecycles.split(", ").collect();
            let selected_lifecycle = get_close_matches(&failed_lifecycle, &available, 0.6);
            replace_command(command, &failed_lifecycle, &selected_lifecycle)
        }
        _ => vec![],
    }
}

pub const RULE: Rule = Rule::new("mvn_unknown_lifecycle_phase", match_, get_new_command);
