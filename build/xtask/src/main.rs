use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};
use xshell::Shell;

mod build;

mod cargo;

mod check;

mod config;
use config::Config;

mod image;
use image::KernelImage;

mod run;

mod rustc;

#[derive(Parser)]
#[clap(long_about = None)]
struct Cli {
    #[clap(subcommand)]
    action: Action,
}

#[derive(Subcommand)]
enum Action {
    /// Builds a distribution of Onyx in the form of a full Kernel Image.
    ///
    /// This is the only supported format to execute Onyx from, and will
    /// be shipped in all release builds.
    Dist {
        /// Path to the build configuration file.
        #[clap(short, long)]
        config: PathBuf,
        /// Invokes cargo in release mode.
        #[clap(short, long)]
        release: bool,
        /// Request verbosity from all invoked tools.
        #[clap(short, long)]
        verbose: bool,
    },

    /// Attempts to `cargo build` a given package in the source tree.
    Build {
        /// The cargo package to build.
        package: String,
        /// Path to the build configuration file.
        #[clap(short, long)]
        config: PathBuf,
        /// Invokes cargo in release mode.
        #[clap(short, long)]
        release: bool,
        /// Request verbosity from all invoked tools.
        #[clap(short, long)]
        verbose: bool,
    },

    /// Attempts to `cargo check` a given package in the source tree.
    Check {
        /// The cargo package to check.
        package: String,
        /// Path to the build configuration file.
        #[clap(short, long)]
        config: PathBuf,
        /// Request verbosity from all invoked tools.
        #[clap(short, long)]
        verbose: bool,
    },

    /// Builds the Kernel Image and runs it in QEMU for testing.
    Run {
        /// Path to the build configuration file.
        #[clap(short, long)]
        config: PathBuf,
        /// Invokes cargo in release mode.
        #[clap(short, long)]
        release: bool,
    },
}

fn read_config<P: AsRef<Path>>(path: P) -> anyhow::Result<Config> {
    // Resolve the path to the supplied config file.
    let mut config_path = rustc::project_root();
    config_path.push("build");
    config_path.push("config");
    config_path.push(path);

    // Deserialize the config file.
    Config::read(config_path)
}

fn build_kernel_image(
    sh: &Shell,
    config: &Config,
    release: bool,
    verbose: bool,
) -> anyhow::Result<PathBuf> {
    // Build the kernel and convert it to a raw binary.
    let kernel = build::build("onyx", config, release, verbose)?;
    let kernel = build::make_raw_binary(sh, kernel)?;

    // Build the kernel loader and convert it to a raw binary.
    let kernel_loader = build::build("onyx-loader", config, release, verbose)?;
    let kernel_loader = build::make_raw_binary(sh, kernel_loader)?;

    // Build the output path for the kernel image.
    let image_path = {
        let mut path = rustc::project_root();
        path.push("target");
        path.push("dist");

        path = sh.create_dir(path)?;
        path.push("onyx.bin");

        path
    };

    KernelImage::new()
        .with_endian(config.endian)
        .with_compression(config.image.compress)
        .with_version(
            env!("CARGO_PKG_VERSION_MAJOR").parse()?,
            env!("CARGO_PKG_VERSION_MINOR").parse()?,
            env!("CARGO_PKG_VERSION_PATCH").parse()?,
        )
        .pack_kernel(kernel)?
        .pack_loader(kernel_loader)?
        .finish(&image_path)?;

    Ok(image_path)
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let shell = Shell::new()?;

    match cli.action {
        Action::Dist {
            config,
            release,
            verbose,
        } => {
            let config = read_config(config)?;
            build_kernel_image(&shell, &config, release, verbose).map(|_| ())
        }

        Action::Build {
            package,
            config,
            release,
            verbose,
        } => {
            let config = read_config(config)?;
            build::build(&package, &config, release, verbose).map(|_| ())
        }

        Action::Check {
            package,
            config,
            verbose,
        } => {
            let config = read_config(config)?;
            check::check(&package, &config, verbose)
        }

        Action::Run { config, release } => {
            let config = read_config(config)?;
            let image = build_kernel_image(&shell, &config, release, false)?;
            run::run_in_qemu(&shell, image, &config)
        }
    }
}
