use super::prelude::*;

fn match_(command: &Command) -> bool {
    for_app(command, &["choco", "cinst"])
        && (command.script.starts_with("choco install") || command.script_parts().iter().any(|part| part == "cinst"))
        && command.output().contains("Installing the following packages")
}

fn get_new_command(command: &Command) -> Vec<String> {
    // Find the argument that is the package name
    for script_part in command.script_parts() {
        if !["choco", "cinst", "install"].contains(&script_part.as_str())
            // Need exact match (bc chocolatey is a package)
            && !script_part.starts_with('-')
            // Leading hyphens are parameters; some packages contain them though
            && !script_part.contains('=')
            && !script_part.contains('/')
        // These are certainly parameters
        {
            return vec![command.script.replace(script_part.as_str(), &format!("{}.install", script_part))];
        }
    }
    vec![]
}

fn enabled_by_default() -> bool {
    which("choco").is_some() || which("cinst").is_some()
}

pub const RULE: Rule = Rule { enabled_by_default, ..Rule::new("choco_install", match_, get_new_command) };
