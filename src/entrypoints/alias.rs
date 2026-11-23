use crate::argument_parser::Args;
use crate::conf;
use crate::logs::warn;
use crate::shells::shell;
use crate::utils::which;

fn get_alias(known_args: &Args) -> String {
    let alias_name = known_args.alias.as_deref().unwrap_or("fuck");

    if known_args.enable_experimental_instant_mode {
        if which("script").is_none() {
            warn("Instant mode requires `script` app");
        } else {
            return shell().instant_mode_alias(alias_name);
        }
    }

    shell().app_alias(alias_name)
}

/// Prints the shell alias, the output is meant to be evaluated by the shell.
pub fn print_alias(known_args: &Args) {
    conf::init(Some(known_args));
    println!("{}", get_alias(known_args));
}
