use super::prelude::*;

fn enabled_by_default() -> bool {
    which("lsof").is_some()
}

const PATTERNS: &[&str] = &[
    r"bind on address \('.*', (?P<port>\d+)\)",
    r"Unable to bind [^ ]*:(?P<port>\d+)",
    r"can't listen on port (?P<port>\d+)",
    r"listen EADDRINUSE [^ ]*:(?P<port>\d+)",
];

fn parse_pid(lsof_output: &str) -> Option<String> {
    lsof_output.split('\n').nth(1)?.split_whitespace().nth(1).map(String::from)
}

fn get_pid_by_port(port: &str) -> Option<String> {
    parse_pid(&run("lsof", &["-i", &format!(":{}", port)]).stdout)
}

fn get_used_port(command: &Command) -> Option<String> {
    PATTERNS
        .iter()
        .find_map(|pattern| regex(pattern).captures(command.output()).map(|matched| matched["port"].to_string()))
}

fn match_(command: &Command) -> bool {
    get_used_port(command).is_some_and(|port| get_pid_by_port(&port).is_some())
}

fn get_new_command(command: &Command) -> Vec<String> {
    get_used_port(command)
        .and_then(|port| get_pid_by_port(&port))
        .map(|pid| and_(&[&format!("kill {}", pid), &command.script]))
        .into_iter()
        .collect()
}

pub const RULE: Rule = Rule { enabled_by_default, ..Rule::new("port_already_in_use", match_, get_new_command) };

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_port_and_pid() {
        let lsof = "COMMAND   PID USER   FD   TYPE DEVICE SIZE/OFF NODE NAME\nnode    18233 nvbn   16u  IPv4 557134      0t0  TCP localhost:http-alt (LISTEN)\n";
        assert_eq!(parse_pid(lsof).unwrap(), "18233");
        assert_eq!(parse_pid(""), None);
        assert_eq!(parse_pid("COMMAND   PID USER\n"), None);

        let port = |output: &str| get_used_port(&Command::new("./app", Some(output)));
        assert_eq!(port("OSError: bind on address ('127.0.0.1', 8080): Address in use").unwrap(), "8080");
        assert_eq!(port("Unable to bind localhost:8080").unwrap(), "8080");
        assert_eq!(port("can't listen on port 8080, please use another port").unwrap(), "8080");
        assert_eq!(port("Error: listen EADDRINUSE 127.0.0.1:8080").unwrap(), "8080");
        assert_eq!(port("something else"), None);
    }
}
