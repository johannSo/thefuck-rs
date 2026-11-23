//! `fuck --update` (or `thefuck --update`): updates The Fuck by installing the
//! latest version from its repository with cargo, to where it's installed now.
//!
//! Called through the alias, whatever is printed to stdout gets evaluated by
//! the shell, so everything, including the output of cargo, goes to stderr.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command as Process, Stdio};

use crate::conf;
use crate::logs;
use crate::utils::{home_dir, which};

/// Where updates come from, `THEFUCK_REPOSITORY` overrides it (for forks).
fn repository() -> String {
    std::env::var("THEFUCK_REPOSITORY")
        .ok()
        .filter(|repository| !repository.is_empty())
        .unwrap_or_else(|| env!("CARGO_PKG_REPOSITORY").to_string())
}

/// Returns the root of the cargo installation the binary belongs to: the
/// directory with `bin/` and the metadata where cargo tracks what it
/// installed. `None` when the binary wasn't installed by cargo, like a
/// development build or a distribution package.
fn cargo_install_root(exe: &Path) -> Option<PathBuf> {
    let bin_dir = exe.parent()?;
    if bin_dir.file_name()? != "bin" {
        return None;
    }
    let root = bin_dir.parent()?;
    let tracked = [".crates2.json", ".crates.toml"].iter().any(|metadata| {
        fs::read_to_string(root.join(metadata))
            .is_ok_and(|content| content.contains(&format!("\"{} ", env!("CARGO_PKG_NAME"))))
    });
    tracked.then(|| root.to_path_buf())
}

/// Like [`cargo_install_root`], but also follows symlinks: on macOS the path
/// of the running binary is the one it was started with, which can be a
/// symlink to the installed binary.
fn find_install_root(exe: &Path) -> Option<PathBuf> {
    cargo_install_root(exe).or_else(|| cargo_install_root(&fs::canonicalize(exe).ok()?))
}

/// Returns the cargo to update with. The one of the installation is preferred
/// (`~/.cargo/bin/cargo` for `~/.cargo`), it doesn't have to be in `PATH`:
/// it often isn't in shells of macOS started before rustup was installed.
fn find_cargo(root: &Path) -> Option<PathBuf> {
    let cargo_home = std::env::var_os("CARGO_HOME")
        .filter(|home| !home.is_empty())
        .map_or_else(|| home_dir().join(".cargo"), PathBuf::from);
    [root.join("bin/cargo"), cargo_home.join("bin/cargo")]
        .into_iter()
        .find(|cargo| cargo.is_file())
        .or_else(|| which("cargo").map(PathBuf::from))
}

/// On macOS cargo needs the linker of the Xcode Command Line Tools. Without
/// them building fails, or macOS pops up a dialog offering to install them.
fn has_build_tools() -> bool {
    !cfg!(target_os = "macos")
        || Process::new("xcode-select")
            .arg("-p")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .is_ok_and(|status| status.success())
}

/// Updates The Fuck, returns `true` on success.
fn update() -> bool {
    let exe = match std::env::current_exe() {
        Ok(exe) => exe,
        Err(error) => {
            logs::failed(&format!(
                "Can't find where The Fuck is installed: {}",
                error
            ));
            return false;
        }
    };
    let Some(root) = find_install_root(&exe) else {
        logs::failed(&format!(
            "{} wasn't installed with cargo, update it the way you installed it.",
            exe.display()
        ));
        return false;
    };
    let Some(cargo) = find_cargo(&root) else {
        logs::failed("cargo is required to update The Fuck, but it isn't installed.");
        return false;
    };
    if !has_build_tools() {
        logs::failed(
            "The Xcode Command Line Tools are required to build The Fuck, \
             install them with `xcode-select --install` and update again.",
        );
        return false;
    }

    let repository = repository();
    eprintln!(
        "Updating The Fuck {} from {}...",
        env!("CARGO_PKG_VERSION"),
        repository
    );
    // Without `--force` cargo rebuilds only when there's something new: a new
    // commit in the repository, or the installed one came from elsewhere.
    // cargo puts the new binary next to the old one and renames it over, so
    // the running binary is never modified in place; on macOS that would
    // invalidate its code signature and get it killed.
    let status = Process::new(&cargo)
        .args(["install", "--locked", "--git", &repository, "--root"])
        .arg(&root)
        .stdin(Stdio::null())
        .stdout(Stdio::from(std::io::stderr()))
        .status();
    match status {
        Ok(status) if status.success() => {
            eprintln!("The Fuck is up to date.");
            true
        }
        Ok(status) => {
            logs::failed(&format!("Updating failed, cargo exited with {}.", status));
            false
        }
        Err(error) => {
            logs::failed(&format!("Can't run {}: {}", cargo.display(), error));
            false
        }
    }
}

pub fn main() {
    conf::init(None);
    if !update() {
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn install_root(name: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!("thefuck-{}-{}", name, std::process::id()));
        fs::create_dir_all(root.join("bin")).unwrap();
        root
    }

    fn track_thefuck(root: &Path) {
        fs::write(
            root.join(".crates.toml"),
            "[v1]\n\"thefuck 3.32.0 (git+https://github.com/johannSo/thefuck-rs#3e7f50ab)\" = [\"fuck\", \"thefuck\"]\n",
        )
        .unwrap();
    }

    #[test]
    fn finds_cargo_install_root() {
        let root = install_root("update");
        let exe = root.join("bin/thefuck");

        assert_eq!(cargo_install_root(&exe), None);
        fs::write(
            root.join(".crates2.json"),
            r#"{"installs":{"ripgrep 14.1.0 (registry+https://github.com/rust-lang/crates.io-index)":{}}}"#,
        )
        .unwrap();
        assert_eq!(cargo_install_root(&exe), None);
        track_thefuck(&root);
        assert_eq!(cargo_install_root(&exe), Some(root.clone()));
        assert_eq!(cargo_install_root(&root.join("target/debug/thefuck")), None);

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn follows_symlinks_to_the_install_root() {
        // Like `/usr/local/bin/thefuck -> ~/.cargo/bin/thefuck` on macOS.
        let root = install_root("update-symlink");
        track_thefuck(&root);
        fs::write(root.join("bin/thefuck"), "").unwrap();
        let link_dir = root.join("elsewhere/bin");
        fs::create_dir_all(&link_dir).unwrap();
        std::os::unix::fs::symlink(root.join("bin/thefuck"), link_dir.join("thefuck")).unwrap();

        let canonical_root = fs::canonicalize(&root).unwrap();
        assert_eq!(cargo_install_root(&link_dir.join("thefuck")), None);
        assert_eq!(
            find_install_root(&link_dir.join("thefuck")),
            Some(canonical_root)
        );
        assert_eq!(
            find_install_root(&root.join("bin/thefuck")),
            Some(root.clone())
        );

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn prefers_cargo_of_the_installation() {
        let root = install_root("update-cargo");
        fs::write(root.join("bin/cargo"), "").unwrap();
        assert_eq!(find_cargo(&root), Some(root.join("bin/cargo")));
        let _ = fs::remove_dir_all(root);
    }
}
