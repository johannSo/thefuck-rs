use super::prelude::*;

fn match_(command: &Command) -> bool {
    for_app(command, &["ifconfig"])
        && command.output().contains("error fetching interface information: Device not found")
}

fn parse_interfaces(output: &str) -> Vec<String> {
    output
        .split_inclusive('\n')
        .filter(|line| *line != "\n" && !line.starts_with(' '))
        .map(|line| line.split(' ').next().unwrap_or("").to_string())
        .collect()
}

fn get_possible_interfaces() -> Vec<String> {
    parse_interfaces(&run("ifconfig", &["-a"]).stdout)
}

fn get_new_command(command: &Command) -> Vec<String> {
    let interface = command.output().split(' ').next().unwrap_or("");
    // without the colon
    let interface: String = interface.chars().take(interface.chars().count().saturating_sub(1)).collect();
    let possible_interfaces = get_possible_interfaces();
    replace_command(command, &interface, &possible_interfaces)
}

pub const RULE: Rule = Rule::new("ifconfig_device_not_found", match_, get_new_command);

#[cfg(test)]
mod tests {
    use super::parse_interfaces;

    #[test]
    fn parses_interfaces() {
        let output = "wlan0     Link encap:Ethernet  HWaddr 00:00:00:00:00:00  \n          inet addr:192.168.1.1  Bcast:192.168.1.255  Mask:255.255.255.0\n\nlo        Link encap:Local Loopback  \n          inet addr:127.0.0.1  Mask:255.0.0.0\n";
        assert_eq!(parse_interfaces(output), ["wlan0", "lo"]);
    }
}
