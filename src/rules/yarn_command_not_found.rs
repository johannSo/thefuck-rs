use super::prelude::*;

const REGEX: &str = r#"error Command "(.*)" not found."#;

fn match_(command: &Command) -> bool {
    for_app(command, &["yarn"]) && regex(REGEX).is_match(command.output())
}

fn npm_command(task: &str) -> Option<&'static str> {
    match task {
        "require" => Some("add"),
        _ => None,
    }
}

fn parse_tasks(help: &str) -> Vec<String> {
    let mut should_yield = false;
    let mut tasks = Vec::new();
    for line in help.split_inclusive('\n') {
        let line = line.trim();

        if line.contains("Commands:") {
            should_yield = true;
            continue;
        }

        if should_yield && line.contains("- ") {
            tasks.push(line.split(' ').next_back().unwrap_or("").to_string());
        }
    }
    tasks
}

fn get_all_tasks() -> Vec<String> {
    cache_for_app("yarn", "yarn_command_not_found.get_all_tasks", || parse_tasks(&run("yarn", &["--help"]).stdout))
}

fn get_new_command(command: &Command) -> Vec<String> {
    let misspelled_task = &findall(REGEX, command.output())[0];
    match npm_command(misspelled_task) {
        Some(yarn_command) => vec![replace_argument(&command.script, misspelled_task, yarn_command)],
        None => replace_command(command, misspelled_task, &get_all_tasks()),
    }
}

pub const RULE: Rule = Rule::new("yarn_command_not_found", match_, get_new_command);

#[cfg(test)]
mod tests {
    use super::parse_tasks;

    #[test]
    fn parses_tasks() {
        let help = "\n  Usage: yarn [command] [flags]\n\n  Options:\n\n    -h, --help                      output usage information\n\n  Commands:\n\n    - access\n    - add\n    - bin\n\n  Run `yarn help COMMAND` for more information on specific commands.\n";
        assert_eq!(parse_tasks(help), ["access", "add", "bin"]);
    }
}
