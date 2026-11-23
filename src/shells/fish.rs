use std::collections::{BTreeSet, HashMap};
use std::fs::{self, OpenOptions};
use std::io::Write;

use serde_json::json;

use super::{unix_time, version_output, Shell, ShellConfiguration};
use crate::conf::settings;
use crate::consts::ARGUMENT_PLACEHOLDER;
use crate::logs;
use crate::utils::{cache, expanduser, run};

pub struct Fish;

fn parse_functions(output: &str, overridden: &[String]) -> HashMap<String, String> {
    output
        .trim()
        .split('\n')
        .filter(|func| !overridden.iter().any(|name| name == func))
        .map(|func| (func.to_string(), func.to_string()))
        .collect()
}

fn parse_aliases(output: &str, overridden: &[String]) -> HashMap<String, String> {
    let mut aliases = HashMap::new();
    let alias_out = output.trim();
    if alias_out.is_empty() {
        return aliases;
    }
    for alias in alias_out.split('\n') {
        let alias = alias.replacen("alias ", "", 1);
        let Some((name, value)) = alias.split_once(' ').or_else(|| alias.split_once('=')) else {
            continue;
        };
        if !overridden.iter().any(|overridden| overridden == name) {
            aliases.insert(name.to_string(), value.to_string());
        }
    }
    aliases
}

