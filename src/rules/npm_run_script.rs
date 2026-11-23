use super::prelude::*;

fn match_(command: &Command) -> bool {
    for_app(command, &["npm"])
        && command.output().contains("Usage: npm <command>")
        && !command.script_parts().iter().any(|part| part.starts_with("ru"))
        && get_scripts().contains(&command.script_parts()[1])
}

fn get_new_command(command: &Command) -> Vec<String> {
    let mut parts = command.script_parts().to_vec();
    parts.insert(1, "run-script".to_string());
    vec![parts.join(" ")]
}

pub const RULE: Rule = Rule { enabled_by_default: npm_available, ..Rule::new("npm_run_script", match_, get_new_command) };
