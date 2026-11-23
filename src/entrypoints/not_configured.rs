//! What the `fuck` binary does: it's only reachable when the alias of the
//! same name isn't configured, so it configures the alias in the shell config
//! right away, or explains how to do it when that can't be done automatically.

use std::fs::{self, OpenOptions};
use std::io::Write;

use crate::conf;
use crate::logs;
use crate::shells::{shell, ShellConfiguration};
use crate::utils::expanduser;

/// Returns `true` when alias already in shell config.
fn is_already_configured(configuration_details: &ShellConfiguration) -> bool {
    fs::read_to_string(expanduser(&configuration_details.path))
        .is_ok_and(|shell_config| shell_config.contains(&configuration_details.content))
}

/// Adds alias to shell config, creating the config when there's none yet.
fn configure(configuration_details: &ShellConfiguration) -> std::io::Result<()> {
    let path = expanduser(&configuration_details.path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut shell_config = OpenOptions::new().append(true).create(true).open(path)?;
    write!(shell_config, "\n{}\n", configuration_details.content)
}

/// Configures the alias automatically when possible, otherwise shows how to
/// do it.
///
/// It'll be only visible when user type fuck and when alias isn't configured.
pub fn main() {
    conf::init(None);
    let configuration_details = shell().how_to_configure();
    if let Some(details) = configuration_details
        .as_ref()
        .filter(|details| details.can_configure_automatically)
    {
        if is_already_configured(details) {
            logs::already_configured(details);
            return;
        }
        match configure(details) {
            Ok(()) => {
                logs::configured_successfully(details);
                return;
            }
            Err(error) => logs::exception(
                &format!("Can't update {}", details.path),
                &error.to_string(),
            ),
        }
    }

    logs::how_to_configure_alias(configuration_details.as_ref());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn configures_once() {
        let path = std::env::temp_dir().join(format!("thefuck-configure-{}", std::process::id()));
        fs::write(&path, "export EDITOR=vim\n").unwrap();
        let details = ShellConfiguration {
            content: "eval \"$(thefuck --alias)\"".to_string(),
            path: path.to_string_lossy().into_owned(),
            reload: "source rc".to_string(),
            can_configure_automatically: true,
        };
        assert!(!is_already_configured(&details));
        configure(&details).unwrap();
        assert!(is_already_configured(&details));
        assert_eq!(
            fs::read_to_string(&path).unwrap(),
            "export EDITOR=vim\n\neval \"$(thefuck --alias)\"\n"
        );
        let _ = fs::remove_file(path);
    }

    #[test]
    fn creates_missing_config() {
        // Like `~/.zshrc` on a fresh macOS, or `config.fish` in a new dir.
        let dir = std::env::temp_dir().join(format!("thefuck-create-{}", std::process::id()));
        let path = dir.join("fish/config.fish");
        let details = ShellConfiguration {
            content: "thefuck --alias | source".to_string(),
            path: path.to_string_lossy().into_owned(),
            reload: "fish".to_string(),
            can_configure_automatically: true,
        };
        assert!(!is_already_configured(&details));
        configure(&details).unwrap();
        assert!(is_already_configured(&details));
        assert_eq!(
            fs::read_to_string(&path).unwrap(),
            "\nthefuck --alias | source\n"
        );
        let _ = fs::remove_dir_all(dir);
    }
}
