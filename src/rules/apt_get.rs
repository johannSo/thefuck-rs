use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;

use super::prelude::*;

/// The helper Debian and Ubuntu use for the hint shown by the shell when a
/// command isn't found.
const COMMAND_NOT_FOUND: &str = "/usr/lib/command-not-found";

fn enabled_by_default() -> bool {
    apt_available() && Path::new(COMMAND_NOT_FOUND).exists()
}

fn get_executable(command: &Command) -> &str {
    let parts = command.script_parts();
    if parts[0] == "sudo" {
        &parts[1]
    } else {
        &parts[0]
    }
}

fn parse_package(advice: &str) -> Option<String> {
    find_first(r"apt(?:-get)?\s+install\s+([^\s]+)", advice)
}

/// Returns the name of the package providing the executable.
fn get_package(executable: &str) -> Option<String> {
    static CACHE: Mutex<Option<HashMap<String, Option<String>>>> = Mutex::new(None);
    let mut cache = CACHE.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    cache
        .get_or_insert_with(HashMap::new)
        .entry(executable.to_string())
        .or_insert_with(|| {
            let advice = run(COMMAND_NOT_FOUND, &["--", executable]);
            parse_package(&advice.stderr).or_else(|| parse_package(&advice.stdout))
        })
        .clone()
}

fn match_(command: &Command) -> bool {
    if command.output().contains("not found") || command.output().contains("not installed") {
        let executable = get_executable(command);
        which(executable).is_none() && get_package(executable).is_some()
    } else {
        false
    }
}

fn get_new_command(command: &Command) -> Vec<String> {
    let executable = get_executable(command);
    get_package(executable)
        .map(|name| and_(&[&format!("sudo apt-get install {}", name), &command.script]))
        .into_iter()
        .collect()
}

pub const RULE: Rule = Rule { enabled_by_default, ..Rule::new("apt_get", match_, get_new_command) };

#[cfg(test)]
mod tests {
    use super::parse_package;

    #[test]
    fn parses_advice() {
        let advice = "\nCommand 'vim' not found, but can be installed with:\n\nsudo apt install vim         # version 2:8.2.3995-1ubuntu2\n";
        assert_eq!(parse_package(advice).unwrap(), "vim");
        let advice = "Command 'convert' not found, but can be installed with:\nsudo snap install imagemagick  # version 7\nsudo apt  install imagemagick-6.q16  # version 8\n";
        assert_eq!(parse_package(advice).unwrap(), "imagemagick-6.q16");
        assert_eq!(parse_package("vom: command not found"), None);
    }
}
