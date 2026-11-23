use std::fs;

use super::prelude::*;

const COMMANDS: &[&str] = &["ssh", "scp"];

fn match_(command: &Command) -> bool {
    if !for_app(command, COMMANDS) {
        return false;
    }
    if !COMMANDS.iter().any(|app| command.script.starts_with(app)) {
        return false;
    }

    let patterns = [
        r"WARNING: REMOTE HOST IDENTIFICATION HAS CHANGED!",
        r"WARNING: POSSIBLE DNS SPOOFING DETECTED!",
        r"Warning: the \S+ host key for '([^']+)' differs from the key for the IP address '([^']+)'",
    ];

    patterns.iter().any(|pattern| regex(pattern).is_match(command.output()))
}

fn get_new_command(command: &Command) -> Vec<String> {
    vec![command.script.clone()]
}

/// Returns the files and the numbers of the lines with outdated host keys.
fn offending_lines(output: &str) -> Vec<(String, usize)> {
    regex(r"(?:Offending (?:key for IP|\S+ key)|Matching host key) in ([^:]+):(\d+)")
        .captures_iter(output)
        .filter_map(|offending| Some((offending[1].to_string(), offending[2].parse().ok()?)))
        .collect()
}

/// Removes the outdated host keys.
fn side_effect(old_cmd: &Command, _command: &str) {
    for (filepath, lineno) in offending_lines(old_cmd.output()) {
        let Ok(content) = fs::read_to_string(&filepath) else { continue };
        let mut lines: Vec<&str> = content.split_inclusive('\n').collect();
        if lineno == 0 || lineno > lines.len() {
            continue;
        }
        lines.remove(lineno - 1);
        let _ = fs::write(&filepath, lines.concat());
    }
}

pub const RULE: Rule = Rule { side_effect: Some(side_effect), ..Rule::new("ssh_known_hosts", match_, get_new_command) };

#[cfg(test)]
mod tests {
    use super::*;

    const OUTPUT: &str = "@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@\n@    WARNING: REMOTE HOST IDENTIFICATION HAS CHANGED!     @\n@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@\nIT IS POSSIBLE THAT SOMEONE IS DOING SOMETHING NASTY!\nAdd correct host key in KNOWN_HOSTS to get rid of this message.\nOffending RSA key in KNOWN_HOSTS:2\nRSA host key for localhost has changed and you have requested strict checking.\nHost key verification failed.";

    #[test]
    fn removes_offending_keys() {
        let path = std::env::temp_dir().join(format!("thefuck-test-known-hosts-{}", std::process::id()));
        fs::write(&path, "123.234.567.890 asdjkasjdakjsd\n98.765.432.321 ejioweojwejrosj\n111.222.333.444 qwepoiwqepoiss\n")
            .unwrap();
        let output = OUTPUT.replace("KNOWN_HOSTS", path.to_str().unwrap());
        assert_eq!(offending_lines(&output), [(path.to_str().unwrap().to_string(), 2)]);

        let command = Command::new("ssh localhost", Some(&output));
        assert!(match_(&command));
        side_effect(&command, "ssh localhost");
        let content = fs::read_to_string(&path).unwrap();
        fs::remove_file(&path).unwrap();
        assert_eq!(content, "123.234.567.890 asdjkasjdakjsd\n111.222.333.444 qwepoiwqepoiss\n");
    }
}
