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

#[derive(Debug, PartialEq)]
enum BashConfig {
    Existing(&'static str),
    Create(&'static str),
    Unknown,
}

/// Returns the config file the alias belongs to.
///
/// Terminals of macOS start login shells, which read only the first existing
/// of `~/.bash_profile`, `~/.bash_login` and `~/.profile`, never `~/.bashrc`.
/// When none exists `~/.bash_profile` is created; it isn't when `~/.profile`
/// exists, as bash would stop reading `~/.profile` then.
///
/// Elsewhere terminals start interactive non-login shells, reading
/// `~/.bashrc`.
fn config_file(macos: bool, exists: impl Fn(&str) -> bool) -> BashConfig {
    if macos {
        ["~/.bash_profile", "~/.bash_login", "~/.profile"]
            .into_iter()
            .find(|config| exists(config))
            .map_or(BashConfig::Create("~/.bash_profile"), BashConfig::Existing)
    } else if exists("~/.bashrc") {
        BashConfig::Existing("~/.bashrc")
    } else if exists("~/.bash_profile") {
        BashConfig::Existing("~/.bash_profile")
    } else {
        BashConfig::Unknown
    }
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
        let content = "eval \"$(thefuck --alias)\"";
        let exists = |path: &str| expanduser(path).exists();
        Some(match config_file(cfg!(target_os = "macos"), exists) {
            BashConfig::Existing(config) => {
                ShellConfiguration::new(content, config, &format!("source {}", config))
            }
            BashConfig::Create(config) => ShellConfiguration::created_if_missing(
                content,
                config,
                &format!("source {}", config),
            ),
            BashConfig::Unknown => {
                ShellConfiguration::new(content, "bash config", "source bash config")
            }
        })
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
    fn picks_config_file() {
        let with = |files: &'static [&'static str]| move |path: &str| files.contains(&path);
        // macOS reads only the login files.
        assert_eq!(
            config_file(true, with(&[])),
            BashConfig::Create("~/.bash_profile")
        );
        assert_eq!(
            config_file(true, with(&["~/.bashrc"])),
            BashConfig::Create("~/.bash_profile")
        );
        assert_eq!(
            config_file(true, with(&["~/.bashrc", "~/.profile"])),
            BashConfig::Existing("~/.profile")
        );
        assert_eq!(
            config_file(true, with(&["~/.profile", "~/.bash_login"])),
            BashConfig::Existing("~/.bash_login")
        );
        assert_eq!(
            config_file(true, with(&["~/.profile", "~/.bash_profile"])),
            BashConfig::Existing("~/.bash_profile")
        );
        // Linux and the rest.
        assert_eq!(
            config_file(false, with(&["~/.bashrc", "~/.bash_profile"])),
            BashConfig::Existing("~/.bashrc")
        );
        assert_eq!(
            config_file(false, with(&["~/.bash_profile"])),
            BashConfig::Existing("~/.bash_profile")
        );
        assert_eq!(config_file(false, with(&[])), BashConfig::Unknown);
    }

    #[test]
    fn formats_history() {
        assert_eq!(Bash.history_line("ls"), "ls\n");
        assert_eq!(Bash.script_from_history("ls -la\n"), "ls -la\n");
        assert_eq!(Bash.and_(&["ls", "cd"]), "ls && cd");
    }
}
