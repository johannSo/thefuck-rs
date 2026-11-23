use super::prelude::*;

fn match_(command: &Command) -> bool {
    sudo_support(command, |command| for_app(command, &["yum"]) && command.output().contains("No such command: "))
}

fn parse_operations(help: &str) -> Vec<String> {
    help.split_inclusive('\n')
        .skip_while(|line| !line.starts_with("List of Commands:"))
        .skip(2)
        .take_while(|line| !line.trim().is_empty())
        .map(|line| line.trim().split(' ').next().unwrap_or("").to_string())
        .collect()
}

fn get_operations() -> Vec<String> {
    cache_for_app("yum", "yum_invalid_operation.get_operations", || parse_operations(&run("yum", &[]).stdout))
}

fn get_new_command(command: &Command) -> Vec<String> {
    sudo_support(command, |command| {
        let invalid_operation = &command.script_parts()[1];

        if invalid_operation == "uninstall" {
            return vec![command.script.replace("uninstall", "remove")];
        }

        replace_command(command, invalid_operation, &get_operations())
    })
}

pub const RULE: Rule =
    Rule { enabled_by_default: yum_available, ..Rule::new("yum_invalid_operation", match_, get_new_command) };

#[cfg(test)]
mod tests {
    use super::parse_operations;

    #[test]
    fn parses_operations() {
        let help = "Loaded plugins: extras_suggestions, langpacks, priorities, update-motd\nYou need to give some command\nUsage: yum [options] COMMAND\n\nList of Commands:\n\ncheck          Check for problems in the rpmdb\ncheck-update   Check for available package updates\nclean          Remove cached data\n\n\nOptions:\n  -h, --help            show this help message and exit\n";
        assert_eq!(parse_operations(help), ["check", "check-update", "clean"]);
    }
}
