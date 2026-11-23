//! Parser of command line arguments, which can handle arguments with our
//! special placeholder.

use crate::consts::ARGUMENT_PLACEHOLDER;
use crate::utils::get_alias;

const USAGE: &str = "\
usage: thefuck [-v] [-a [ALIAS]] [-l SHELL_LOGGER]
               [--enable-experimental-instant-mode] [-h] [-y | -r] [-d]
               [command ...]
";

const HELP: &str = "
positional arguments:
  command               command that should be fixed

options:
  -v, --version         show program's version number and exit
  -a, --alias [ALIAS]   [custom-alias-name] prints alias for current shell
  -l, --shell-logger SHELL_LOGGER
                        log shell output to the file
  --enable-experimental-instant-mode
                        enable experimental instant mode, use on your own risk
  -h, --help            show this help message and exit
  -y, --yes, --yeah, --hard
                        execute fixed command without confirmation
  -r, --repeat          repeat on failure
  -d, --debug           enable debug output
";

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Args {
    pub version: bool,
    pub alias: Option<String>,
    pub shell_logger: Option<String>,
    pub enable_experimental_instant_mode: bool,
    pub help: bool,
    pub yes: bool,
    pub repeat: bool,
    pub debug: bool,
    pub force_command: Option<String>,
    pub command: Vec<String>,
}

#[derive(Clone, Copy, PartialEq)]
enum Opt {
    Version,
    Alias,
    ShellLogger,
    InstantMode,
    Help,
    Yes,
    Repeat,
    Debug,
    ForceCommand,
}

const LONG_OPTIONS: &[(&str, Opt)] = &[
    ("version", Opt::Version),
    ("alias", Opt::Alias),
    ("shell-logger", Opt::ShellLogger),
    ("enable-experimental-instant-mode", Opt::InstantMode),
    ("help", Opt::Help),
    ("yes", Opt::Yes),
    ("yeah", Opt::Yes),
    ("hard", Opt::Yes),
    ("repeat", Opt::Repeat),
    ("debug", Opt::Debug),
    ("force-command", Opt::ForceCommand),
];

const SHORT_OPTIONS: &[(char, Opt)] = &[
    ('v', Opt::Version),
    ('a', Opt::Alias),
    ('l', Opt::ShellLogger),
    ('h', Opt::Help),
    ('y', Opt::Yes),
    ('r', Opt::Repeat),
    ('d', Opt::Debug),
];

impl Opt {
    fn display(self) -> &'static str {
        match self {
            Opt::Version => "-v/--version",
            Opt::Alias => "-a/--alias",
            Opt::ShellLogger => "-l/--shell-logger",
            Opt::InstantMode => "--enable-experimental-instant-mode",
            Opt::Help => "-h/--help",
            Opt::Yes => "-y/--yes/--yeah/--hard",
            Opt::Repeat => "-r/--repeat",
            Opt::Debug => "-d/--debug",
            Opt::ForceCommand => "--force-command",
        }
    }
}

fn long_option(name: &str) -> Result<Opt, String> {
    if let Some((_, opt)) = LONG_OPTIONS.iter().find(|(long, _)| *long == name) {
        return Ok(*opt);
    }
    // Unambiguous prefixes are accepted as well.
    let mut candidates: Vec<(&str, Opt)> = LONG_OPTIONS
        .iter()
        .filter(|(long, _)| long.starts_with(name))
        .copied()
        .collect();
    candidates.dedup_by(|x, y| x.1 == y.1);
    match candidates[..] {
        [(_, opt)] if !name.is_empty() => Ok(opt),
        [] | [_] => Err(format!("unrecognized arguments: --{}", name)),
        _ => {
            let matches: Vec<String> = candidates
                .iter()
                .map(|(long, _)| format!("--{}", long))
                .collect();
            Err(format!(
                "ambiguous option: --{} could match {}",
                name,
                matches.join(", ")
            ))
        }
    }
}

