//! Runs the shell in a pseudo terminal, logging everything it shows.
//!
//! Works like unix script command with `-f` flag. The log is used by the
//! experimental instant mode, see `output_readers::read_log`.

use std::ffi::CString;
use std::fs::{File, OpenOptions};
use std::os::unix::fs::FileExt;
use std::sync::atomic::{AtomicI32, Ordering};

use crate::consts::{LOG_SIZE_IN_BYTES, LOG_SIZE_TO_CLEAN};
use crate::logs;

/// Log of the fixed size, the oldest data is dropped when it's full.
struct Log {
    file: File,
    buffer: Vec<u8>,
    position: usize,
}

impl Log {
    fn create(path: &str) -> std::io::Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)?;
        let buffer = vec![0; LOG_SIZE_IN_BYTES];
        file.write_all_at(&buffer, 0)?;
        Ok(Log {
            file,
            buffer,
            position: 0,
        })
    }

    fn write(&mut self, data: &[u8]) {
        let data = &data[data.len().saturating_sub(LOG_SIZE_TO_CLEAN)..];
        if self.position + data.len() > LOG_SIZE_IN_BYTES {
            let position = LOG_SIZE_IN_BYTES - LOG_SIZE_TO_CLEAN;
            self.buffer.copy_within(LOG_SIZE_TO_CLEAN.., 0);
            self.buffer[position..].fill(0);
            self.position = self.position.saturating_sub(LOG_SIZE_TO_CLEAN);
            let _ = self.file.write_all_at(&self.buffer, 0);
        }
        self.buffer[self.position..self.position + data.len()].copy_from_slice(data);
        let _ = self.file.write_all_at(data, self.position as u64);
        self.position += data.len();
    }
}

static MASTER_FD: AtomicI32 = AtomicI32::new(-1);

/// Makes the pseudo terminal as big as the real one.
fn set_pty_size() {
    let master_fd = MASTER_FD.load(Ordering::Relaxed);
    // SAFETY: ioctls on valid descriptors with a properly sized struct; both
    // are async-signal-safe, as the function is used as a signal handler.
    unsafe {
        let mut size: libc::winsize = std::mem::zeroed();
        if libc::ioctl(libc::STDOUT_FILENO, libc::TIOCGWINSZ, &mut size) == 0 {
            libc::ioctl(master_fd, libc::TIOCSWINSZ, &size);
        }
    }
}

extern "C" fn on_window_change(_signal: libc::c_int) {
    set_pty_size();
}

fn write_all(fd: libc::c_int, mut data: &[u8]) {
    while !data.is_empty() {
        // SAFETY: writes from a valid buffer of the given length.
        let written = unsafe { libc::write(fd, data.as_ptr().cast(), data.len()) };
        if written > 0 {
            data = &data[written as usize..];
        } else if std::io::Error::last_os_error().kind() != std::io::ErrorKind::Interrupted {
            return;
        }
    }
}

fn read(fd: libc::c_int, buffer: &mut [u8]) -> Option<usize> {
    loop {
        // SAFETY: reads into a valid buffer of the given length.
        let count = unsafe { libc::read(fd, buffer.as_mut_ptr().cast(), buffer.len()) };
        if count > 0 {
            return Some(count as usize);
        } else if count == 0
            || std::io::Error::last_os_error().kind() != std::io::ErrorKind::Interrupted
        {
            // End of file; for the pty it's reported as EIO when the shell exits.
            return None;
        }
    }
}

/// Passes input to the shell and its output to the terminal and the log.
fn copy(master_fd: libc::c_int, log: &mut Log) {
    let mut buffer = [0u8; 1024];
    let mut fds = [
        libc::pollfd {
            fd: master_fd,
            events: libc::POLLIN,
            revents: 0,
        },
        libc::pollfd {
            fd: libc::STDIN_FILENO,
            events: libc::POLLIN,
            revents: 0,
        },
    ];
    loop {
        // SAFETY: `fds` is a valid array of two pollfd structs.
        if unsafe { libc::poll(fds.as_mut_ptr(), 2, -1) } < 0 {
            if std::io::Error::last_os_error().kind() == std::io::ErrorKind::Interrupted {
                continue;
            }
            return;
        }
        if fds[0].revents != 0 {
            match read(master_fd, &mut buffer) {
                Some(count) => {
                    log.write(&buffer[..count]);
                    write_all(libc::STDOUT_FILENO, &buffer[..count]);
                }
                None => return,
            }
        }
        if fds[1].revents != 0 {
            match read(libc::STDIN_FILENO, &mut buffer) {
                Some(count) => write_all(master_fd, &buffer[..count]),
                // Input is closed, stop polling it.
                None => fds[1].fd = -1,
            }
        }
    }
}

