fn main() {
    // `fuck --update` works before the alias is configured too.
    if std::env::args().nth(1).as_deref() == Some("--update") {
        thefuck::entrypoints::update::main();
    } else {
        thefuck::entrypoints::not_configured::main();
    }
}
