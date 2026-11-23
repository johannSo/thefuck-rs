use std::collections::HashMap;

use super::{
    aliases_from_env, instant_mode_alias, unix_time, unquote, version_output, Shell,
    ShellConfiguration,
};
use crate::conf::settings;
use crate::consts::{ARGUMENT_PLACEHOLDER, USER_COMMAND_MARK};
use crate::utils::expanduser;

pub struct Zsh;

fn parse_alias(alias: &str) -> Option<(String, String)> {
    let (name, value) = alias.split_once('=')?;
    Some((name.to_string(), unquote(value).to_string()))
}

impl Shell for Zsh {
    fn friendly_name(&self) -> &'static str {
        "ZSH"
    }

    fn app_alias(&self, alias_name: &str) -> String {
        // It is VERY important to have the variables declared WITHIN the function
        format!(
            "
            {name} () {{
                export TF_SHELL=zsh;
                export TF_ALIAS={name};
                TF_SHELL_ALIASES=$(alias);
                export TF_SHELL_ALIASES;
                TF_HISTORY=\"$(fc -ln -10)\";
                export TF_HISTORY;
                TF_CMD=$(
                    thefuck {argument_placeholder} $@
                ) && eval $TF_CMD;
                unset TF_HISTORY;
                {alter_history}
            }}
        ",
            name = alias_name,
            argument_placeholder = ARGUMENT_PLACEHOLDER,
            alter_history = if settings().alter_history {
                "test -n \"$TF_CMD\" && print -s $TF_CMD"
            } else {
                ""
            }
        )
    }

    fn instant_mode_alias(&self, alias_name: &str) -> String {
        let mark = format!(
            "%{{{}{}%}}",
            USER_COMMAND_MARK,
            "\x08".repeat(USER_COMMAND_MARK.chars().count())
        );
        instant_mode_alias(self, alias_name, &mark, "rm -f")
    }

    fn get_aliases(&self) -> HashMap<String, String> {
        aliases_from_env(parse_alias)
    }

    fn history_file_name(&self) -> String {
        std::env::var("HISTFILE")
            .unwrap_or_else(|_| expanduser("~/.zsh_history").to_string_lossy().into_owned())
    }

    fn history_line(&self, command_script: &str) -> String {
        format!(": {}:0;{}\n", unix_time(), command_script)
    }

    fn script_from_history(&self, line: &str) -> String {
        line.split_once(';')
            .map_or("", |(_, script)| script)
            .to_string()
    }

    fn how_to_configure(&self) -> Option<ShellConfiguration> {
        Some(ShellConfiguration::new(
            "eval $(thefuck --alias)",
            "~/.zshrc",
            "source ~/.zshrc",
        ))
    }

    fn version(&self) -> String {
        version_output("zsh", &["-c", "echo $ZSH_VERSION"])
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
            parse_alias("l='ls -CF'").unwrap(),
            ("l".into(), "ls -CF".into())
        );
        assert_eq!(parse_alias("g=git").unwrap(), ("g".into(), "git".into()));
        assert_eq!(
            parse_alias("ll=\"ls -alF\"").unwrap(),
            ("ll".into(), "ls -alF".into())
        );
    }

    #[test]
    fn reads_and_formats_history() {
        assert_eq!(
            Zsh.script_from_history(": 1432613911:0;ls -la; pwd\n"),
            "ls -la; pwd\n"
        );
        assert_eq!(Zsh.script_from_history("ls\n"), "");
        assert!(Zsh.history_line("git push").ends_with(":0;git push\n"));
    }

    #[test]
    fn generates_alias() {
        let alias = Zsh.app_alias("fuck");
        assert!(alias.contains("fuck () {"));
        assert!(alias.contains("thefuck THEFUCK_ARGUMENT_PLACEHOLDER $@"));
        assert!(alias.contains("print -s $TF_CMD"));
    }
}
