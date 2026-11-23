//! Interaction with the terminal and the operating system.

use std::fs;
use std::process::{Command, Stdio};

use crate::consts::Key;
use crate::utils::which;

/// Puts the terminal into raw mode, restores it when dropped.
struct RawMode {
    original: Option<libc::termios>,
}

impl RawMode {
    fn new() -> Self {
        // SAFETY: termios is plain data, the calls only read/write the struct.
        unsafe {
            let mut original: libc::termios = std::mem::zeroed();
            if libc::tcgetattr(libc::STDIN_FILENO, &mut original) != 0 {
                return RawMode { original: None };
            }
            let mut raw = original;
            libc::cfmakeraw(&mut raw);
            libc::tcsetattr(libc::STDIN_FILENO, libc::TCSAFLUSH, &raw);
            RawMode {
                original: Some(original),
            }
        }
    }
}

impl Drop for RawMode {
    fn drop(&mut self) {
        if let Some(original) = self.original {
            // SAFETY: restores the attributes obtained in `new`.
            unsafe {
                libc::tcsetattr(libc::STDIN_FILENO, libc::TCSADRAIN, &original);
            }
        }
    }
}

fn read_byte() -> Option<u8> {
    let mut byte = 0u8;
    loop {
        // SAFETY: reads at most one byte into a valid one byte buffer.
        let count = unsafe { libc::read(libc::STDIN_FILENO, (&mut byte as *mut u8).cast(), 1) };
        match count {
            1 => return Some(byte),
            -1 if std::io::Error::last_os_error().kind() == std::io::ErrorKind::Interrupted => {
                continue
            }
            _ => return None,
        }
    }
}

/// Reads one char from the terminal. Returns `None` when the input is closed.
fn read_char() -> Option<char> {
    let first = read_byte()?;
    let len = match first {
        byte if byte >= 0xf0 => 4,
        byte if byte >= 0xe0 => 3,
        byte if byte >= 0xc0 => 2,
        _ => 1,
    };
    let mut bytes = vec![first];
    while bytes.len() < len {
        bytes.push(read_byte()?);
    }
    Some(
        std::str::from_utf8(&bytes)
            .ok()
            .and_then(|s| s.chars().next())
            .unwrap_or('\u{fffd}'),
    )
}

/// Waits for a key press, without waiting for enter and without echoing the
/// key. A closed input counts as ctrl+c.
pub fn get_key() -> Key {
    // The terminal stays in raw mode for the whole key: entering the mode
    // drops pending input, which would break keys sending several chars.
    let _raw_mode = RawMode::new();
    match read_char() {
        None | Some('\x03') => Key::CtrlC,
        Some('\x0e') => Key::CtrlN,
        Some('\x10') => Key::CtrlP,
        Some('\x1b') => match read_char() {
            Some('[') => match read_char() {
                Some('A') => Key::Up,
                Some('B') => Key::Down,
                Some(ch) => Key::Char(ch),
                None => Key::CtrlC,
            },
            Some(ch) => Key::Char(ch),
            None => Key::CtrlC,
        },
        Some(ch) => Key::Char(ch),
    }
}

/// Returns the command opening `arg` with the default application.
pub fn open_command(arg: &str) -> String {
    if which("xdg-open").is_some() {
        format!("xdg-open {}", arg)
    } else {
        format!("open {}", arg)
    }
}

fn window_size(fd: libc::c_int) -> Option<libc::winsize> {
    // SAFETY: TIOCGWINSZ fills the passed winsize struct.
    unsafe {
        let mut size: libc::winsize = std::mem::zeroed();
        (libc::ioctl(fd, libc::TIOCGWINSZ, &mut size) == 0 && size.ws_col > 0).then_some(size)
    }
}

/// Returns the width of the terminal, 80 if it can't be determined.
pub fn terminal_columns() -> u16 {
    if let Some(columns) = std::env::var("COLUMNS")
        .ok()
        .and_then(|c| c.parse().ok())
        .filter(|c| *c > 0)
    {
        return columns;
    }
    [libc::STDOUT_FILENO, libc::STDERR_FILENO, libc::STDIN_FILENO]
        .into_iter()
        .find_map(window_size)
        .map_or(80, |size| size.ws_col)
}

/// A running process: `(pid, parent pid, name)`.
pub type Process = (u32, u32, String);

fn processes_from_proc() -> Option<Vec<Process>> {
    let mut processes = Vec::new();
    for entry in fs::read_dir("/proc").ok()?.flatten() {
        let Some(pid) = entry
            .file_name()
            .to_str()
            .and_then(|name| name.parse::<u32>().ok())
        else {
            continue;
        };
        let Ok(stat) = fs::read_to_string(entry.path().join("stat")) else {
            continue;
        };
        // The format is `pid (name) state ppid ...`, the name can contain
        // anything, including spaces and parentheses.
        let (Some(open), Some(close)) = (stat.find('('), stat.rfind(')')) else {
            continue;
        };
        let Some(ppid) = stat[close + 1..]
            .split_whitespace()
            .nth(1)
            .and_then(|p| p.parse().ok())
        else {
            continue;
        };
        processes.push((pid, ppid, stat[open + 1..close].to_string()));
    }
    (!processes.is_empty()).then_some(processes)
}

fn processes_from_ps() -> Vec<Process> {
    let output = Command::new("ps")
        .args(["-A", "-o", "pid=,ppid=,comm="])
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output();
    let Ok(output) = output else { return vec![] };
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| {
            let mut fields = line.split_whitespace();
            let pid = fields.next()?.parse().ok()?;
            let ppid = fields.next()?.parse().ok()?;
            let command = fields.collect::<Vec<_>>().join(" ");
            let name = command
                .rsplit('/')
                .next()
                .unwrap_or("")
                .trim_start_matches('-');
            Some((pid, ppid, name.to_string()))
        })
        .collect()
}

/// Returns all running processes.
pub fn processes() -> Vec<Process> {
    processes_from_proc().unwrap_or_else(processes_from_ps)
}

/// Returns the names of the ancestors of the current process, the closest
/// first.
pub fn ancestor_names() -> Vec<String> {
    let processes = processes();
    let mut names = Vec::new();
    let mut pid = std::process::id();
    while let Some((_, ppid, _)) = processes.iter().find(|process| process.0 == pid) {
        match processes.iter().find(|process| process.0 == *ppid) {
            Some((_, _, name)) if names.len() < 64 => names.push(name.clone()),
            _ => break,
        }
        pid = *ppid;
    }
    names
}

/// Returns the pids of all descendants of the process, children first.
pub fn descendants(pid: u32) -> Vec<u32> {
    let processes = processes();
    let mut result = vec![];
    let mut queue = vec![pid];
    while let Some(parent) = queue.pop() {
        for (child, ppid, _) in &processes {
            if *ppid == parent && !result.contains(child) && *child != pid {
                result.push(*child);
                queue.push(*child);
            }
        }
    }
    result
}
