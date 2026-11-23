use super::prelude::*;

fn parse_commands(help: &str) -> Vec<String> {
    help.split_inclusive('\n')
        .map(str::trim)
        .skip_while(|line| *line != "The commands are:")
        .skip(2)
        .take_while(|line| !line.is_empty())
        .map(|line| line.split(' ').next().unwrap_or("").to_string())
        .collect()
}

fn get_golang_commands() -> Vec<String> {
    cache_for_app("go", "go_unknown_command.get_golang_commands", || parse_commands(&run("go", &[]).stderr))
}

fn match_(command: &Command) -> bool {
    for_app(command, &["go"]) && command.output().contains("unknown command")
}

fn get_new_command(command: &Command) -> Vec<String> {
    let broken = &command.script_parts()[1];
    get_closest(broken, &get_golang_commands(), 0.6, true)
        .map(|closest_subcommand| replace_argument(&command.script, broken, &closest_subcommand))
        .into_iter()
        .collect()
}

pub const RULE: Rule = Rule::new("go_unknown_command", match_, get_new_command);

#[cfg(test)]
mod tests {
    use super::parse_commands;

    #[test]
    fn parses_commands() {
        let help = "Go is a tool for managing Go source code.\n\nUsage:\n\n\tgo <command> [arguments]\n\nThe commands are:\n\n\tbug         start a bug report\n\tbuild       compile packages and dependencies\n\tclean       remove object files and cached files\n\nUse \"go help <command>\" for more information about a command.\n";
        assert_eq!(parse_commands(help), ["bug", "build", "clean"]);
    }
}
