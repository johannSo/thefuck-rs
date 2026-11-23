//! Ways of obtaining the output of the command that should be fixed.

use crate::conf::settings;

pub mod read_log;
pub mod rerun;
pub mod shell_logger;

/// Get output of the script.
///
/// `script` is the console script as typed, `expanded` is the same script
/// with expanded aliases.
pub fn get_output(script: &str, expanded: &str) -> Option<String> {
    if cfg!(test) {
        // Tests never run the commands they are fixing.
        return None;
    }
    if shell_logger::is_available() {
        shell_logger::get_output(script)
    } else if settings().instant_mode {
        read_log::get_output(script)
    } else {
        rerun::get_output(script, expanded)
    }
}

/// Renders output of a terminal application the way a terminal would show
/// it: applies carriage returns, cursor movements and drops colors.
fn render(lines: &[&str]) -> String {
    let rows = u16::try_from(lines.len()).unwrap_or(u16::MAX).max(1);
    let columns = crate::system::terminal_columns();
    let mut parser = vt100::Parser::new(rows, columns, 0);
    parser.process(lines.join("\n").as_bytes());
    let display: Vec<String> = parser.screen().rows(0, columns).collect();
    display.join("\n").trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::render;

    #[test]
    fn renders_terminal_output() {
        assert_eq!(
            render(&["$ ls\r", "\x1b[31mfoo\x1b[0m  bar\r", "$ "]),
            "$ ls\nfoo  bar\n$"
        );
        assert_eq!(
            render(&["git psuh\x08\x08\x08ush\r", "done\r"]),
            "git push\ndone"
        );
        assert_eq!(
            render(&["progress 10%\rprogress 99%\r", ""]),
            "progress 99%"
        );
    }
}
