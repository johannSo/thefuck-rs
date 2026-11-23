//! Application settings.
//!
//! Settings are assembled from the defaults, `settings.py` in the user
//! directory, `THEFUCK_*` environment variables and command line arguments,
//! in this order.
//!
//! `settings.py` keeps the name and the syntax it has in the Python version of
//! The Fuck. It isn't executed, only simple `name = literal` assignments are
//! understood, which covers everything the documentation ever suggested to put
//! there.

use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::PathBuf;
use std::sync::OnceLock;

use crate::argument_parser::Args;
use crate::consts::{ALL_ENABLED, ENV_TO_ATTR, SETTINGS_TEMPLATE};
use crate::logs;
use crate::utils::expanduser;

#[derive(Debug, Clone, PartialEq)]
pub struct Settings {
    pub rules: Vec<String>,
    pub exclude_rules: Vec<String>,
    pub wait_command: f64,
    pub require_confirmation: bool,
    pub no_colors: bool,
    pub debug: bool,
    pub priority: HashMap<String, i64>,
    pub history_limit: Option<usize>,
    pub alter_history: bool,
    pub wait_slow_command: f64,
    pub slow_commands: Vec<String>,
    pub repeat: bool,
    pub instant_mode: bool,
    pub num_close_matches: usize,
    pub env: BTreeMap<String, String>,
    pub excluded_search_path_prefixes: Vec<String>,
    pub user_dir: PathBuf,
}

impl Default for Settings {
    fn default() -> Self {
        let strings = |items: &[&str]| items.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        Settings {
            rules: strings(&[ALL_ENABLED]),
            exclude_rules: vec![],
            wait_command: 3.0,
            require_confirmation: true,
            no_colors: false,
            debug: false,
            priority: HashMap::new(),
            history_limit: None,
            alter_history: true,
            wait_slow_command: 15.0,
            slow_commands: strings(&["lein", "react-native", "gradle", "./gradlew", "vagrant"]),
            repeat: false,
            instant_mode: false,
            num_close_matches: 3,
            env: [("LC_ALL", "C"), ("LANG", "C"), ("GIT_TRACE", "1")]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            excluded_search_path_prefixes: vec![],
            user_dir: PathBuf::new(),
        }
    }
}

static SETTINGS: OnceLock<Settings> = OnceLock::new();
static DEFAULT_SETTINGS: OnceLock<Settings> = OnceLock::new();

/// Returns the global settings: the defaults until [`init`] is called.
pub fn settings() -> &'static Settings {
    SETTINGS
        .get()
        .unwrap_or_else(|| DEFAULT_SETTINGS.get_or_init(Settings::default))
}

/// Fills the global settings with values from `settings.py`, env and args.
pub fn init(args: Option<&Args>) {
    let mut settings = Settings::default();
    let mut problems = Vec::new();

    settings.user_dir = get_user_dir_path(&mut problems);
    if let Err(error) = fs::create_dir_all(&settings.user_dir) {
        problems.push(("Can't create config directory", error.to_string()));
    }
    let settings_path = settings.user_dir.join("settings.py");
    if !settings_path.is_file() {
        if let Err(error) = fs::write(&settings_path, SETTINGS_TEMPLATE) {
            problems.push(("Can't create settings file", error.to_string()));
        }
    }

    let from_file = fs::read_to_string(&settings_path)
        .map_err(|error| error.to_string())
        .and_then(|source| settings_from_source(&settings, &source));
    match from_file {
        Ok(updated) => settings = updated,
        Err(error) => problems.push(("Can't load settings from file", error)),
    }

    match settings_from_env(&settings, |name| std::env::var(name).ok()) {
        Ok(updated) => settings = updated,
        Err(error) => problems.push(("Can't load settings from env", error)),
    }

    if let Some(args) = args {
        if args.yes {
            settings.require_confirmation = false;
        }
        if args.debug {
            settings.debug = true;
        }
        if args.repeat {
            settings.repeat = true;
        }
    }

    let has_python_rules = fs::read_dir(settings.user_dir.join("rules"))
        .map(|entries| {
            entries.flatten().any(|entry| {
                let name = entry.file_name().to_string_lossy().into_owned();
                name.ends_with(".py") && name != "__init__.py"
            })
        })
        .unwrap_or(false);

    // Report problems only now, so logging already respects `no_colors`.
    let _ = SETTINGS.set(settings);
    for (title, details) in problems {
        logs::exception(title, &details);
    }
    if has_python_rules {
        logs::debug("Rules written in Python aren't supported, ignoring the user rules directory");
    }
}

