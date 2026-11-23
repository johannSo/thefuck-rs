use std::fs;
use std::io;
use std::path::Path;

use super::prelude::*;

const BREW_CMD_PATH: &str = "/Homebrew/Library/Homebrew/cmd";
const TAP_PATH: &str = "/Homebrew/Library/Taps";

fn listdir(path: &str) -> io::Result<Vec<String>> {
    fs::read_dir(path)?.map(|entry| Ok(entry?.file_name().to_string_lossy().into_owned())).collect()
}

/// To get brew default commands on local environment
fn get_brew_commands(brew_path_prefix: &str) -> io::Result<Vec<String>> {
    let brew_cmd_path = format!("{}{}", brew_path_prefix, BREW_CMD_PATH);
    Ok(listdir(&brew_cmd_path)?
        .iter()
        .filter(|name| name.ends_with(".rb") || name.ends_with(".sh"))
        .map(|name| name[..name.len() - 3].to_string())
        .collect())
}

fn get_directory_names_only(path: &str) -> io::Result<Vec<String>> {
    Ok(listdir(path)?.into_iter().filter(|name| Path::new(path).join(name).is_dir()).collect())
}

fn is_brew_tap_cmd_naming(name: &str) -> bool {
    name.starts_with("brew-") && name.ends_with(".rb")
}

/// To get tap's specific commands
/// https://github.com/Homebrew/homebrew/blob/master/Library/brew.rb#L115
fn get_brew_tap_specific_commands(brew_path_prefix: &str) -> io::Result<Vec<String>> {
    let mut commands = Vec::new();
    let brew_taps_path = format!("{}{}", brew_path_prefix, TAP_PATH);

    for user in get_directory_names_only(&brew_taps_path)? {
        let taps = get_directory_names_only(&format!("{}/{}", brew_taps_path, user))?;

        // Brew Taps's naming rule
        // https://github.com/Homebrew/homebrew/blob/master/share/doc/homebrew/brew-tap.md#naming-conventions-and-limitations
        for tap in taps.iter().filter(|tap| tap.starts_with("homebrew-")) {
            let tap_cmd_path = format!("{}/{}/{}/cmd", brew_taps_path, user, tap);

            if Path::new(&tap_cmd_path).is_dir() {
                commands.extend(
                    listdir(&tap_cmd_path)?
                        .iter()
                        .filter(|name| is_brew_tap_cmd_naming(name))
                        .map(|name| name.replace("brew-", "").replace(".rb", "")),
                );
            }
        }
    }

    Ok(commands)
}

fn brew_commands() -> Vec<String> {
    if let Some(brew_path_prefix) = get_brew_path_prefix() {
        let commands = get_brew_commands(brew_path_prefix).and_then(|mut commands| {
            commands.extend(get_brew_tap_specific_commands(brew_path_prefix)?);
            Ok(commands)
        });
        if let Ok(commands) = commands {
            return commands;
        }
    }

    // Failback commands for testing (Based on Homebrew 0.9.5)
    strings(&[
        "info", "home", "options", "install", "uninstall", "search", "list", "update", "upgrade", "pin",
        "unpin", "doctor", "create", "edit", "cask",
    ])
}

fn broken_command(command: &Command) -> String {
    findall(r"Error: Unknown command: ([a-z]+)", command.output())[0].clone()
}

fn match_(command: &Command) -> bool {
    let is_proper_command = command.script.contains("brew") && command.output().contains("Unknown command");

    if is_proper_command {
        let broken_cmd = broken_command(command);
        return get_closest(&broken_cmd, &brew_commands(), 0.6, true).is_some_and(|closest| !closest.is_empty());
    }
    false
}

fn get_new_command(command: &Command) -> Vec<String> {
    let broken_cmd = broken_command(command);
    replace_command(command, &broken_cmd, &brew_commands())
}

pub const RULE: Rule =
    Rule { enabled_by_default: brew_available, ..Rule::new("brew_unknown_command", match_, get_new_command) };