/// Prepares arguments by:
///
/// - removing placeholder and moving arguments after it to beginning,
///   we need this to distinguish arguments from `command` with ours;
///
/// - adding `--` before `command`, so our parse would ignore arguments
///   of `command`.
fn prepare_arguments(argv: &[String]) -> Vec<String> {
    let separator = || std::iter::once("--".to_string());
    if let Some(index) = argv.iter().position(|arg| arg == ARGUMENT_PLACEHOLDER) {
        argv[index + 1..]
            .iter()
            .cloned()
            .chain(separator())
            .chain(argv[..index].iter().cloned())
            .collect()
    } else if argv.first().is_some_and(|first| !first.starts_with('-')) {
        separator().chain(argv.iter().cloned()).collect()
    } else {
        argv.to_vec()
    }
}

/// Parses arguments, `argv` includes the name of the program.
pub fn parse(argv: &[String]) -> Result<Args, String> {
    let arguments = prepare_arguments(argv.get(1..).unwrap_or(&[]));
    let mut args = Args::default();
    let mut index = 0;
    let mut only_command = false;

    while index < arguments.len() {
        let argument = &arguments[index];
        index += 1;

        // Options with their values attached, if any.
        let mut options: Vec<(Opt, Option<String>)> = Vec::new();
        if only_command {
            args.command.push(argument.clone());
        } else if argument == "--" {
            only_command = true;
        } else if let Some(long) = argument.strip_prefix("--") {
            match long.split_once('=') {
                Some((name, value)) => options.push((long_option(name)?, Some(value.to_string()))),
                None => options.push((long_option(long)?, None)),
            }
        } else if argument.starts_with('-') && argument.len() > 1 {
            let cluster: Vec<char> = argument.chars().skip(1).collect();
            for (position, flag) in cluster.iter().enumerate() {
                let Some((_, opt)) = SHORT_OPTIONS.iter().find(|(short, _)| short == flag) else {
                    return Err(format!("unrecognized arguments: {}", argument));
                };
                let rest: String = cluster[position + 1..].iter().collect();
                if matches!(opt, Opt::Alias | Opt::ShellLogger) && !rest.is_empty() {
                    options.push((*opt, Some(rest)));
                    break;
                }
                options.push((*opt, None));
            }
        } else {
            args.command.push(argument.clone());
        }

        for (opt, value) in options {
            let takes_value = matches!(opt, Opt::Alias | Opt::ShellLogger | Opt::ForceCommand);
            if let Some(value) = value.as_ref().filter(|_| !takes_value) {
                return Err(format!(
                    "argument {}: ignored explicit argument {:?}",
                    opt.display(),
                    value
                ));
            }
            // A value can also be the next argument.
            let mut value = value;
            if value.is_none() && takes_value {
                if let Some(next) = arguments.get(index).filter(|next| !next.starts_with('-')) {
                    value = Some(next.clone());
                    index += 1;
                }
            }
            let conflict = |other: Opt| {
                Err(format!(
                    "argument {}: not allowed with argument {}",
                    opt.display(),
                    other.display()
                ))
            };
            match opt {
                Opt::Version => args.version = true,
                Opt::Alias => args.alias = Some(value.unwrap_or_else(get_alias)),
                Opt::InstantMode => args.enable_experimental_instant_mode = true,
                Opt::Help => args.help = true,
                // It's too dangerous to use `-y` and `-r` together.
                Opt::Yes if args.repeat => return conflict(Opt::Repeat),
                Opt::Yes => args.yes = true,
                Opt::Repeat if args.yes => return conflict(Opt::Yes),
                Opt::Repeat => args.repeat = true,
                Opt::Debug => args.debug = true,
                Opt::ShellLogger | Opt::ForceCommand => {
                    let Some(value) = value else {
                        return Err(format!("argument {}: expected one argument", opt.display()));
                    };
                    if opt == Opt::ShellLogger {
                        args.shell_logger = Some(value);
                    } else {
                        args.force_command = Some(value);
                    }
                }
            }
        }
    }

    Ok(args)
}

