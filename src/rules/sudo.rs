use super::prelude::*;

const PATTERNS: &[&str] = &[
    "permission denied",
    "eacces",
    "pkg: insufficient privileges",
    "you cannot perform this operation unless you are root",
    "non-root users cannot",
    "operation not permitted",
    "not super-user",
    "superuser privilege",
    "root privilege",
    "this command has to be run under the root user.",
    "this operation requires root.",
    "requested operation requires superuser privilege",
    "must be run as root",
    "must run as root",
    "must be superuser",
    "must be root",
    "need to be root",
    "need root",
    "needs to be run as root",
    "only root can ",
    "you don't have access to the history db.",
    "authentication is required",
    "edspermissionerror",
    "you don't have write permissions",
    "use `sudo`",
    "sudorequirederror",
    "error: insufficient privileges",
    "updatedb: can not open a temporary file",
];

fn match_(command: &Command) -> bool {
    let parts = command.script_parts();
    if !parts.is_empty() && !parts.iter().any(|part| part == "&&") && parts[0] == "sudo" {
        return false;
    }

    let output = command.output().to_lowercase();
    PATTERNS.iter().any(|pattern| output.contains(pattern))
}

fn get_new_command(command: &Command) -> Vec<String> {
    if command.script.contains("&&") {
        let script: Vec<&str> =
            command.script_parts().iter().filter(|part| *part != "sudo").map(String::as_str).collect();
        vec![format!("sudo sh -c \"{}\"", script.join(" "))]
    } else if command.script.contains('>') {
        vec![format!("sudo sh -c \"{}\"", command.script.replace('"', "\\\""))]
    } else {
        vec![format!("sudo {}", command.script)]
    }
}

pub const RULE: Rule = Rule::new("sudo", match_, get_new_command);
