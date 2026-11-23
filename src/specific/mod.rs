//! Helpers for rules of specific applications.

pub mod archlinux;
pub mod brew;
pub mod git;
pub mod npm;
pub mod sudo;
pub mod url;

use crate::utils::which;

pub fn apt_available() -> bool {
    which("apt-get").is_some()
}

pub fn dnf_available() -> bool {
    which("dnf").is_some()
}

pub fn nix_available() -> bool {
    which("nix").is_some()
}

pub fn yum_available() -> bool {
    which("yum").is_some()
}
