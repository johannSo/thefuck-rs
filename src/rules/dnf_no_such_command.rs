use super::prelude::*;

fn match_(command: &Command) -> bool {
    sudo_support(command, |command| {
        for_app(command, &["dnf"]) && command.output().to_lowercase().contains("no such command")
    })
}

fn parse_operations(help_text_lines: &str) -> Vec<String> {
    findall(r"(?m)^([a-z-]+) +", help_text_lines)
}

fn get_operations() -> Vec<String> {
    parse_operations(&run("dnf", &["--help"]).stdout)
}

fn get_new_command(command: &Command) -> Vec<String> {
    sudo_support(command, |command| {
        let misspelled_command = &findall(r"No such command: (.*)\.", command.output())[0];
        replace_command(command, misspelled_command, &get_operations())
    })
}

pub const RULE: Rule =
    Rule { enabled_by_default: dnf_available, ..Rule::new("dnf_no_such_command", match_, get_new_command) };
