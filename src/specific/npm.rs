use std::sync::OnceLock;

use crate::utils::{regex, run, which};

pub fn npm_available() -> bool {
    which("npm").is_some()
}

fn parse_scripts(output: &str) -> Vec<String> {
    let mut should_yield = false;
    let mut scripts = Vec::new();
    for line in output.split_inclusive('\n') {
        if line.contains("available via `npm run-script`:") {
            should_yield = true;
            continue;
        }

        if should_yield && regex(r"^  [^ ]+").is_match(line) {
            scripts.push(line.trim().split(' ').next().unwrap_or("").to_string());
        }
    }
    scripts
}

/// Get custom npm scripts.
pub fn get_scripts() -> &'static [String] {
    static SCRIPTS: OnceLock<Vec<String>> = OnceLock::new();
    SCRIPTS.get_or_init(|| parse_scripts(&run("npm", &["run-script"]).stdout))
}

#[cfg(test)]
mod tests {
    use super::parse_scripts;

    #[test]
    fn parses_scripts() {
        let output = "Lifecycle scripts included in code-view-web:\n  test\n    jest\n\n\
                      available via `npm run-script`:\n  build\n    cp node_modules/ace-builds/src-min/ -a resources/ace/ && webpack --progress --colors -p --config ./webpack.production.config.js\n  \
                      develop\n    cp node_modules/ace-builds/src/ -a resources/ace/ && webpack-dev-server --progress --colors\n  \
                      watch-test\n    jest --verbose --watch\n\n";
        assert_eq!(parse_scripts(output), ["build", "develop", "watch-test"]);
    }
}
