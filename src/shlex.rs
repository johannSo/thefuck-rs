//! Shell-like splitting and quoting, following the POSIX mode of Python's
//! `shlex` module, which the rules were written against.

/// Reasons a string can't be split.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SplitError {
    NoClosingQuotation,
    NoEscapedCharacter,
}

/// Splits the string using shell-like syntax.
pub fn split(s: &str) -> Result<Vec<String>, SplitError> {
    #[derive(Clone, Copy, PartialEq)]
    enum State {
        Whitespace,
        Word,
        Quote(char),
        /// Holds the state to return to after the escaped character.
        Escape(Option<char>),
    }

    let mut tokens = Vec::new();
    let mut token = String::new();
    // A quoted token is emitted even when it's empty.
    let mut quoted = false;
    let mut state = State::Whitespace;

    for ch in s.chars() {
        state = match state {
            State::Whitespace | State::Word => match ch {
                ' ' | '\t' | '\r' | '\n' => {
                    if state == State::Word && (!token.is_empty() || quoted) {
                        tokens.push(std::mem::take(&mut token));
                        quoted = false;
                    }
                    State::Whitespace
                }
                '\\' => State::Escape(None),
                '\'' | '"' => {
                    quoted = true;
                    State::Quote(ch)
                }
                _ => {
                    token.push(ch);
                    State::Word
                }
            },
            State::Quote(quote) => {
                if ch == quote {
                    State::Word
                } else if ch == '\\' && quote == '"' {
                    State::Escape(Some(quote))
                } else {
                    token.push(ch);
                    state
                }
            }
            State::Escape(quote) => {
                // In quotes a backslash only escapes the quote and itself.
                if let Some(quote) = quote {
                    if ch != '\\' && ch != quote {
                        token.push('\\');
                    }
                }
                token.push(ch);
                quote.map_or(State::Word, State::Quote)
            }
        };
    }

    match state {
        State::Quote(_) => Err(SplitError::NoClosingQuotation),
        State::Escape(_) => Err(SplitError::NoEscapedCharacter),
        State::Whitespace | State::Word => {
            if !token.is_empty() || quoted {
                tokens.push(token);
            }
            Ok(tokens)
        }
    }
}

/// Returns a shell-escaped version of the string.
pub fn quote(s: &str) -> String {
    if s.is_empty() {
        return "''".to_string();
    }
    let is_safe = |c: char| c.is_ascii_alphanumeric() || "_@%+=:,./-".contains(c);
    if s.chars().all(is_safe) {
        return s.to_string();
    }
    // Use single quotes, and put single quotes into double quotes:
    // the string $'b is then quoted as '$'"'"'b'
    format!("'{}'", s.replace('\'', "'\"'\"'"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ok(s: &str) -> Vec<String> {
        split(s).unwrap()
    }

    #[test]
    fn splits_like_a_shell() {
        assert_eq!(
            ok("git commit -m 'a b'  \"c d\""),
            ["git", "commit", "-m", "a b", "c d"]
        );
        assert_eq!(ok("  ls\t-la\n"), ["ls", "-la"]);
        assert_eq!(ok("echo ''"), ["echo", ""]);
        assert_eq!(ok("echo a''b \"\"c"), ["echo", "ab", "c"]);
        assert_eq!(ok(r"a\ b c\\d"), ["a b", r"c\d"]);
        assert_eq!(ok(r#""a\"b" "a\nb" 'a\b'"#), ["a\"b", r"a\nb", r"a\b"]);
        assert_eq!(ok("#not a comment"), ["#not", "a", "comment"]);
        assert_eq!(ok(""), Vec::<String>::new());
    }

    #[test]
    fn reports_unbalanced_input() {
        assert_eq!(split("echo 'a"), Err(SplitError::NoClosingQuotation));
        assert_eq!(split("echo \"a"), Err(SplitError::NoClosingQuotation));
        assert_eq!(split("echo a\\"), Err(SplitError::NoEscapedCharacter));
    }

    #[test]
    fn quotes_when_needed() {
        assert_eq!(quote(""), "''");
        assert_eq!(quote("a-b_c/d.e:f"), "a-b_c/d.e:f");
        assert_eq!(quote("a b"), "'a b'");
        assert_eq!(quote("it's"), "'it'\"'\"'s'");
        assert_eq!(quote("ö"), "'ö'");
    }
}
