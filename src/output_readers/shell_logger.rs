//! Obtains the output from an external shell logger listening on a socket,
//! see <https://github.com/nvbn/shell_logger>.

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::Path;

use serde_json::{json, Value};

use super::render;
use crate::consts::{SHELL_LOGGER_LIMIT, SHELL_LOGGER_SOCKET_ENV};
use crate::logs;

fn get_socket_path() -> Option<String> {
    std::env::var(SHELL_LOGGER_SOCKET_ENV)
        .ok()
        .filter(|path| !path.is_empty())
}

/// Returns `true` if shell logger socket available.
pub fn is_available() -> bool {
    get_socket_path().is_some_and(|path| Path::new(&path).exists())
}

fn get_last_n(n: usize) -> Option<Vec<Value>> {
    let mut client = UnixStream::connect(get_socket_path()?).ok()?;
    let request = format!("{}\n", json!({"type": "list", "count": n}));
    client.write_all(request.as_bytes()).ok()?;
    let mut response = String::new();
    BufReader::new(client).read_line(&mut response).ok()?;
    let response: Value = serde_json::from_str(&response).ok()?;
    response["commands"].as_array().cloned()
}

/// Gets command output from shell logger.
pub fn get_output(script: &str) -> Option<String> {
    logs::debug_time("Read output from external shell logger", || {
        let commands = get_last_n(SHELL_LOGGER_LIMIT).unwrap_or_default();
        let output = commands
            .iter()
            .find(|command| command["command"] == script)
            .and_then(|command| command["output"].as_str())
            .map(|output| render(&output.split('\n').collect::<Vec<_>>()));
        if output.is_none() {
            logs::warn("Output isn't available in shell logger");
        }
        output
    })
}
