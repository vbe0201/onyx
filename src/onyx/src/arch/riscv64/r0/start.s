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

// Loads the absolute address of a label into a register given
// a register containing a relative base address. Using this
// ensures that the kernel can execute from anywhere in memory.
.macro LOAD_LABEL_ADDR reg, base, symbol
    lla \reg, \symbol
    ld \reg, 0(\reg)
    add \reg, \base, \reg
.endm

// fn __onyx_bootstrap_kernel(...)
//
// This is responsible for bootstrapping the Kernel on hart 0.
//
.section .r0.text, "ax", %progbits
.global __onyx_bootstrap_kernel
.type __onyx_bootstrap_kernel, %function
__onyx_bootstrap_kernel:
    // Disable all interrupts.
    csrw sie, zero
    csrci sstatus, 2

    // Stash away kernel arguments for later use.
    mv s0, a0
    mv s1, a1

    // Disable the MMU for physical addressing.
    csrr t0, satp
    li t1, 0x0FFFFFFFFFFFFFFF
    and t0, t0, t1
    csrw satp, t0

    // Flush all caches for coherency and predictability.
    // TODO

    // Compute the Kernel Loader entrypoint in memory and call
    // it with the following arguments:
    //
    //   - a0: The kernel base address in memory.
    //   - a1: A pointer to the KernelLayout structure.
    //   - a2: A pointer to the embedded KIP1 list.
    //
    // Loader state will be returned in a0 for us to re-use.
    lla a0, __onyx_start
    lla a1, __onyx_kernel_layout
    LOAD_LABEL_ADDR a2, a0, __onyx_kip1_base
    LOAD_LABEL_ADDR t0, a0, __onyx_kernel_loader_base
    jalr ra, t0, 0
