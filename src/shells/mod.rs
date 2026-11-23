//! Shell specific actions.
//!
//! Every supported shell implements [`Shell`], overriding the parts in which
//! it differs from a generic POSIX-like shell.

use std::sync::OnceLock;

mod bash;
mod fish;
mod generic;
mod powershell;
mod tcsh;
mod zsh;

pub use bash::Bash;
pub use fish::Fish;
pub use generic::{Generic, Shell, ShellConfiguration};
pub use powershell::Powershell;
pub use tcsh::Tcsh;
pub use zsh::Zsh;

fn shell_by_name(name: &str) -> Option<Box<dyn Shell>> {
    Some(match name {
        "bash" => Box::new(Bash),
        "fish" => Box::new(Fish),
        "zsh" => Box::new(Zsh),
        "csh" | "tcsh" => Box::new(Tcsh),
        "powershell" | "pwsh" => Box::new(Powershell),
        _ => return None,
    })
}

fn get_shell_from_env() -> Option<Box<dyn Shell>> {
    shell_by_name(&std::env::var("TF_SHELL").ok()?)
}

fn get_shell_from_proc() -> Box<dyn Shell> {
    crate::system::ancestor_names()
        .iter()
        .find_map(|name| shell_by_name(name.split('.').next().unwrap_or("")))
        .unwrap_or_else(|| Box::new(Generic))
}

/// Returns the shell The Fuck was called from.
pub fn shell() -> &'static dyn Shell {
    static SHELL: OnceLock<Box<dyn Shell>> = OnceLock::new();
    SHELL
        .get_or_init(|| {
            if cfg!(test) {
                Box::new(Generic)
            } else {
                get_shell_from_env().unwrap_or_else(get_shell_from_proc)
            }
        })
        .as_ref()
}

/// Returns a random name suffix for a temporary file.
fn unique_suffix() -> String {
    use std::hash::{BuildHasher, Hasher};
    // `RandomState` is seeded randomly for every instance.
    let random = |salt: u64| {
        let mut hasher = std::collections::hash_map::RandomState::new().build_hasher();
        hasher.write_u64(salt);
        hasher.finish()
    };
    format!(
        "{:016x}{:016x}",
        random(u64::from(std::process::id())),
        random(0)
    )
}

/// Returns the instant mode alias of shells supporting it: the first call
/// starts the shell logger, in which the shell is started again and gets the
/// usual alias together with the marked prompt.
fn instant_mode_alias(shell: &dyn Shell, alias_name: &str, mark: &str, remove_log: &str) -> String {
    if std::env::var("THEFUCK_INSTANT_MODE").is_ok_and(|mode| mode.to_lowercase() == "true") {
        format!(
            "
                export PS1=\"{mark}$PS1\";
                {app_alias}
            ",
            mark = mark,
            app_alias = shell.app_alias(alias_name)
        )
    } else {
        let log_path = std::env::temp_dir().join(format!("thefuck-script-log-{}", unique_suffix()));
        format!(
            "
                export THEFUCK_INSTANT_MODE=True;
                export THEFUCK_OUTPUT_LOG={log};
                thefuck --shell-logger {log};
                {remove_log} {log};
                exit
            ",
            log = log_path.display(),
            remove_log = remove_log
        )
    }
}

/// Parses `name=value` lines of the `alias` builtin, passed by the shell
/// alias in `TF_SHELL_ALIASES`.
fn aliases_from_env(
    parse: fn(&str) -> Option<(String, String)>,
) -> std::collections::HashMap<String, String> {
    std::env::var("TF_SHELL_ALIASES")
        .unwrap_or_default()
        .split('\n')
        .filter(|alias| !alias.is_empty() && alias.contains('='))
        .filter_map(parse)
        .collect()
}

/// Removes quotes around the value of an alias.
fn unquote(value: &str) -> &str {
    let quoted = |quote| value.len() >= 2 && value.starts_with(quote) && value.ends_with(quote);
    if quoted('"') || quoted('\'') {
        &value[1..value.len() - 1]
    } else {
        value
    }
}

/// Returns the first line of the output of the program.
fn version_output(program: &str, args: &[&str]) -> String {
    crate::utils::run(program, args).stdout
}

fn unix_time() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |since| since.as_secs())
}
