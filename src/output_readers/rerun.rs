//! Obtains the output by running the command once more.

use std::io::Read;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use crate::conf::settings;
use crate::logs;
use crate::shlex;

/// The shell running the script; its path gets into error messages, like
/// `/bin/sh: line 1: gti: command not found`.
pub const SHELL: &str = "/bin/sh";

/// Tries to kill the process otherwise just logs a debug message, the
/// process will be killed when thefuck terminates.
fn kill_process(pid: u32) {
    // SAFETY: plain syscall, at worst it fails.
    if unsafe { libc::kill(pid as libc::pid_t, libc::SIGKILL) } != 0 {
        let error = std::io::Error::last_os_error();
        if error.raw_os_error() != Some(libc::ESRCH) {
            logs::debug(&format!(
                "Rerun: process PID {} could not be terminated: {}",
                pid, error
            ));
        }
    }
}

/// Runs the script and obtains stdout and stderr; `None` when the script
/// can't be run or doesn't finish in time.
pub fn get_output(script: &str, expanded: &str) -> Option<String> {
    let settings = settings();
    let split_expand = shlex::split(expanded).unwrap_or_default();
    let is_slow = split_expand
        .first()
        .is_some_and(|app| settings.slow_commands.contains(app));
    let timeout = if is_slow {
        settings.wait_slow_command
    } else {
        settings.wait_command
    };

    let message = format!(
        "Call: {}; with env: {:?}; is slow: {}",
        script, settings.env, is_slow
    );
    logs::debug_time(&message, || {
        let (mut reader, writer) = std::io::pipe().ok()?;
        let mut child = {
            let mut command = Command::new(SHELL);
            command
                .arg("-c")
                .arg(expanded)
                .envs(&settings.env)
                .stdin(Stdio::piped())
                .stdout(writer.try_clone().ok()?)
                .stderr(writer);
            command.spawn().ok()?
            // `command` is dropped here together with our copies of the
            // writing end, so reading ends when the script is done.
        };

        // Read in the background, a script filling the pipe would never
        // finish otherwise.
        let (sender, receiver) = mpsc::channel();
        thread::spawn(move || {
            let mut buffer = [0u8; 8192];
            loop {
                match reader.read(&mut buffer) {
                    Ok(0) | Err(_) => break,
                    Ok(count) => {
                        if sender.send(buffer[..count].to_vec()).is_err() {
                            break;
                        }
                    }
                }
            }
        });

        const TICK: Duration = Duration::from_millis(10);
        // Background processes started by the script can keep the pipe open
        // forever, so once the script itself is done, the rest of the output
        // is awaited only for a moment.
        const GRACE: Duration = Duration::from_millis(200);

        let deadline = Instant::now() + Duration::from_secs_f64(timeout.max(0.0));
        let mut output = Vec::new();
        let mut output_closed = false;
        let mut exited_at = None;
        loop {
            if output_closed {
                thread::sleep(TICK);
            } else {
                match receiver.recv_timeout(TICK) {
                    Ok(chunk) => output.extend(chunk),
                    Err(mpsc::RecvTimeoutError::Disconnected) => output_closed = true,
                    Err(mpsc::RecvTimeoutError::Timeout) => {}
                }
            }
            if exited_at.is_none() && !matches!(child.try_wait(), Ok(None)) {
                exited_at = Some(Instant::now());
            }
            match exited_at {
                Some(at) if output_closed || at.elapsed() > GRACE => break,
                None if Instant::now() >= deadline => {
                    for pid in crate::system::descendants(child.id()) {
                        kill_process(pid);
                    }
                    kill_process(child.id());
                    let _ = child.wait();
                    logs::debug("Execution timed out!");
                    return None;
                }
                _ => {}
            }
        }

        let output = String::from_utf8_lossy(&output).into_owned();
        logs::debug(&format!("Received output: {}", output));
        Some(output)
    })
}

#[cfg(test)]
mod tests {
    use super::get_output;

    #[test]
    fn reads_both_streams() {
        assert_eq!(
            get_output("x", "echo out; echo err >&2").unwrap(),
            "out\nerr\n"
        );
    }

    #[test]
    fn passes_env_from_settings() {
        assert_eq!(get_output("x", "echo $LANG $GIT_TRACE").unwrap(), "C 1\n");
    }

    #[test]
    fn reads_long_output() {
        let output = get_output("x", "head -c 300000 /dev/zero | tr '\\0' 'x'").unwrap();
        assert_eq!(output.len(), 300_000);
    }

    #[test]
    fn does_not_wait_for_background_processes() {
        assert_eq!(
            get_output("x", "sleep 30 & echo started").unwrap(),
            "started\n"
        );
    }
}
