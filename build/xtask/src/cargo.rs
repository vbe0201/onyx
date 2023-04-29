use std::{
    io::{self, BufReader, Write},
    path::PathBuf,
    process::{Command, Stdio},
};

use cargo_metadata::Message;

use crate::{config::Config, rustc};

/// Runs a `cargo` subcommand based on the given build configuration
/// and obtains its stdout.
pub fn subcommand(
    subcommand: &str,
    pkg: &str,
    config: &Config,
    release: bool,
    verbose: bool,
) -> anyhow::Result<Vec<u8>> {
    // Prepare cargo flags to pass to the command.
    let release_arg = if release { &["--release"][..] } else { &[] };
    let verbose_arg = if verbose { &["--verbose"][..] } else { &[] };

    // Build the path to the linker script to utilize.
    let mut linker_script = PathBuf::new();
    linker_script.push("build");
    linker_script.push("linker-scripts");
    linker_script.push(if pkg == "onyx" {
        &config.kernel.linker_script
    } else {
        &config.loader.linker_script
    });

    // Run the cargo command with all relevant build options set.
    // XXX: Can't use xshell because they don't give stdout on command error.
    let output = Command::new("cargo")
        .arg(subcommand)
        .args(release_arg)
        .args(verbose_arg)
        .args(["-p", pkg])
        .arg("--target")
        .arg(&config.target)
        .args(["--features", &config.board])
        .args(["-Z", "build-std=core,alloc,compiler_builtins"])
        .args(["-Z", "build-std-features=compiler-builtins-mem"])
        .arg("--message-format=json-diagnostic-rendered-ansi")
        .env(
            "RUSTFLAGS",
            format!(
                "-C relocation-model=pic -C link-arg=--pie -C link-arg=-T{}",
                linker_script.display()
            ),
        )
        .current_dir(rustc::project_root())
        .stdout(Stdio::piped())
        .spawn()?
        .wait_with_output()?;

    // Return the stdout buffer we extracted.
    Ok(output.stdout)
}

/// Parses `cargo` output and returns a list of build artifacts,
/// while printing every message to stdout.
///
/// This should be used with [`subcommand`] so that diagnostics
/// are not lost while driving cargo through this build script.
pub fn parse_print_output(output: &[u8]) -> anyhow::Result<Vec<PathBuf>> {
    let mut artifacts = Vec::new();

    let stdout = io::stdout();
    let mut stdout = stdout.lock();

    let reader = BufReader::new(output);
    for message in Message::parse_stream(reader) {
        match message.unwrap() {
            Message::CompilerMessage(msg) => {
                if let Some(msg) = &msg.message.rendered {
                    stdout.write_all(msg.as_bytes())?;
                }
            }
            Message::CompilerArtifact(artifact) => {
                let message = format!("Compiled {}\n", &artifact.package_id.repr);
                stdout.write_all(message.as_bytes())?;

                if let Some(exe) = artifact.executable {
                    artifacts.push(PathBuf::from(exe));
                }
            }
            Message::BuildScriptExecuted(build) => {
                let message = format!("Build script executed ({:?})\n", build.out_dir);
                stdout.write_all(message.as_bytes())?;
            }
            Message::BuildFinished(res) if res.success => {
                stdout.write_all(b"Build completed successfully!\n")?;
            }
            Message::BuildFinished(_) => {
                stdout.write_all(b"Errors occured during build!\n")?;
            }
            Message::TextLine(s) => {
                // Unknown message content.
                stdout.write_all(s.as_bytes())?;
                stdout.write_all(b"\n")?;
            }

            // Unhandled message types.
            _ => (),
        }
    }

    Ok(artifacts)
}
