use std::sync::OnceLock;

use crate::utils::{run, which};

pub fn brew_available() -> bool {
    which("brew").is_some()
}

/// To get brew path
pub fn get_brew_path_prefix() -> Option<&'static str> {
    static PREFIX: OnceLock<Option<String>> = OnceLock::new();
    PREFIX
        .get_or_init(|| {
            let output = run("brew", &["--prefix"]);
            output.success.then(|| output.stdout.trim().to_string())
        })
        .as_deref()
}
