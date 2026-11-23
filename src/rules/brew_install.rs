use super::prelude::*;

fn get_suggestions(suggestions: &str) -> Vec<String> {
    suggestions.replace(" or ", ", ").split(", ").map(String::from).collect()
}

fn match_(command: &Command) -> bool {
    is_app(command, &["brew"], 2)
        && command.script.contains("install")
        && command.output().contains("No available formula")
        && command.output().contains("Did you mean")
}

fn get_new_command(command: &Command) -> Vec<String> {
    let matcher = &findall(
        r#"Warning: No available formula with the name "(?:[^"]+)". Did you mean (.+)\?"#,
        command.output(),
    )[0];
    get_suggestions(matcher).iter().map(|formula| format!("brew install {}", formula)).collect()
}

pub const RULE: Rule = Rule { enabled_by_default: brew_available, ..Rule::new("brew_install", match_, get_new_command) };
