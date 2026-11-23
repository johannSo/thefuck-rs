//! Minimal URL splitting, compatible with Python's `urllib.parse.urlparse`
//! for the parts the rules are interested in.

#[derive(Debug, PartialEq)]
pub struct Url {
    pub scheme: String,
    pub netloc: String,
    pub path: String,
}

pub fn urlparse(url: &str, default_scheme: &str) -> Url {
    let mut scheme = default_scheme.to_string();
    let mut rest = url;

    if let Some((candidate, after)) = url.split_once(':') {
        let is_scheme = candidate.starts_with(|c: char| c.is_ascii_alphabetic())
            && candidate
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || "+-.".contains(c));
        if is_scheme {
            scheme = candidate.to_lowercase();
            rest = after;
        }
    }

    let mut netloc = "";
    if let Some(after) = rest.strip_prefix("//") {
        let end = after.find(['/', '?', '#']).unwrap_or(after.len());
        netloc = &after[..end];
        rest = &after[end..];
    }

    let path = &rest[..rest.find('#').unwrap_or(rest.len())];
    let path = &path[..path.find('?').unwrap_or(path.len())];
    // Parameters of the last path segment aren't a part of the path.
    let last_segment = path.rfind('/').map_or(0, |slash| slash + 1);
    let path = match path[last_segment..].find(';') {
        Some(params) => &path[..last_segment + params],
        None => path,
    };

    Url {
        scheme,
        netloc: netloc.to_string(),
        path: path.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parts(url: &str, default_scheme: &str) -> (String, String, String) {
        let url = urlparse(url, default_scheme);
        (url.scheme, url.netloc, url.path)
    }

    #[test]
    fn parses_urls_like_python() {
        let tuple = |a: &str, b: &str, c: &str| (a.to_string(), b.to_string(), c.to_string());
        assert_eq!(
            parts("https://en.wikipedia.org/wiki/Main_Page", ""),
            tuple("https", "en.wikipedia.org", "/wiki/Main_Page")
        );
        assert_eq!(
            parts("en.wikipedia.org", ""),
            tuple("", "", "en.wikipedia.org")
        );
        assert_eq!(
            parts("git@github.com:nvbn/thefuck.git", "ssh"),
            tuple("ssh", "", "git@github.com:nvbn/thefuck.git")
        );
        assert_eq!(
            parts("ssh://git@github.com:22/nvbn/thefuck.git", "ssh"),
            tuple("ssh", "git@github.com:22", "/nvbn/thefuck.git")
        );
        assert_eq!(
            parts("HTTP://x.org/a;b?c#d", ""),
            tuple("http", "x.org", "/a")
        );
        assert_eq!(parts("notacommand", "ssh"), tuple("ssh", "", "notacommand"));
        assert_eq!(parts("foo:bar", "ssh"), tuple("foo", "", "bar"));
    }
}
