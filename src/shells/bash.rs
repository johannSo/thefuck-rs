use std::collections::HashMap;

use super::{
    aliases_from_env, instant_mode_alias, unquote, version_output, Shell, ShellConfiguration,
};
use crate::conf::settings;
use crate::consts::{ARGUMENT_PLACEHOLDER, USER_COMMAND_MARK};
use crate::utils::expanduser;

pub struct Bash;

fn parse_alias(alias: &str) -> Option<(String, String)> {
    let alias = alias.replacen("alias ", "", 1);
    let (name, value) = alias.split_once('=')?;
    Some((name.to_string(), unquote(value).to_string()))
}

impl Shell for Bash {
    fn friendly_name(&self) -> &'static str {
        "Bash"
    }

    fn app_alias(&self, alias_name: &str) -> String {
        // It is VERY important to have the variables declared WITHIN the function
        format!(
            "
            function {name} () {{
                export TF_SHELL=bash;
                export TF_ALIAS={name};
                export TF_SHELL_ALIASES=$(alias);
                export TF_HISTORY=$(fc -ln -10);
                TF_CMD=$(
                    thefuck {argument_placeholder} \"$@\"
                ) && eval \"$TF_CMD\";
                unset TF_HISTORY;
                {alter_history}
            }}
        ",
            name = alias_name,
            argument_placeholder = ARGUMENT_PLACEHOLDER,
            alter_history = if settings().alter_history {
                "history -s $TF_CMD;"
            } else {
                ""
            }
        )
    }

    fn instant_mode_alias(&self, alias_name: &str) -> String {
        let mark = format!(
            "{}{}",
            USER_COMMAND_MARK,
            "\x08".repeat(USER_COMMAND_MARK.chars().count())
        );
        instant_mode_alias(self, alias_name, &mark, "rm")
    }

    fn get_aliases(&self) -> HashMap<String, String> {
        aliases_from_env(parse_alias)
    }

    fn history_file_name(&self) -> String {
        std::env::var("HISTFILE")
            .unwrap_or_else(|_| expanduser("~/.bash_history").to_string_lossy().into_owned())
    }

    fn history_line(&self, command_script: &str) -> String {
        format!("{}\n", command_script)
    }

    fn how_to_configure(&self) -> Option<ShellConfiguration> {
        let config = if expanduser("~/.bashrc").exists() {
            "~/.bashrc"
        } else if expanduser("~/.bash_profile").exists() {
            "~/.bash_profile"
        } else {
            "bash config"
        };

        Some(ShellConfiguration::new(
            "eval \"$(thefuck --alias)\"",
            config,
            &format!("source {}", config),
        ))
    }

    fn version(&self) -> String {
        version_output("bash", &["-c", "echo $BASH_VERSION"])
            .trim()
            .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_aliases() {
        assert_eq!(
            parse_alias("alias fuck='eval $(thefuck $(fc -ln -1))'")
                .unwrap()
                .0,
            "fuck"
        );
        assert_eq!(
            parse_alias("alias l='ls -CF'").unwrap(),
            ("l".into(), "ls -CF".into())
        );
        assert_eq!(
            parse_alias("alias g=git").unwrap(),
            ("g".into(), "git".into())
        );
        assert_eq!(
            parse_alias("alias empty=").unwrap(),
            ("empty".into(), "".into())
        );
        assert_eq!(
            parse_alias("alias quoted=\"a b\"").unwrap(),
            ("quoted".into(), "a b".into())
        );
    }

    #[test]
    fn generates_alias() {
        let alias = Bash.app_alias("fix");
        assert!(alias.contains("function fix ()"));
        assert!(alias.contains("TF_ALIAS=fix"));
        assert!(alias.contains("thefuck THEFUCK_ARGUMENT_PLACEHOLDER \"$@\""));
        assert!(alias.contains("history -s $TF_CMD;"));
    }

    #[test]
    fn formats_history() {
        assert_eq!(Bash.history_line("ls"), "ls\n");
        assert_eq!(Bash.script_from_history("ls -la\n"), "ls -la\n");
        assert_eq!(Bash.and_(&["ls", "cd"]), "ls && cd");
    }
}
