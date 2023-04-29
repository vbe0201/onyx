use std::path::{Path, PathBuf};

use binrw::Endian;
use serde::{de::Deserializer, Deserialize};

/// The build configuration for an Onyx distribution.
///
/// This defines build targets, the individual pieces of software
/// to build, and some customization options related to the inner
/// workings of the Kernel and the resulting Kernel Image.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    /// The path to the target description file in JSON format.
    ///
    /// Paths are expected to be absolute or relative to the
    /// project root.
    pub target: PathBuf,
    /// The endianness used by the target architecture.
    ///
    /// Defaults to little endian.
    #[serde(deserialize_with = "deserialize_endian", default = "little_endian")]
    pub endian: Endian,
    /// The board the kernel is building for, if any.
    ///
    /// Will be passed as a feature flag to cargo during the build
    /// to conditionally compile board-specific code.
    ///
    /// This is useful for specialized targets and keeps Onyx
    /// modular for porting.
    ///
    /// Defaults to the `"generic"` board which does not pull in
    /// any board-specific peripheral code.
    #[serde(default = "generic_board")]
    pub board: String,
    /// Image-related configuration.
    pub image: Image,
    /// Kernel-specific build configuration.
    pub kernel: Kernel,
    /// Loader-specific build configuration.
    pub loader: Loader,
    /// QEMU configuration for testing.
    pub qemu: Qemu,
}

impl Config {
    /// Reads a configuration file in TOML format from the given path.
    pub fn read<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let config = std::fs::read_to_string(path)?;
        toml::from_str(&config).map_err(Into::into)
    }
}

/// Build configuration for the final Kernel Image blob.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Image {
    /// Whether the resulting image should be compressed.
    ///
    /// The image will be compressed with gzip if this is
    /// set to `true`.
    ///
    /// Defaults to `false`.
    #[serde(default = "dont_compress")]
    pub compress: bool,
}

/// Build configuration for the `onyx` kernel application.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Kernel {
    /// Path to the linker script to use for building the kernel.
    ///
    /// Paths are expected to be absolute or relative to the
    /// project root.
    pub linker_script: PathBuf,
}

/// Build configuration for the `onyx-loader` kernel loader application.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Loader {
    /// Path to the linker script to use for building the loader.
    ///
    /// Paths are expected to be absolute or relative to the
    /// project root.
    pub linker_script: PathBuf,
}

/// QEMU configuration for testing Onyx builds through emulation.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Qemu {
    /// The name of the QEMU executable to use.
    pub name: String,
    /// Extra arguments to pass to a QEMU invocation.
    pub extra_args: Vec<String>,
}

fn deserialize_endian<'de, D: Deserializer<'de>>(de: D) -> Result<Endian, D::Error> {
    let endian: &str = Deserialize::deserialize(de)?;
    match endian {
        "little" => Ok(Endian::Little),
        "big" => Ok(Endian::Big),
        _ => Err(serde::de::Error::custom(
            "expected `little` or `big` for platform endianness",
        )),
    }
}

fn generic_board() -> String {
    "generic".to_string()
}

fn dont_compress() -> bool {
    false
}

fn little_endian() -> Endian {
    Endian::Little
}