/// Returns the path of the user config directory.
fn get_user_dir_path(problems: &mut Vec<(&'static str, String)>) -> PathBuf {
    let xdg_config_home = std::env::var("XDG_CONFIG_HOME").unwrap_or_else(|_| "~/.config".into());
    let user_dir = expanduser(&xdg_config_home).join("thefuck");
    let legacy_user_dir = expanduser("~/.thefuck");

    // For backward compatibility use legacy '~/.thefuck' if it exists:
    if legacy_user_dir.is_dir() {
        problems.push((
            "Deprecated config path",
            format!(
                "Config path {} is deprecated. Please move to {}",
                legacy_user_dir.display(),
                user_dir.display()
            ),
        ));
        legacy_user_dir
    } else {
        user_dir
    }
}

/// Applies the assignments found in the source of `settings.py`.
pub fn settings_from_source(base: &Settings, source: &str) -> Result<Settings, String> {
    let mut settings = base.clone();
    for (name, value) in literal::parse_assignments(source)? {
        let wrong = |expected: &str| format!("`{}` should be {}", name, expected);
        match name.as_str() {
            "rules" => {
                settings.rules = value
                    .to_strings()
                    .ok_or_else(|| wrong("a list of strings"))?
            }
            "exclude_rules" => {
                settings.exclude_rules = value
                    .to_strings()
                    .ok_or_else(|| wrong("a list of strings"))?
            }
            "slow_commands" => {
                settings.slow_commands = value
                    .to_strings()
                    .ok_or_else(|| wrong("a list of strings"))?
            }
            "excluded_search_path_prefixes" => {
                settings.excluded_search_path_prefixes = value
                    .to_strings()
                    .ok_or_else(|| wrong("a list of strings"))?
            }
            "wait_command" => {
                settings.wait_command = value.to_f64().ok_or_else(|| wrong("a number"))?
            }
            "wait_slow_command" => {
                settings.wait_slow_command = value.to_f64().ok_or_else(|| wrong("a number"))?
            }
            "require_confirmation" => {
                settings.require_confirmation = value.to_bool().ok_or_else(|| wrong("a boolean"))?
            }
            "no_colors" => {
                settings.no_colors = value.to_bool().ok_or_else(|| wrong("a boolean"))?
            }
            "debug" => settings.debug = value.to_bool().ok_or_else(|| wrong("a boolean"))?,
            "alter_history" => {
                settings.alter_history = value.to_bool().ok_or_else(|| wrong("a boolean"))?
            }
            "repeat" => settings.repeat = value.to_bool().ok_or_else(|| wrong("a boolean"))?,
            "instant_mode" => {
                settings.instant_mode = value.to_bool().ok_or_else(|| wrong("a boolean"))?
            }
            "history_limit" => {
                settings.history_limit = match value {
                    literal::Value::None => None,
                    literal::Value::Int(limit) if limit >= 0 => Some(limit as usize),
                    _ => return Err(wrong("a non-negative integer or None")),
                }
            }
            "num_close_matches" => {
                settings.num_close_matches = match value {
                    literal::Value::Int(n) if n >= 0 => n as usize,
                    _ => return Err(wrong("a non-negative integer")),
                }
            }
            "priority" => {
                let pairs = value.to_pairs().ok_or_else(|| wrong("a dict"))?;
                settings.priority = pairs
                    .into_iter()
                    .map(|(rule, priority)| match (rule, priority) {
                        (literal::Value::Str(rule), literal::Value::Int(priority)) => {
                            Some((rule, priority))
                        }
                        _ => None,
                    })
                    .collect::<Option<_>>()
                    .ok_or_else(|| wrong("a dict of rule names to integers"))?;
            }
            "env" => {
                let pairs = value.to_pairs().ok_or_else(|| wrong("a dict"))?;
                settings.env = pairs
                    .into_iter()
                    .map(|(name, value)| match (name, value) {
                        (literal::Value::Str(name), literal::Value::Str(value)) => {
                            Some((name, value))
                        }
                        _ => None,
                    })
                    .collect::<Option<_>>()
                    .ok_or_else(|| wrong("a dict of strings to strings"))?;
            }
            // Other names are just variables of the settings file.
            _ => {}
        }
    }
    Ok(settings)
}

/// Transforms rules list from env-string to a list of rules.
fn rules_from_env(val: &str) -> Vec<String> {
    let rules: Vec<String> = val.split(':').map(String::from).collect();
    if rules.iter().any(|rule| rule == "DEFAULT_RULES") {
        std::iter::once(ALL_ENABLED.to_string())
            .chain(rules.into_iter().filter(|rule| rule != "DEFAULT_RULES"))
            .collect()
    } else {
        rules
    }
}

/// Gets priority pairs from env.
fn priority_from_env(val: &str) -> HashMap<String, i64> {
    val.split(':')
        .filter_map(|part| {
            let pieces: Vec<&str> = part.split('=').collect();
            match pieces[..] {
                [rule, priority] => Some((rule.to_string(), priority.trim().parse().ok()?)),
                _ => None,
            }
        })
        .collect()
}

/// Loads settings from env.
pub fn settings_from_env(
    base: &Settings,
    getenv: impl Fn(&str) -> Option<String>,
) -> Result<Settings, String> {
    let mut settings = base.clone();
    for (env, attr) in ENV_TO_ATTR {
        let Some(val) = getenv(env) else { continue };
        let invalid = || format!("invalid value for {}: {:?}", env, val);
        let is_true = val.to_lowercase() == "true";
        let split = || val.split(':').map(String::from).collect::<Vec<_>>();
        match *attr {
            "rules" => settings.rules = rules_from_env(&val),
            "exclude_rules" => settings.exclude_rules = rules_from_env(&val),
            "priority" => settings.priority = priority_from_env(&val),
            "wait_command" => {
                settings.wait_command = val.trim().parse::<i64>().map_err(|_| invalid())? as f64
            }
            "wait_slow_command" => {
                settings.wait_slow_command =
                    val.trim().parse::<i64>().map_err(|_| invalid())? as f64
            }
            "history_limit" => {
                settings.history_limit = Some(val.trim().parse().map_err(|_| invalid())?)
            }
            "num_close_matches" => {
                settings.num_close_matches = val.trim().parse().map_err(|_| invalid())?
            }
            "require_confirmation" => settings.require_confirmation = is_true,
            "no_colors" => settings.no_colors = is_true,
            "debug" => settings.debug = is_true,
            "alter_history" => settings.alter_history = is_true,
            "instant_mode" => settings.instant_mode = is_true,
            "repeat" => settings.repeat = !val.is_empty(),
            "slow_commands" => settings.slow_commands = split(),
            "excluded_search_path_prefixes" => settings.excluded_search_path_prefixes = split(),
            _ => unreachable!("unknown setting {}", attr),
        }
    }
    Ok(settings)
}

/// A reader for the subset of Python that `settings.py` consists of.
pub mod literal {
    use std::collections::HashMap;

    use crate::consts::ALL_ENABLED;

    #[derive(Debug, Clone, PartialEq)]
    pub enum Value {
        None,
        Bool(bool),
        Int(i64),
        Float(f64),
        Str(String),
        List(Vec<Value>),
        Dict(Vec<(Value, Value)>),
    }

    impl Value {
        pub fn to_strings(&self) -> Option<Vec<String>> {
            match self {
                Value::List(items) => items
                    .iter()
                    .map(|item| match item {
                        Value::Str(s) => Some(s.clone()),
                        _ => None,
                    })
                    .collect(),
                _ => None,
            }
        }

        pub fn to_f64(&self) -> Option<f64> {
            match self {
                Value::Int(n) => Some(*n as f64),
                Value::Float(n) => Some(*n),
                _ => None,
            }
        }

        pub fn to_bool(&self) -> Option<bool> {
            match self {
                Value::Bool(b) => Some(*b),
                Value::Int(n) => Some(*n != 0),
                _ => None,
            }
        }

        pub fn to_pairs(&self) -> Option<Vec<(Value, Value)>> {
            match self {
                Value::Dict(pairs) => Some(pairs.clone()),
                _ => None,
            }
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    enum Token {
        Name(String),
        Str(String),
        Number(Value),
        Punct(char),
        Newline,
        End,
    }

    struct Lexer {
        chars: Vec<char>,
        pos: usize,
        line: usize,
        depth: usize,
    }

    impl Lexer {
        fn error<T>(&self, msg: &str) -> Result<T, String> {
            Err(format!("settings.py, line {}: {}", self.line, msg))
        }

        fn peek_char(&self, offset: usize) -> Option<char> {
            self.chars.get(self.pos + offset).copied()
        }

        fn next_token(&mut self) -> Result<Token, String> {
            loop {
                let Some(ch) = self.peek_char(0) else {
                    return Ok(Token::End);
                };
                match ch {
                    '#' => {
                        while self.peek_char(0).is_some_and(|c| c != '\n') {
                            self.pos += 1;
                        }
                    }
                    '\\' if self.peek_char(1) == Some('\n') => {
                        self.pos += 2;
                        self.line += 1;
                    }
                    '\n' => {
                        self.pos += 1;
                        self.line += 1;
                        if self.depth == 0 {
                            return Ok(Token::Newline);
                        }
                    }
                    c if c.is_whitespace() => self.pos += 1,
                    '\'' | '"' => return self.string(false),
                    c if c.is_ascii_digit() => return self.number(),
                    c if c.is_alphabetic() || c == '_' => {
                        let start = self.pos;
                        while self
                            .peek_char(0)
                            .is_some_and(|c| c.is_alphanumeric() || c == '_')
                        {
                            self.pos += 1;
                        }
                        let name: String = self.chars[start..self.pos].iter().collect();
                        let is_prefix =
                            name.len() <= 2 && name.chars().all(|c| "uUrRbBfF".contains(c));
                        if is_prefix && matches!(self.peek_char(0), Some('\'' | '"')) {
                            return self.string(name.to_lowercase().contains('r'));
                        }
                        return Ok(Token::Name(name));
                    }
                    _ => {
                        self.pos += 1;
                        match ch {
                            '[' | '(' | '{' => self.depth += 1,
                            ']' | ')' | '}' => self.depth = self.depth.saturating_sub(1),
                            _ => {}
                        }
                        return Ok(Token::Punct(ch));
                    }
                }
            }
        }

        fn number(&mut self) -> Result<Token, String> {
            let start = self.pos;
            while self
                .peek_char(0)
                .is_some_and(|c| c.is_ascii_alphanumeric() || c == '.' || c == '_')
            {
                self.pos += 1;
            }
            let text: String = self.chars[start..self.pos]
                .iter()
                .filter(|c| **c != '_')
                .collect();
            if let Ok(int) = text.parse::<i64>() {
                Ok(Token::Number(Value::Int(int)))
            } else if let Ok(float) = text.parse::<f64>() {
                Ok(Token::Number(Value::Float(float)))
            } else {
                self.error(&format!("invalid number {}", text))
            }
        }

        fn string(&mut self, raw: bool) -> Result<Token, String> {
            let quote = self.chars[self.pos];
            let triple = self.peek_char(1) == Some(quote) && self.peek_char(2) == Some(quote);
            self.pos += if triple { 3 } else { 1 };
            let mut result = String::new();
            loop {
                let Some(ch) = self.peek_char(0) else {
                    return self.error("unterminated string");
                };
                if ch == quote
                    && (!triple
                        || (self.peek_char(1) == Some(quote) && self.peek_char(2) == Some(quote)))
                {
                    self.pos += if triple { 3 } else { 1 };
                    return Ok(Token::Str(result));
                }
                self.pos += 1;
                if ch == '\n' {
                    if !triple {
                        return self.error("unterminated string");
                    }
                    self.line += 1;
                }
                if ch != '\\' {
                    result.push(ch);
                    continue;
                }
                let Some(escaped) = self.peek_char(0) else {
                    return self.error("unterminated string");
                };
                self.pos += 1;
                if raw {
                    result.push('\\');
                    result.push(escaped);
                    continue;
                }
                match escaped {
                    '\n' => self.line += 1,
                    'n' => result.push('\n'),
                    't' => result.push('\t'),
                    'r' => result.push('\r'),
                    '0' => result.push('\0'),
                    'a' => result.push('\x07'),
                    'b' => result.push('\x08'),
                    'f' => result.push('\x0c'),
                    'v' => result.push('\x0b'),
                    '\\' | '\'' | '"' => result.push(escaped),
                    'x' | 'u' | 'U' => {
                        let len = match escaped {
                            'x' => 2,
                            'u' => 4,
                            _ => 8,
                        };
                        let digits: String = self.chars.iter().skip(self.pos).take(len).collect();
                        let decoded = u32::from_str_radix(&digits, 16)
                            .ok()
                            .and_then(char::from_u32);
                        match decoded {
                            Some(decoded) if digits.chars().count() == len => result.push(decoded),
                            _ => return self.error("invalid escape sequence"),
                        }
                        self.pos += len;
                    }
                    other => {
                        result.push('\\');
                        result.push(other);
                    }
                }
            }
        }
    }

    struct Parser {
        lexer: Lexer,
        token: Token,
        variables: HashMap<String, Value>,
    }

    impl Parser {
        fn advance(&mut self) -> Result<Token, String> {
            let next = self.lexer.next_token()?;
            Ok(std::mem::replace(&mut self.token, next))
        }

        fn error<T>(&self, msg: &str) -> Result<T, String> {
            self.lexer.error(msg)
        }

        fn expect(&mut self, punct: char) -> Result<(), String> {
            if self.token == Token::Punct(punct) {
                self.advance()?;
                Ok(())
            } else {
                self.error(&format!("expected `{}`", punct))
            }
        }

        fn dotted_name(&mut self, first: String) -> Result<String, String> {
            let mut name = first;
            while self.token == Token::Punct('.') {
                self.advance()?;
                match self.advance()? {
                    Token::Name(part) => name = part,
                    _ => return self.error("expected a name"),
                }
            }
            Ok(name)
        }

        fn expression(&mut self) -> Result<Value, String> {
            let mut value = self.atom()?;
            while self.token == Token::Punct('+') {
                self.advance()?;
                let right = self.atom()?;
                value = match (value, right) {
                    (Value::List(mut left), Value::List(right)) => {
                        left.extend(right);
                        Value::List(left)
                    }
                    (Value::Str(left), Value::Str(right)) => Value::Str(left + &right),
                    (Value::Int(left), Value::Int(right)) => Value::Int(left + right),
                    _ => return self.error("unsupported operands for `+`"),
                };
            }
            Ok(value)
        }

        fn sequence(&mut self, close: char) -> Result<(Vec<Value>, bool), String> {
            let mut items = Vec::new();
            let mut has_comma = false;
            while self.token != Token::Punct(close) {
                items.push(self.expression()?);
                if self.token == Token::Punct(',') {
                    has_comma = true;
                    self.advance()?;
                } else {
                    break;
                }
            }
            self.expect(close)?;
            Ok((items, has_comma))
        }

        fn atom(&mut self) -> Result<Value, String> {
            match self.advance()? {
                Token::Str(mut string) => {
                    // Adjacent string literals are concatenated.
                    while let Token::Str(next) = &self.token {
                        string.push_str(next);
                        self.advance()?;
                    }
                    Ok(Value::Str(string))
                }
                Token::Number(number) => Ok(number),
                Token::Punct('-') => match self.atom()? {
                    Value::Int(n) => Ok(Value::Int(-n)),
                    Value::Float(n) => Ok(Value::Float(-n)),
                    _ => self.error("bad operand for unary `-`"),
                },
                Token::Punct('[') => Ok(Value::List(self.sequence(']')?.0)),
                Token::Punct('(') => {
                    let (mut items, has_comma) = self.sequence(')')?;
                    if items.len() == 1 && !has_comma {
                        Ok(items.remove(0))
                    } else {
                        Ok(Value::List(items))
                    }
                }
                Token::Punct('{') => {
                    let mut pairs = Vec::new();
                    let mut items = Vec::new();
                    while self.token != Token::Punct('}') {
                        let key = self.expression()?;
                        if self.token == Token::Punct(':') {
                            self.advance()?;
                            pairs.push((key, self.expression()?));
                        } else {
                            items.push(key);
                        }
                        if self.token == Token::Punct(',') {
                            self.advance()?;
                        } else {
                            break;
                        }
                    }
                    self.expect('}')?;
                    if items.is_empty() {
                        Ok(Value::Dict(pairs))
                    } else if pairs.is_empty() {
                        Ok(Value::List(items))
                    } else {
                        self.error("mixed dict and set syntax")
                    }
                }
                Token::Name(name) => {
                    let name = self.dotted_name(name)?;
                    match name.as_str() {
                        "True" => Ok(Value::Bool(true)),
                        "False" => Ok(Value::Bool(false)),
                        "None" => Ok(Value::None),
                        "DEFAULT_RULES" => Ok(Value::List(vec![Value::Str(ALL_ENABLED.into())])),
                        "ALL_ENABLED" => Ok(Value::Str(ALL_ENABLED.into())),
                        _ => match self.variables.get(&name) {
                            Some(value) => Ok(value.clone()),
                            None => self.error(&format!("name `{}` is not defined", name)),
                        },
                    }
                }
                _ => self.error("expected a value"),
            }
        }
    }

    /// Returns all top level `name = value` assignments of the source.
    pub fn parse_assignments(source: &str) -> Result<Vec<(String, Value)>, String> {
        let lexer = Lexer {
            chars: source.chars().collect(),
            pos: 0,
            line: 1,
            depth: 0,
        };
        let mut parser = Parser {
            lexer,
            token: Token::Newline,
            variables: HashMap::new(),
        };
        let mut assignments = Vec::new();
        loop {
            match parser.advance()? {
                Token::End => return Ok(assignments),
                Token::Newline | Token::Punct(';') => continue,
                Token::Name(keyword) if keyword == "import" || keyword == "from" => {
                    while !matches!(parser.token, Token::Newline | Token::End) {
                        parser.advance()?;
                    }
                }
                Token::Name(name) => {
                    let name = parser.dotted_name(name)?;
                    parser.expect('=')?;
                    let value = parser.expression()?;
                    if !matches!(
                        parser.token,
                        Token::Newline | Token::End | Token::Punct(';')
                    ) {
                        return parser.error("unsupported syntax");
                    }
                    parser.variables.insert(name.clone(), value.clone());
                    assignments.push((name, value));
                }
                _ => return parser.error("unsupported syntax"),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_settings_file_changes_nothing() {
        let base = Settings::default();
        assert_eq!(
            settings_from_source(&base, SETTINGS_TEMPLATE).unwrap(),
            base
        );
    }

    #[test]
    fn reads_settings_file() {
        let source = r#"
from thefuck.const import DEFAULT_RULES
# comment
rules = DEFAULT_RULES + ['rm_root',
                         "git_push_force"]  # trailing comment
exclude_rules = ('git_pull', 'git_push')
require_confirmation = False
wait_command = 10
no_colors = True
priority = {'sudo': 100, 'no_command': 9999}
history_limit = 9999
env = {'LANG': 'C', u'FOO': r'b\ar'}
slow_commands = ['react-native', 'gradle']
num_close_matches = 5
"#;
        let settings = settings_from_source(&Settings::default(), source).unwrap();
        assert_eq!(settings.rules, [ALL_ENABLED, "rm_root", "git_push_force"]);
        assert_eq!(settings.exclude_rules, ["git_pull", "git_push"]);
        assert!(!settings.require_confirmation);
        assert_eq!(settings.wait_command, 10.0);
        assert!(settings.no_colors);
        assert_eq!(settings.priority["sudo"], 100);
        assert_eq!(settings.priority["no_command"], 9999);
        assert_eq!(settings.history_limit, Some(9999));
        assert_eq!(settings.env["FOO"], "b\\ar");
        assert_eq!(settings.env.len(), 2);
        assert_eq!(settings.slow_commands, ["react-native", "gradle"]);
        assert_eq!(settings.num_close_matches, 5);
    }

    #[test]
    fn rejects_unsupported_settings_file() {
        assert!(settings_from_source(&Settings::default(), "rules = [x for x in y]").is_err());
        assert!(settings_from_source(&Settings::default(), "wait_command = 'long'").is_err());
        assert!(settings_from_source(&Settings::default(), "rules = ['sudo'").is_err());
    }

    #[test]
    fn reads_settings_from_env() {
        let env = |name: &str| {
            let value = match name {
                "THEFUCK_RULES" => "DEFAULT_RULES:bash:lisp",
                "THEFUCK_EXCLUDE_RULES" => "git:vim",
                "THEFUCK_WAIT_COMMAND" => "55",
                "THEFUCK_REQUIRE_CONFIRMATION" => "true",
                "THEFUCK_NO_COLORS" => "false",
                "THEFUCK_PRIORITY" => "bash=10:lisp=wrong:vim=15",
                "THEFUCK_WAIT_SLOW_COMMAND" => "999",
                "THEFUCK_SLOW_COMMANDS" => "lein:react-native:./gradlew",
                "THEFUCK_NUM_CLOSE_MATCHES" => "359",
                "THEFUCK_EXCLUDED_SEARCH_PATH_PREFIXES" => "/media/:/mnt/",
                _ => return None,
            };
            Some(value.to_string())
        };
        let settings = settings_from_env(&Settings::default(), env).unwrap();
        assert_eq!(settings.rules, [ALL_ENABLED, "bash", "lisp"]);
        assert_eq!(settings.exclude_rules, ["git", "vim"]);
        assert_eq!(settings.wait_command, 55.0);
        assert!(settings.require_confirmation);
        assert!(!settings.no_colors);
        assert_eq!(
            settings.priority,
            HashMap::from([("bash".into(), 10), ("vim".into(), 15)])
        );
        assert_eq!(settings.wait_slow_command, 999.0);
        assert_eq!(
            settings.slow_commands,
            ["lein", "react-native", "./gradlew"]
        );
        assert_eq!(settings.num_close_matches, 359);
        assert_eq!(settings.excluded_search_path_prefixes, ["/media/", "/mnt/"]);

        let broken = |name: &str| (name == "THEFUCK_WAIT_COMMAND").then(|| "soon".to_string());
        assert!(settings_from_env(&Settings::default(), broken).is_err());
    }
}
