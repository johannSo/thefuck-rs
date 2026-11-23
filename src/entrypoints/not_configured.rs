//! What the `fuck` binary does: it's only reachable when the alias of the
//! same name isn't configured, so it explains how to configure the alias, and
//! does it itself when called twice in a row.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::os::unix::process::parent_id;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{json, Value};

use crate::conf;
use crate::consts::CONFIGURATION_TIMEOUT;
use crate::logs;
use crate::shells::{shell, ShellConfiguration};
use crate::utils::expanduser;

fn now() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0.0, |since| since.as_secs_f64())
}

/// Returns path of special file where we store latest shell pid.
fn get_not_configured_usage_tracker_path() -> PathBuf {
    let user = ["LOGNAME", "USER", "LNAME", "USERNAME"]
        .iter()
        .find_map(|name| std::env::var(name).ok().filter(|user| !user.is_empty()))
        // SAFETY: getuid can't fail.
        .unwrap_or_else(|| unsafe { libc::getuid() }.to_string());
    std::env::temp_dir().join(format!("thefuck.last_not_configured_run_{}", user))
}

/// Records shell pid to tracker file.
fn record_first_run() {
    let info = json!({"pid": parent_id(), "time": now()});
    let _ = fs::write(get_not_configured_usage_tracker_path(), info.to_string());
}

/// Returns `true` when the tracker says that this shell already ran `fuck`,
/// and that it was the previous command or it happened recently.
fn is_second_run_recorded(
    info: &Value,
    shell_pid: u32,
    previous_command: Option<&str>,
    now: f64,
) -> bool {
    if !info.is_object() || info["pid"] != json!(shell_pid) {
        return false;
    }

    previous_command == Some("fuck")
        || now - info["time"].as_f64().unwrap_or(0.0) < CONFIGURATION_TIMEOUT
}

/// Returns `true` when we know that `fuck` called second time.
fn is_second_run() -> bool {
    let Ok(tracker) = fs::read_to_string(get_not_configured_usage_tracker_path()) else {
        return false;
    };
    let Ok(info) = serde_json::from_str::<Value>(&tracker) else {
        return false;
    };
    let history = shell().get_history();
    is_second_run_recorded(
        &info,
        parent_id(),
        history.last().map(String::as_str),
        now(),
    )
}

/// Returns `true` when alias already in shell config.
fn is_already_configured(configuration_details: &ShellConfiguration) -> bool {
    fs::read_to_string(expanduser(&configuration_details.path))
        .is_ok_and(|shell_config| shell_config.contains(&configuration_details.content))
}

/// Adds alias to shell config.
fn configure(configuration_details: &ShellConfiguration) -> std::io::Result<()> {
    let mut shell_config = OpenOptions::new()
        .append(true)
        .open(expanduser(&configuration_details.path))?;
    write!(shell_config, "\n{}\n", configuration_details.content)
}

/// Shows useful information about how-to configure alias on a first run
/// and configure automatically on a second.
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
        } else if is_second_run() {
            match configure(details) {
                Ok(()) => logs::configured_successfully(details),
                Err(error) => logs::exception(
                    &format!("Can't update {}", details.path),
                    &error.to_string(),
                ),
            }
            return;
        } else {
            record_first_run();
        }
    }

    logs::how_to_configure_alias(configuration_details.as_ref());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_second_run() {
        let info = json!({"pid": 12, "time": 1000.0});
        assert!(is_second_run_recorded(&info, 12, Some("ls"), 1010.0));
        assert!(is_second_run_recorded(&info, 12, Some("fuck"), 5000.0));
        assert!(!is_second_run_recorded(&info, 12, Some("ls"), 5000.0));
        assert!(!is_second_run_recorded(&info, 12, None, 5000.0));
        assert!(!is_second_run_recorded(&info, 13, Some("fuck"), 1010.0));
        assert!(!is_second_run_recorded(
            &json!([12]),
            12,
            Some("fuck"),
            1010.0
        ));
        assert!(!is_second_run_recorded(
            &json!({}),
            12,
            Some("fuck"),
            1010.0
        ));
    }
}
