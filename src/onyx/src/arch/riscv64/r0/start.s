// Global entrypoint to the Kernel. This is the start of
// every Kernel Image and the first bit of executed code.
.section .r0.text.start, "ax", %progbits
.global __onyx_start
__onyx_start:
    j __onyx_bootstrap_kernel

// The Onyx KernelMeta structure. See `xbuild` for details.
// Make sure that the structure layout always matches the one
// found in both the build script and the `onyx-loader` crate.
__onyx_magic:
    .ascii "ONYX"
__onyx_kip1_base:
    .quad 0x0000000000000000
__onyx_kernel_loader_base:
    .quad 0x0000000000000000
__onyx_version:
    .word 0xFFFFFFFF
__onyx_kernel_layout:
    .word __onyx_start     - __onyx_start  // text_start
    .word __text_end__     - __onyx_start  // text_end
    .word __rodata_start__ - __onyx_start  // rodata_start
    .word __rodata_end__   - __onyx_start  // rodata_end
    .word __data_start__   - __onyx_start  // data_start
    .word __data_end__     - __onyx_start  // data_end
    .word __bss_start__    - __onyx_start  // bss_start
    .word __bss_end__      - __onyx_start  // bss_end
    .word __end__          - __onyx_start  // kernel_end
    .word _DYNAMIC         - __onyx_start  // dynamic_start

// fn __onyx_bootstrap_kernel(...)
//
// This is responsible for bootstrapping the Kernel on hart 0.
//
__onyx_bootstrap_kernel:
    j __onyx_bootstrap_kernel
