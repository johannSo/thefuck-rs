use super::prelude::*;

fn match_(command: &Command) -> bool {
    sudo_support(command, |command| {
        for_app(command, &["docker"])
            && (command.output().contains("is not a docker command") || command.output().contains("Usage:\tdocker"))
    })
}

fn parse_commands(lines: &[&str], starts_with: &str) -> Vec<String> {
    lines
        .iter()
        .skip_while(|line| !line.starts_with(starts_with))
        .skip(1)
        .take_while(|line| !line.trim().is_empty())
        .map(|line| line.trim().split(' ').next().unwrap_or("").to_string())
        .collect()
}

fn get_docker_commands() -> Vec<String> {
    cache_for_app("docker", "docker_not_command.get_docker_commands", || {
        let help = run("docker", &[]);
        // Old version docker returns its output to stdout, while newer version returns to stderr.
        let lines = if help.stdout.is_empty() { help.stderr_lines() } else { help.stdout_lines() };

        // Only newer versions of docker have management commands in the help text.
        let mut commands = if lines.contains(&"Management Commands:\n") {
            parse_commands(&lines, "Management Commands:")
        } else {
            vec![]
        };
        commands.extend(parse_commands(&lines, "Commands:"));
        commands
    })
}

fn get_new_command(command: &Command) -> Vec<String> {
    sudo_support(command, |command| {
        if command.output().contains("Usage:") && command.script_parts().len() > 1 {
            let lines: Vec<&str> = command.output().split('\n').collect();
            let management_subcommands = parse_commands(&lines, "Commands:");
            return replace_command(command, &command.script_parts()[2], &management_subcommands);
        }

        let wrong_command = &findall(r"docker: '(\w+)' is not a docker command.", command.output())[0];
        replace_command(command, wrong_command, &get_docker_commands())
    })
}

pub const RULE: Rule = Rule::new("docker_not_command", match_, get_new_command);

#[cfg(test)]
mod tests {
    use super::parse_commands;

    #[test]
    fn parses_commands() {
        let help = "Usage:\tdocker [OPTIONS] COMMAND\n\nManagement Commands:\n  config      Manage Docker configs\n  container   Manage containers\n\nCommands:\n  attach      Attach local standard input\n  build       Build an image from a Dockerfile\n\nRun 'docker COMMAND --help' for more information on a command.\n";
        let lines: Vec<&str> = help.split_inclusive('\n').collect();
        assert_eq!(parse_commands(&lines, "Management Commands:"), ["config", "container"]);
        assert_eq!(parse_commands(&lines, "Commands:"), ["attach", "build"]);
        assert!(parse_commands(&lines, "Missing:").is_empty());
    }
}
