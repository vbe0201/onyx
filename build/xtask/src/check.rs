use crate::{cargo, config::Config};

/// Runs `cargo clippy` for the program and forwards all output to stdout.
pub fn check(pkg: &str, config: &Config, verbose: bool) -> anyhow::Result<()> {
    let output = cargo::subcommand("clippy", pkg, config, false, verbose)?;
    cargo::parse_print_output(&output).map(|_| ())
}
