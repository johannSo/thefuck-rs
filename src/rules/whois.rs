use super::prelude::*;
use crate::specific::url::urlparse;

/// What the `whois` command returns depends on the 'Whois server' it contacted
/// and is not consistent through different servers. But there can be only two
/// types of errors I can think of with `whois`:
///     - `whois https://en.wikipedia.org/` → `whois en.wikipedia.org`;
///     - `whois en.wikipedia.org` → `whois wikipedia.org`.
/// So we match any `whois` command and then:
///     - if there is a slash: keep only the FQDN;
///     - if there is no slash but there is a point: removes the left-most
///       subdomain.
///
/// We cannot either remove all subdomains because we cannot know which part is
/// the subdomains and which is the domain, consider:
///     - www.google.fr → subdomain: www, domain: 'google.fr';
///     - google.co.uk → subdomain: None, domain; 'google.co.uk'.
fn match_(command: &Command) -> bool {
    is_app(command, &["whois"], 1)
}

fn get_new_command(command: &Command) -> Vec<String> {
    let url = &command.script_parts()[1];

    if command.script.contains('/') {
        vec![format!("whois {}", urlparse(url, "").netloc)]
    } else if command.script.contains('.') {
        let path = urlparse(url, "").path;
        let path: Vec<&str> = path.split('.').collect();
        (1..path.len()).map(|n| format!("whois {}", path[n..].join("."))).collect()
    } else {
        vec![]
    }
}

pub const RULE: Rule = Rule::new("whois", match_, get_new_command);