fn get_overridden_aliases() -> Vec<String> {
    let overridden = std::env::var("THEFUCK_OVERRIDDEN_ALIASES")
        .or_else(|_| std::env::var("TF_OVERRIDDEN_ALIASES"))
        .unwrap_or_default();
    let mut default: BTreeSet<String> = ["cd", "grep", "ls", "man", "open"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    for alias in overridden.split(',') {
        default.insert(alias.trim().to_string());
    }
    default.into_iter().collect()
}

fn expand_aliases(aliases: &HashMap<String, String>, command_script: &str) -> String {
    let binary = command_script.split(' ').next().unwrap_or("");
    match aliases.get(binary) {
        Some(expansion) if expansion != binary => command_script.replacen(binary, expansion, 1),
        Some(_) => format!("fish -ic \"{}\"", command_script.replace('"', "\\\"")),
        None => command_script.to_string(),
    }
}

impl Shell for Fish {
    fn friendly_name(&self) -> &'static str {
        "Fish Shell"
    }

    fn app_alias(&self, alias_name: &str) -> String {
        let alter_history = if settings().alter_history {
            "    builtin history delete --exact --case-sensitive -- $fucked_up_command\n    builtin history merge\n"
        } else {
            ""
        };
        // It is VERY important to have the variables declared WITHIN the alias
        format!(
            "function {0} -d \"Correct your previous console command\"\n\
             \x20 set -l fucked_up_command $history[1]\n\
             \x20 env TF_SHELL=fish TF_ALIAS={0} thefuck $fucked_up_command {2} $argv | read -l unfucked_command\n\
             \x20 if [ \"$unfucked_command\" != \"\" ]\n\
             \x20   eval $unfucked_command\n{1}\
             \x20 end\n\
             end",
            alias_name, alter_history, ARGUMENT_PLACEHOLDER
        )
    }

    fn get_aliases(&self) -> HashMap<String, String> {
        let overridden = get_overridden_aliases();
        let key = |kind: &str| format!("fish.{}:{}", kind, overridden.join(","));
        let functions = cache(
            &key("functions"),
            &["~/.config/fish/config.fish", "~/.config/fish/functions"],
            || {
                json!(parse_functions(
                    &run("fish", &["-ic", "functions"]).stdout,
                    &overridden
                ))
            },
        );
        let raw_aliases = cache(&key("aliases"), &["~/.config/fish/config.fish"], || {
            json!(parse_aliases(
                &run("fish", &["-ic", "alias"]).stdout,
                &overridden
            ))
        });
        let mut aliases: HashMap<String, String> =
            serde_json::from_value(functions).unwrap_or_default();
        aliases.extend(
            serde_json::from_value::<HashMap<String, String>>(raw_aliases).unwrap_or_default(),
        );
        aliases
    }

    fn expand_aliases(&self, command_script: &str) -> String {
        expand_aliases(&self.get_aliases(), command_script)
    }

    fn history_file_name(&self) -> String {
        expanduser("~/.config/fish/fish_history")
            .to_string_lossy()
            .into_owned()
    }

    fn history_line(&self, command_script: &str) -> String {
        format!("- cmd: {}\n   when: {}\n", command_script, unix_time())
    }

    fn script_from_history(&self, line: &str) -> String {
        line.split_once("- cmd: ")
            .map_or("", |(_, script)| script)
            .to_string()
    }

    fn and_(&self, commands: &[&str]) -> String {
        commands.join("; and ")
    }

    fn or_(&self, commands: &[&str]) -> String {
        commands.join("; or ")
    }

    fn how_to_configure(&self) -> Option<ShellConfiguration> {
        Some(ShellConfiguration::created_if_missing(
            "thefuck --alias | source",
            "~/.config/fish/config.fish",
            "fish",
        ))
    }

    fn version(&self) -> String {
        version_output("fish", &["--version"])
            .split_whitespace()
            .last()
            .unwrap_or("")
            .to_string()
    }

    /// Puts command script to shell history.
    fn put_to_history(&self, command_script: &str) {
        let history_file_name = self.history_file_name();
        if !fs::metadata(&history_file_name).is_ok_and(|metadata| metadata.is_file()) {
            return;
        }
        let appended = OpenOptions::new()
            .append(true)
            .open(&history_file_name)
            .and_then(|mut history| {
                history.write_all(self.history_line(command_script).as_bytes())
            });
        if let Err(error) = appended {
            logs::exception("Can't update history", &error.to_string());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn overridden() -> Vec<String> {
        ["cd", "grep", "ls", "man", "open"]
            .iter()
            .map(|s| s.to_string())
            .collect()
    }

    #[test]
    fn parses_functions_and_aliases() {
        let functions = parse_functions(
            "cd\nfish_config\nfuck\nfunced\ngrep\nls\nman\n",
            &overridden(),
        );
        let mut names: Vec<_> = functions.keys().cloned().collect();
        names.sort();
        assert_eq!(names, ["fish_config", "fuck", "funced"]);

        let aliases = parse_aliases(
            "alias fish_key_reader /usr/bin/fish_key_reader\nalias g git\nalias alias_with_equal_sign=echo\ninvalid_alias\nalias ls 'ls -G'\n",
            &overridden(),
        );
        assert_eq!(aliases.len(), 3);
        assert_eq!(aliases["g"], "git");
        assert_eq!(aliases["alias_with_equal_sign"], "echo");
        assert!(parse_aliases("", &overridden()).is_empty());
    }

    #[test]
    fn expands_aliases() {
        let aliases: HashMap<String, String> =
            [("g", "git"), ("fuck", "fuck"), ("funced", "funced")]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect();
        assert_eq!(expand_aliases(&aliases, "pwd"), "pwd");
        assert_eq!(expand_aliases(&aliases, "g st"), "git st");
        assert_eq!(expand_aliases(&aliases, "funced"), "fish -ic \"funced\"");
        assert_eq!(
            expand_aliases(&aliases, "funced \"x\""),
            "fish -ic \"funced \\\"x\\\"\""
        );
    }

    #[test]
    fn joins_commands_and_reads_history() {
        assert_eq!(Fish.and_(&["foo", "bar"]), "foo; and bar");
        assert_eq!(Fish.or_(&["foo", "bar"]), "foo; or bar");
        assert_eq!(Fish.script_from_history("- cmd: ls -la\n"), "ls -la\n");
        assert_eq!(Fish.script_from_history("   when: 1432613911\n"), "");
    }

    #[test]
    fn generates_alias() {
        let alias = Fish.app_alias("fuck");
        assert!(alias.starts_with("function fuck -d \"Correct your previous console command\"\n  set -l fucked_up_command $history[1]\n"));
        assert!(alias.contains("  env TF_SHELL=fish TF_ALIAS=fuck thefuck $fucked_up_command THEFUCK_ARGUMENT_PLACEHOLDER $argv | read -l unfucked_command\n"));
        assert!(alias.contains("    eval $unfucked_command\n    builtin history delete --exact --case-sensitive -- $fucked_up_command\n    builtin history merge\n  end\nend"));
    }
}
