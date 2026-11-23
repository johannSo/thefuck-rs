use super::prelude::*;

fn match_(command: &Command) -> bool {
    sudo_support(command, |command| {
        for_app(command, &["apt", "apt-get", "apt-cache"]) && command.output().contains("E: Invalid operation")
    })
}

fn parse_apt_operations(help_text_lines: &[&str]) -> Vec<String> {
    let mut is_commands_list = false;
    let mut operations = Vec::new();
    for line in help_text_lines {
        let line = line.trim();
        if is_commands_list && !line.is_empty() {
            operations.extend(line.split_whitespace().next().map(String::from));
        } else if line.starts_with("Basic commands:") || line.starts_with("Most used commands:") {
            is_commands_list = true;
        }
    }
    operations
}

fn parse_apt_get_and_cache_operations(help_text_lines: &[&str]) -> Vec<String> {
    let mut is_commands_list = false;
    let mut operations = Vec::new();
    for line in help_text_lines {
        let line = line.trim();
        if is_commands_list {
            if line.is_empty() {
                break;
            }
            operations.extend(line.split_whitespace().next().map(String::from));
        } else if line.starts_with("Commands:") || line.starts_with("Most used commands:") {
            is_commands_list = true;
        }
    }
    operations
}

fn get_operations(app: &str) -> Vec<String> {
    let help = run(app, &["--help"]);
    if app == "apt" {
        parse_apt_operations(&help.stdout_lines())
    } else {
        parse_apt_get_and_cache_operations(&help.stdout_lines())
    }
}

fn get_new_command(command: &Command) -> Vec<String> {
    sudo_support(command, |command| {
        let invalid_operation = command.output().split_whitespace().next_back().unwrap_or("");

        if invalid_operation == "uninstall" {
            vec![command.script.replace("uninstall", "remove")]
        } else {
            let operations = get_operations(&command.script_parts()[0]);
            replace_command(command, invalid_operation, &operations)
        }
    })
}

pub const RULE: Rule =
    Rule { enabled_by_default: apt_available, ..Rule::new("apt_invalid_operation", match_, get_new_command) };

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_apt_help() {
        let help = "apt 1.0.10.2ubuntu1 for amd64 compiled on Oct  5 2015 15:55:05\nUsage: apt [options] command\n\nCLI for apt.\nBasic commands: \n list - list packages based on package names\n search - search in package descriptions\n show - show package details\n\n update - update list of available packages\n";
        assert_eq!(parse_apt_operations(&help.lines().collect::<Vec<_>>()), ["list", "search", "show", "update"]);
    }

    #[test]
    fn parses_apt_get_help() {
        let help = "apt 1.6.12 (amd64)\nUsage: apt-get [options] command\n\nMost used commands:\n  update - Retrieve new lists of packages\n  upgrade - Perform an upgrade\n\nSee apt-get(8) for more information about the available commands.\n";
        assert_eq!(parse_apt_get_and_cache_operations(&help.lines().collect::<Vec<_>>()), ["update", "upgrade"]);
    }
}
