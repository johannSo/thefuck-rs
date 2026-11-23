use super::prelude::*;

fn match_(command: &Command) -> bool {
    for_app(command, &["react-native"]) && regex(r"Unrecognized command '.*'").is_match(command.output())
}

fn parse_commands(help: &str) -> Vec<String> {
    let mut should_yield = false;
    let mut commands = Vec::new();
    for line in help.split_inclusive('\n') {
        let line = line.trim();

        if line.is_empty() {
            continue;
        }

        if line.contains("Commands:") {
            should_yield = true;
            continue;
        }

        if should_yield {
            commands.push(line.split(' ').next().unwrap_or("").to_string());
        }
    }
    commands
}

fn get_commands() -> Vec<String> {
    cache_strings("react_native_command_unrecognized.get_commands", &["package.json"], || {
        parse_commands(&run("react-native", &["--help"]).stdout)
    })
}

fn get_new_command(command: &Command) -> Vec<String> {
    let misspelled_command = &findall(r"Unrecognized command '(.*)'", command.output())[0];
    let commands = get_commands();
    replace_command(command, misspelled_command, &commands)
}

pub const RULE: Rule = Rule::new("react_native_command_unrecognized", match_, get_new_command);

#[cfg(test)]
mod tests {
    use super::parse_commands;

    #[test]
    fn parses_commands() {
        let help = "Scanning 615 folders for symlinks in /home/nvbn/work/zcho/BookkaWebView/node_modules (6ms)\n\n  Usage: react-native [options] [command]\n\n  Options:\n\n    -V, --version  output the version number\n\n  Commands:\n\n    start [options]                    starts the webserver\n    run-ios [options]                  builds your app and starts it on iOS simulator\n    log-android [options]              starts adb logcat\n";
        assert_eq!(parse_commands(help), ["start", "run-ios", "log-android"]);
    }
}
