//! Everything that is shown to the user, apart from the fixed command itself.
//!
//! The fixed command is the only thing ever written to stdout (the shell alias
//! evaluates it), so all messages of the fixing flow go to stderr.

use std::time::Instant;

use crate::conf::settings;
use crate::consts::USER_COMMAND_MARK;
use crate::shells::ShellConfiguration;
use crate::types::CorrectedCommand;

const BRIGHT: &str = "\x1b[1m";
const RESET_ALL: &str = "\x1b[0m";
const RED: &str = "\x1b[31m";
const GREEN: &str = "\x1b[32m";
const BLUE: &str = "\x1b[34m";
const WARN: &str = "\x1b[41m\x1b[37m\x1b[1m";

/// Utility for ability to disabling colored output.
fn color(color: &'static str) -> &'static str {
    if settings().no_colors {
        ""
    } else {
        color
    }
}

pub fn warn(title: &str) {
    eprintln!("{}[WARN] {}{}", color(WARN), title, color(RESET_ALL));
}

pub fn exception(title: &str, details: &str) {
    eprintln!(
        "{warn}[WARN] {title}:{reset}\n{details}\n{warn}----------------------------{reset}\n",
        warn = color(WARN),
        reset = color(RESET_ALL),
        title = title,
        details = details.trim_end()
    );
}

pub fn rule_failed(rule_name: &str, details: &str) {
    exception(&format!("Rule {}", rule_name), details);
}

pub fn failed(msg: &str) {
    eprintln!("{}{}{}", color(RED), msg, color(RESET_ALL));
}

fn side_effect_mark(corrected_command: &CorrectedCommand) -> &'static str {
    if corrected_command.side_effect.is_some() {
        " (+side effect)"
    } else {
        ""
    }
}

pub fn show_corrected_command(corrected_command: &CorrectedCommand) {
    eprintln!(
        "{prefix}{bold}{script}{reset}{side_effect}",
        prefix = USER_COMMAND_MARK,
        bold = color(BRIGHT),
        script = corrected_command.script,
        reset = color(RESET_ALL),
        side_effect = side_effect_mark(corrected_command)
    );
}

pub fn confirm_text(corrected_command: &CorrectedCommand) {
    eprint!(
        "{prefix}\x1b[1K\r{bold}{script}{reset}{side_effect} \
         [{green}enter{reset}/{blue}↑{reset}/{blue}↓{reset}/{red}ctrl+c{reset}]",
        prefix = USER_COMMAND_MARK,
        bold = color(BRIGHT),
        script = corrected_command.script,
        reset = color(RESET_ALL),
        side_effect = side_effect_mark(corrected_command),
        green = color(GREEN),
        blue = color(BLUE),
        red = color(RED)
    );
}

pub fn debug(msg: &str) {
    if settings().debug {
        eprintln!(
            "{blue}{bold}DEBUG:{reset} {msg}",
            blue = color(BLUE),
            bold = color(BRIGHT),
            reset = color(RESET_ALL),
            msg = msg
        );
    }
}

/// Runs `f` and, in debug mode, reports how long it took.
pub fn debug_time<T>(msg: &str, f: impl FnOnce() -> T) -> T {
    if !settings().debug {
        return f();
    }
    let started = Instant::now();
    let result = f();
    let elapsed = started.elapsed();
    let seconds = elapsed.as_secs();
    debug(&format!(
        "{} took: {}:{:02}:{:02}.{:06}",
        msg,
        seconds / 3600,
        seconds / 60 % 60,
        seconds % 60,
        elapsed.subsec_micros()
    ));
    result
}

pub fn how_to_configure_alias(configuration_details: Option<&ShellConfiguration>) {
    let bold = color(BRIGHT);
    let reset = color(RESET_ALL);
    println!("Seems like {bold}fuck{reset} alias isn't configured!");

    if let Some(details) = configuration_details {
        println!(
            "Please put {bold}{content}{reset} in your {bold}{path}{reset} and apply \
             changes with {bold}{reload}{reset} or restart your shell.",
            content = details.content,
            path = details.path,
            reload = details.reload
        );

        if details.can_configure_automatically {
            println!("Or run {bold}fuck{reset} a second time to configure it automatically.");
        }
    }

    println!("More details - https://github.com/nvbn/thefuck#manual-installation");
}

pub fn already_configured(configuration_details: &ShellConfiguration) {
    println!(
        "Seems like {bold}fuck{reset} alias already configured!\n\
         For applying changes run {bold}{reload}{reset} or restart your shell.",
        bold = color(BRIGHT),
        reset = color(RESET_ALL),
        reload = configuration_details.reload
    );
}

pub fn configured_successfully(configuration_details: &ShellConfiguration) {
    println!(
        "{bold}fuck{reset} alias configured successfully!\n\
         For applying changes run {bold}{reload}{reset} or restart your shell.",
        bold = color(BRIGHT),
        reset = color(RESET_ALL),
        reload = configuration_details.reload
    );
}

pub fn version(thefuck_version: &str, shell_info: &str) {
    eprintln!("The Fuck {} using Rust and {}", thefuck_version, shell_info);
}
