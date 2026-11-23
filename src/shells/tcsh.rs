use std::collections::HashMap;

use super::{unix_time, version_output, Shell, ShellConfiguration};
use crate::utils::{expanduser, run};

pub struct Tcsh;

fn parse_aliases(output: &str) -> HashMap<String, String> {
    output
        .split('\n')
        .filter_map(|alias| alias.split_once('\t'))
        .map(|(name, value)| (name.to_string(), value.to_string()))
        .collect()
}

impl Shell for Tcsh {
    fn friendly_name(&self) -> &'static str {
        "Tcsh"
    }

    fn app_alias(&self, alias_name: &str) -> String {
        format!(
            "alias {0} 'setenv TF_SHELL tcsh && setenv TF_ALIAS {0} && \
             set fucked_cmd=`history -h 2 | head -n 1` && \
             eval `thefuck ${{fucked_cmd}}`'",
            alias_name
        )
    }

    fn get_aliases(&self) -> HashMap<String, String> {
        parse_aliases(&run("tcsh", &["-ic", "alias"]).stdout)
    }

    fn history_file_name(&self) -> String {
        std::env::var("HISTFILE")
            .unwrap_or_else(|_| expanduser("~/.history").to_string_lossy().into_owned())
    }

    fn history_line(&self, command_script: &str) -> String {
        format!("#+{}\n{}\n", unix_time(), command_script)
    }

    fn how_to_configure(&self) -> Option<ShellConfiguration> {
        Some(ShellConfiguration::new(
            "eval `thefuck --alias`",
            "~/.tcshrc",
            "tcsh",
        ))
    }

    fn version(&self) -> String {
        version_output("tcsh", &["--version"])
            .split_whitespace()
            .nth(1)
            .unwrap_or("")
            .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_aliases() {
        let aliases = parse_aliases("fuck\teval `thefuck ${fucked_cmd}`\nl\tls -CF\nbroken\n");
        assert_eq!(aliases.len(), 2);
        assert_eq!(aliases["l"], "ls -CF");
    }

    #[test]
    fn generates_alias() {
        let alias = Tcsh.app_alias("fuck");
        assert!(alias.starts_with("alias fuck 'setenv TF_SHELL tcsh && setenv TF_ALIAS fuck && "));
        assert!(alias.ends_with("eval `thefuck ${fucked_cmd}`'"));
    }
}
