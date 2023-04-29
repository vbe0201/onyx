use std::path::PathBuf;

use xshell::{cmd, Shell};

use crate::{config::Config, rustc};

/// Launches a QEMU instance emulating the kernel image at a given path.
///
/// Uses the build [`Config`] to retrieve additional arguments to pass
/// to QEMU.
pub fn run_in_qemu(sh: &Shell, image: PathBuf, config: &Config) -> anyhow::Result<()> {
    let _cwd = sh.push_dir(rustc::project_root());

    let (system, extra_args) = (&config.qemu.name, &config.qemu.extra_args);
    cmd!(
        sh,
        "qemu-system-{system}
            {extra_args...}
            -kernel {image}"
    )
    .run()?;

    Ok(())
}
