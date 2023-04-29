use std::path::PathBuf;

use anyhow::Result;
use xshell::{cmd, Shell};

/// Returns the path to the root of this cargo workspace.
pub fn project_root() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop();
    path.pop();
    path
}

/// Gets the path to the sysroot of the currently used rustc.
pub fn sysroot(sh: &Shell) -> Result<PathBuf> {
    let rustc = std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into());
    let output = cmd!(sh, "{rustc} --print sysroot").output()?;

    let sysroot = String::from_utf8(output.stdout)?;
    Ok(PathBuf::from(sysroot.trim()))
}

/// Gets the path to an LLVM binutil given its name.
pub fn llvm_binutil(sh: &Shell, name: &str) -> Result<PathBuf> {
    let mut path = sysroot(sh)?;

    path.push("lib");
    path.push("rustlib");
    path.push(rustc_version::version_meta()?.host);
    path.push("bin");
    path.push(format!("llvm-{name}"));

    Ok(path)
}
