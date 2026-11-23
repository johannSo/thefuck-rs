use crate::argument_parser;
use crate::logs;
use crate::shells::shell;

use super::alias::print_alias;
use super::fix_command::fix_command;
use super::shell_logger::shell_logger;
use super::update;

/// Entry point of the `thefuck` binary.
pub fn main() {
    let argv: Vec<String> = std::env::args_os()
        .map(|arg| arg.to_string_lossy().into_owned())
        .collect();
    let known_args =
        argument_parser::parse(&argv).unwrap_or_else(|error| argument_parser::fail(&error));

    if known_args.help {
        argument_parser::print_help();
    } else if known_args.version {
        logs::version(crate::version(), &shell().info());
    } else if known_args.update {
        update::main();
    // It's important to check if an alias is being requested before checking if
    // `TF_HISTORY` is in `os.environ`, otherwise it might mess with subshells.
    // Check https://github.com/nvbn/thefuck/issues/921 for reference
    } else if known_args.alias.is_some() {
        print_alias(&known_args);
    } else if !known_args.command.is_empty() || std::env::var_os("TF_HISTORY").is_some() {
        fix_command(&known_args);
    } else if let Some(output) = &known_args.shell_logger {
        shell_logger(output);
    } else {
        argument_parser::print_usage();
    }
}
