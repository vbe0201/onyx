use std::path::PathBuf;

use anyhow::{anyhow, Result};
use xshell::{cmd, Shell};

use crate::{cargo, config::Config, rustc};

/// Builds a given cargo package in the source tree and returns the
/// path to the produced ELF binary.
pub fn build(pkg: &str, config: &Config, release: bool, verbose: bool) -> Result<PathBuf> {
    let output = cargo::subcommand("build", pkg, config, release, verbose)?;
    let artifacts = cargo::parse_print_output(&output)?;

    // Try to extract the produced ELF binary for successful builds.
    artifacts
        .into_iter()
        .next()
        .ok_or_else(|| anyhow!("failed to extract build artifact for package `{}`", pkg))
}

/// Converts a given ELF file into a raw binary using objcopy and
/// returns the new path.
pub fn make_raw_binary(sh: &Shell, elf: PathBuf) -> Result<PathBuf> {
    let mut output = elf.clone();
    output.set_extension("bin");

    let objcopy = rustc::llvm_binutil(sh, "objcopy")?;
    cmd!(sh, "{objcopy} -S -O binary {elf} {output}").run()?;

    Ok(output)
}
