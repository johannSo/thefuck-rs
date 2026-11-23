//! The Fuck: a magnificent app which corrects your previous console command.
//!
//! The crate mirrors the layout of the original Python package: `rules`
//! contains one module per correction rule, `shells` the shell integrations,
//! `output_readers` the ways of obtaining the output of the failed command and
//! `entrypoints` the things the binaries can do.

pub mod argument_parser;
pub mod conf;
pub mod consts;
pub mod corrector;
pub mod difflib;
pub mod entrypoints;
pub mod logs;
pub mod output_readers;
pub mod rules;
pub mod shells;
pub mod shlex;
pub mod specific;
pub mod system;
pub mod types;
pub mod ui;
pub mod utils;

/// Version of the application, without the cargo patch component so that it
/// lines up with the upstream release numbering (`3.32`).
pub fn version() -> &'static str {
    concat!(
        env!("CARGO_PKG_VERSION_MAJOR"),
        ".",
        env!("CARGO_PKG_VERSION_MINOR")
    )
}