pub fn print_usage() {
    eprint!("{}", USAGE);
}

pub fn print_help() {
    eprint!("{}{}", USAGE, HELP);
}

/// Reports wrong arguments and terminates the application.
pub fn fail(message: &str) -> ! {
    print_usage();
    eprintln!("thefuck: error: {}", message);
    std::process::exit(2)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_args(args: &[&str]) -> Result<Args, String> {
        let argv: Vec<String> = std::iter::once("thefuck")
            .chain(args.iter().copied())
            .map(String::from)
            .collect();
        parse(&argv)
    }

    fn strings(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn parses_flags() {
        assert_eq!(parse_args(&[]).unwrap(), Args::default());
        assert_eq!(
            parse_args(&["-v"]).unwrap(),
            Args {
                version: true,
                ..Args::default()
            }
        );
        assert_eq!(
            parse_args(&["--help"]).unwrap(),
            Args {
                help: true,
                ..Args::default()
            }
        );
        assert_eq!(
            parse_args(&["-yd"]).unwrap(),
            Args {
                yes: true,
                debug: true,
                ..Args::default()
            }
        );
        assert_eq!(
            parse_args(&["--hard"]).unwrap(),
            Args {
                yes: true,
                ..Args::default()
            }
        );
        assert_eq!(
            parse_args(&["--rep"]).unwrap(),
            Args {
                repeat: true,
                ..Args::default()
            }
        );
    }

    #[test]
    fn parses_alias() {
        let alias = |name: &str| Args {
            alias: Some(name.to_string()),
            ..Args::default()
        };
        assert_eq!(parse_args(&["-a"]).unwrap(), alias(&get_alias()));
        assert_eq!(parse_args(&["--alias", "fix"]).unwrap(), alias("fix"));
        assert_eq!(parse_args(&["--alias=fix"]).unwrap(), alias("fix"));
        assert_eq!(
            parse_args(&["-a", "--enable-experimental-instant-mode"]).unwrap(),
            Args {
                enable_experimental_instant_mode: true,
                ..alias(&get_alias())
            }
        );
        assert_eq!(
            parse_args(&["-l", "/tmp/log"])
                .unwrap()
                .shell_logger
                .unwrap(),
            "/tmp/log"
        );
    }

    #[test]
    fn parses_command() {
        assert_eq!(
            parse_args(&["git", "branch", "-a"]).unwrap().command,
            strings(&["git", "branch", "-a"])
        );
        assert_eq!(
            parse_args(&["git", "branch", "-a", ARGUMENT_PLACEHOLDER, "-y"]).unwrap(),
            Args {
                yes: true,
                command: strings(&["git", "branch", "-a"]),
                ..Args::default()
            }
        );
        assert_eq!(
            parse_args(&["ls", "-r", ARGUMENT_PLACEHOLDER, "-r", "-d"]).unwrap(),
            Args {
                repeat: true,
                debug: true,
                command: strings(&["ls", "-r"]),
                ..Args::default()
            }
        );
        assert_eq!(
            parse_args(&[ARGUMENT_PLACEHOLDER]).unwrap(),
            Args::default()
        );
        assert_eq!(
            parse_args(&["--force-command", "ls -la"])
                .unwrap()
                .force_command
                .unwrap(),
            "ls -la"
        );
    }

    #[test]
    fn rejects_wrong_arguments() {
        assert_eq!(
            parse_args(&["-y", "-r"]).unwrap_err(),
            "argument -r/--repeat: not allowed with argument -y/--yes/--yeah/--hard"
        );
        assert_eq!(
            parse_args(&["--bogus"]).unwrap_err(),
            "unrecognized arguments: --bogus"
        );
        assert_eq!(
            parse_args(&["-x"]).unwrap_err(),
            "unrecognized arguments: -x"
        );
        assert_eq!(
            parse_args(&["-l"]).unwrap_err(),
            "argument -l/--shell-logger: expected one argument"
        );
    }
}
