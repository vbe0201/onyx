use std::{
    fs,
    io::{Cursor, Seek, SeekFrom, Write},
    path::Path,
};

use anyhow::bail;
use binrw::{binrw, BinRead, BinWrite, Endian};
use flate2::{write::GzEncoder, Compression};
use memchr::memmem;

const PAGE_SIZE: usize = 0x1000;

/// The metadata magic of the Onyx kernel binary.
pub const KERNEL_MAGIC: [u8; 4] = *b"ONYX";

/// Representation of an Onyx Kernel Image.
///
/// A Kernel Image bundles the kernel itself, the kernel loader binary
/// and a list of initial processes into a single, self-contained binary
/// blob for distribution.
///
/// When uncompressed, the resulting image file can be directly executed
/// from its start after loading it into memory.
pub struct KernelImage {
    endian: Endian,
    compress: bool,

    kernel: Vec<u8>,
    kernel_meta: (usize, KernelMeta),

    loader: Vec<u8>,

    version: u32,
}

impl KernelImage {
    /// Creates a new, empty kernel image.
    pub fn new() -> Self {
        Self {
            endian: Endian::Little,
            compress: false,

            kernel: Vec::new(),
            kernel_meta: (0, KernelMeta::default()),

            loader: Vec::new(),

            version: 0,
        }
    }

    /// Configures the endianness to use for encoding data.
    pub fn with_endian(mut self, endian: Endian) -> Self {
        self.endian = endian;
        self
    }

    /// Configures a version for the Onyx release.
    pub fn with_version(mut self, major: u8, minor: u8, patch: u8) -> Self {
        self.version = ((major as u32) << 24) | ((minor as u32) << 16) | ((patch as u32) << 8);
        self
    }

    /// Configures whether the kernel image should be compressed.
    pub fn with_compression(mut self, enable: bool) -> Self {
        self.compress = enable;
        self
    }

    /// Packs an `onyx` binary into the kernel image.
    ///
    /// This must always be provided before calling [`KernelImage::finish`].
    pub fn pack_kernel<P: AsRef<Path>>(mut self, kernel: P) -> anyhow::Result<Self> {
        let kernel = fs::read(kernel)?;

        // We try to find the metadata offset for the kernel first.
        // However ,it must not be at 0 because the image needs to
        // start with executable code. At the same time, it is fair
        // to assume it's a logic bug when metadata are *too* far in.
        let finder = memmem::Finder::new(&KERNEL_MAGIC);
        let meta_offset = match finder.find(&kernel) {
            Some(off) if off == 0 || off > 0x10 => {
                bail!("suspicious metadata offset found; please confirm");
            }
            Some(off) => off,
            None => bail!("Malformed kernel binary!"),
        };

        // Now deserialize the kernel meta blob.
        let mut cursor = Cursor::new(&kernel[meta_offset..]);
        let meta = KernelMeta::read_options(&mut cursor, self.endian, ())?;

        // Confirm some assertions about the memory layout of the kernel.
        assert!(kernel.len() <= meta.layout.kernel_end as usize);
        assert!(meta.layout.text_start <= meta.layout.text_end);
        assert!(meta.layout.rodata_start <= meta.layout.rodata_end);
        assert!(meta.layout.data_start <= meta.layout.data_end);
        assert!(meta.layout.bss_start <= meta.layout.bss_end);

        // Store the kernel along with its meta.
        self.kernel = kernel;
        self.kernel_meta = (meta_offset, meta);

        Ok(self)
    }

    /// Packs an `onyx-loader` binary into the image.
    ///
    /// This must always be provided before calling [`KernelImage::finish`].
    pub fn pack_loader<P: AsRef<Path>>(mut self, loader: P) -> anyhow::Result<Self> {
        let loader = fs::read(loader)?;
        self.loader = loader;

        Ok(self)
    }

    /// Finishes the image construction and writes the resulting blob to
    /// the given output path.
    pub fn finish<P: AsRef<Path>>(mut self, out: P) -> anyhow::Result<()> {
        if self.kernel_meta.0 == 0 || self.loader.is_empty() {
            bail!("cannot build kernel image without Kernel or Loader");
        }

        // Calculate the start and end offsets of the Kernel Loader.
        let loader_start = align_up(self.kernel.len(), PAGE_SIZE);
        let loader_end = loader_start + self.loader.len();

        // Update our header accordingly.
        self.kernel_meta.1.loader_base = loader_start as u64;
        self.kernel_meta.1.version = self.version;

        // Now build the resulting image blob.
        let mut image = Cursor::new(Vec::new());
        {
            // Write the initial bits of kernel code.
            image.write_all(&self.kernel[..self.kernel_meta.0])?;

            // Re-serialize the kernel metadata.
            self.kernel_meta
                .1
                .write_options(&mut image, self.endian, ())?;

            // Write the rest of the kernel code.
            image.write_all(&self.kernel[(self.kernel_meta.0 + self.kernel_meta.1.size())..])?;

            // Write the Kernel Loader code.
            image.seek(SeekFrom::Start(loader_start as u64))?;
            image.write_all(&self.loader)?;

            // Append trailing padding at an aligned image end.
            image.seek(SeekFrom::Start(align_up(loader_end, PAGE_SIZE) as u64))?;
            image.write_all(&[0; PAGE_SIZE])?;
        }

        // Write the image to the output file.
        let mut output = fs::OpenOptions::new().write(true).create(true).open(out)?;
        if self.compress {
            let mut encoder = GzEncoder::new(&mut output, Compression::default());
            encoder.write_all(image.get_ref())?;
            encoder.finish()?;
        } else {
            output.write_all(image.get_ref())?;
        }

        Ok(())
    }
}

/// Encoded kernel metadata.
#[derive(Debug, Default)]
#[binrw]
#[brw(magic = b"ONYX")]
pub struct KernelMeta {
    /// The offset to the serialized KIP1 blob which holds all
    /// Kernel Initial Processes.
    pub kip1_base: u64,
    /// The base address of the Kernel Loader binary.
    pub loader_base: u64,
    /// The current kernel version.
    pub version: u32,
    /// The memory layout of the kernel binary.
    pub layout: KernelLayout,
}

impl KernelMeta {
    #[inline]
    fn size(&self) -> usize {
        use std::mem::size_of;

        // (kip1_base + loader_base) + (magic + version) + KernelLayout
        (size_of::<u64>() * 2) + (size_of::<u32>() * 2) + (size_of::<u32>() * 10)
    }
}

/// The memory layout of the kernel binary.
#[derive(Debug, Default)]
#[binrw]
pub struct KernelLayout {
    /// Start of the kernel .text section.
    pub text_start: u32,
    /// End of the kernel .text section.
    pub text_end: u32,
    /// Start of the kernel .rodata section.
    pub rodata_start: u32,
    /// End of the kernel .rodata section.
    pub rodata_end: u32,
    /// Start of the kernel .data section.
    pub data_start: u32,
    /// End of the kernel .data section.
    pub data_end: u32,
    /// Start of the kernel .bss section.
    pub bss_start: u32,
    /// End of the kernel .bss section.
    pub bss_end: u32,
    /// End of the kernel blob.
    pub kernel_end: u32,
    /// Start of the _DYNAMIC section.
    pub dynamic_start: u32,
}

#[inline(always)]
const fn align_up(value: usize, align: usize) -> usize {
    debug_assert!(align.is_power_of_two());
    (value + align - 1) & !(align - 1)
}