/// Create a spawned process. Returns its exit code.
fn spawn(shell: &CString, log: &mut Log) -> i32 {
    let mut master_fd: libc::c_int = -1;
    // SAFETY: the process is single threaded at this point; the child only
    // calls async-signal-safe functions before exec.
    let pid = unsafe {
        libc::forkpty(
            &mut master_fd,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        )
    };
    if pid < 0 {
        logs::warn(&format!(
            "Can't create a pseudo terminal: {}",
            std::io::Error::last_os_error()
        ));
        return 1;
    }
    if pid == 0 {
        // SAFETY: arguments are valid NUL-terminated strings, the list ends with NULL.
        unsafe {
            libc::execlp(
                shell.as_ptr(),
                shell.as_ptr(),
                std::ptr::null::<libc::c_char>(),
            );
            libc::_exit(127);
        }
    }

    // SAFETY: termios is plain data; attributes are restored below.
    let original_mode = unsafe {
        let mut mode: libc::termios = std::mem::zeroed();
        if libc::tcgetattr(libc::STDIN_FILENO, &mut mode) == 0 {
            let mut raw = mode;
            libc::cfmakeraw(&mut raw);
            libc::tcsetattr(libc::STDIN_FILENO, libc::TCSAFLUSH, &raw);
            Some(mode)
        } else {
            None
        }
    };

    MASTER_FD.store(master_fd, Ordering::Relaxed);
    set_pty_size();
    // SAFETY: the handler only calls async-signal-safe functions.
    unsafe {
        libc::signal(
            libc::SIGWINCH,
            on_window_change as extern "C" fn(libc::c_int) as libc::sighandler_t,
        );
    }

    copy(master_fd, log);

    let mut status: libc::c_int = 0;
    // SAFETY: restores the state changed above and reaps the child.
    unsafe {
        libc::signal(libc::SIGWINCH, libc::SIG_DFL);
        if let Some(mode) = original_mode {
            libc::tcsetattr(libc::STDIN_FILENO, libc::TCSAFLUSH, &mode);
        }
        libc::close(master_fd);
        libc::waitpid(pid, &mut status, 0);
    }
    if libc::WIFEXITED(status) {
        libc::WEXITSTATUS(status)
    } else {
        128 + libc::WTERMSIG(status)
    }
}

/// Logs shell output to the `output`.
pub fn shell_logger(output: &str) -> ! {
    let shell = std::env::var("SHELL")
        .ok()
        .filter(|shell| !shell.is_empty());
    let Some(shell) = shell.and_then(|shell| CString::new(shell).ok()) else {
        logs::warn("Shell logger doesn't support your platform.");
        std::process::exit(1);
    };

    let mut log = match Log::create(output) {
        Ok(log) => log,
        Err(error) => {
            logs::warn(&format!("Can't create the log {}: {}", output, error));
            std::process::exit(1);
        }
    };
    let return_code = spawn(&shell, &mut log);

    std::process::exit(return_code);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn log_drops_the_oldest_data_when_full() {
        let path = std::env::temp_dir().join(format!("thefuck-test-log-{}", std::process::id()));
        let mut log = Log::create(path.to_str().unwrap()).unwrap();
        assert_eq!(
            std::fs::metadata(&path).unwrap().len(),
            LOG_SIZE_IN_BYTES as u64
        );

        let chunk = [b'a'; 1024];
        for _ in 0..LOG_SIZE_IN_BYTES / 1024 {
            log.write(&chunk);
        }
        log.write(b"the end");

        let content = std::fs::read(&path).unwrap();
        std::fs::remove_file(&path).unwrap();
        assert_eq!(content.len(), LOG_SIZE_IN_BYTES);
        let written = LOG_SIZE_IN_BYTES - LOG_SIZE_TO_CLEAN;
        assert!(content[..written].iter().all(|byte| *byte == b'a'));
        assert_eq!(&content[written..written + 7], b"the end");
        assert!(content[written + 7..].iter().all(|byte| *byte == 0));
    }
}
