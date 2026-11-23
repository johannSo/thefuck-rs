use super::prelude::*;

const ADB_COMMANDS: &[&str] = &[
    "backup",
    "bugreport",
    "connect",
    "devices",
    "disable-verity",
    "disconnect",
    "enable-verity",
    "emu",
    "forward",
    "get-devpath",
    "get-serialno",
    "get-state",
    "install",
    "install-multiple",
    "jdwp",
    "keygen",
    "kill-server",
    "logcat",
    "pull",
    "push",
    "reboot",
    "reconnect",
    "restore",
    "reverse",
    "root",
    "run-as",
    "shell",
    "sideload",
    "start-server",
    "sync",
    "tcpip",
    "uninstall",
    "unroot",
    "usb",
    "wait-for",
];

fn match_(command: &Command) -> bool {
    for_app(command, &["adb"]) && command.output().starts_with("Android Debug Bridge version")
}

fn get_new_command(command: &Command) -> Vec<String> {
    let parts = command.script_parts();
    for (idx, arg) in parts[1..].iter().enumerate() {
        // allowed params to ADB are a/d/e/s/H/P/L where s, H, P and L take additional args
        // for example 'adb -s 111 logcat' or 'adb -e logcat'
        let previous = parts[idx].as_str();
        if !arg.starts_with('-') && !["-s", "-H", "-P", "-L"].contains(&previous) {
            return get_closest(arg, ADB_COMMANDS, 0.6, true)
                .map(|adb_cmd| replace_argument(&command.script, arg, &adb_cmd))
                .into_iter()
                .collect();
        }
    }
    vec![]
}

pub const RULE: Rule = Rule::new("adb_unknown_command", match_, get_new_command);
