use super::prelude::*;

fn match_(command: &Command) -> bool {
    for_app(command, &["gem"])
        && command.output().contains("ERROR:  While executing gem ... (Gem::CommandLineError)")
        && command.output().contains("Unknown command")
}

fn get_unknown_command(command: &Command) -> String {
    findall(r"Unknown command (.*)\n?\z", command.output())[0].clone()
}

fn parse_commands(help: &str) -> Vec<String> {
    help.split_inclusive('\n')
        .filter(|line| line.starts_with("    "))
        .map(|line| line.trim().split(' ').next().unwrap_or("").to_string())
        .collect()
}

fn get_all_commands() -> Vec<String> {
    cache_for_app("gem", "gem_unknown_command.get_all_commands", || {
        parse_commands(&run("gem", &["help", "commands"]).stdout)
    })
}

fn get_new_command(command: &Command) -> Vec<String> {
    let unknown_command = get_unknown_command(command);
    let all_commands = get_all_commands();
    replace_command(command, &unknown_command, &all_commands)
}

pub const RULE: Rule = Rule::new("gem_unknown_command", match_, get_new_command);

#[cfg(test)]
mod tests {
    use super::parse_commands;

    #[test]
    fn parses_commands() {
        let help = "GEM commands are:\n\n    build             Build a gem from a gemspec\n    cert              Manage RubyGems certificates and signing settings\n    cleanup           Clean up old versions of installed gems\n\nFor help on a particular command, use 'gem help COMMAND'.\n";
        assert_eq!(parse_commands(help), ["build", "cert", "cleanup"]);
    }
}
