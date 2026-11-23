//! Constants shared by the whole application.

/// Keys the confirmation prompt understands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    Up,
    Down,
    CtrlC,
    CtrlN,
    CtrlP,
    Char(char),
}

/// Actions triggered by pressed keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Select,
    Abort,
    Previous,
    Next,
}

/// Marker used in `settings.rules` meaning "every rule enabled by default".
pub const ALL_ENABLED: &str = "<const: All rules enabled>";

pub const DEFAULT_PRIORITY: i64 = 1000;

pub const ENV_TO_ATTR: &[(&str, &str)] = &[
    ("THEFUCK_RULES", "rules"),
    ("THEFUCK_EXCLUDE_RULES", "exclude_rules"),
    ("THEFUCK_WAIT_COMMAND", "wait_command"),
    ("THEFUCK_REQUIRE_CONFIRMATION", "require_confirmation"),
    ("THEFUCK_NO_COLORS", "no_colors"),
    ("THEFUCK_DEBUG", "debug"),
    ("THEFUCK_PRIORITY", "priority"),
    ("THEFUCK_HISTORY_LIMIT", "history_limit"),
    ("THEFUCK_ALTER_HISTORY", "alter_history"),
    ("THEFUCK_WAIT_SLOW_COMMAND", "wait_slow_command"),
    ("THEFUCK_SLOW_COMMANDS", "slow_commands"),
    ("THEFUCK_REPEAT", "repeat"),
    ("THEFUCK_INSTANT_MODE", "instant_mode"),
    ("THEFUCK_NUM_CLOSE_MATCHES", "num_close_matches"),
    (
        "THEFUCK_EXCLUDED_SEARCH_PATH_PREFIXES",
        "excluded_search_path_prefixes",
    ),
];

/// Content of a freshly created `settings.py`.
///
/// The file keeps the syntax of the Python version (simple `name = literal`
/// assignments), so existing configuration keeps working.
pub const SETTINGS_TEMPLATE: &str = "# The Fuck settings file
#
# The rules are defined as in the example bellow:
#
# rules = ['cd_parent', 'git_push', 'python_command', 'sudo']
#
# The default values are as follows. Uncomment and change to fit your needs.
# See https://github.com/nvbn/thefuck#settings for more information.
#

# rules = [<const: All rules enabled>]
# exclude_rules = []
# wait_command = 3
# require_confirmation = True
# no_colors = False
# debug = False
# priority = {}
# history_limit = None
# alter_history = True
# wait_slow_command = 15
# slow_commands = ['lein', 'react-native', 'gradle', './gradlew', 'vagrant']
# repeat = False
# instant_mode = False
# num_close_matches = 3
# env = {'LC_ALL': 'C', 'LANG': 'C', 'GIT_TRACE': '1'}
# excluded_search_path_prefixes = []
";

pub const ARGUMENT_PLACEHOLDER: &str = "THEFUCK_ARGUMENT_PLACEHOLDER";

/// Ten zero-width spaces, used to find user commands in the instant mode log.
pub const USER_COMMAND_MARK: &str =
    "\u{200B}\u{200B}\u{200B}\u{200B}\u{200B}\u{200B}\u{200B}\u{200B}\u{200B}\u{200B}";

pub const LOG_SIZE_IN_BYTES: usize = 1024 * 1024;

pub const LOG_SIZE_TO_CLEAN: usize = 10 * 1024;

pub const DIFF_WITH_ALIAS: f64 = 0.5;

pub const SHELL_LOGGER_SOCKET_ENV: &str = "SHELL_LOGGER_SOCKET";

pub const SHELL_LOGGER_LIMIT: usize = 5;
