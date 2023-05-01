.section .r0.text.start, "ax", %progbits
.global __onyx_loader_start
__onyx_loader_start:
    j __onyx_loader_entry

// Loads the absolute address of a label into a register given
// a register containing a relative base address. Using this
// ensures that the loader can execute from anywhere in memory.
.macro LOAD_LABEL_ADDR reg, base, symbol
    lla \reg, \symbol
    ld \reg, 0(\reg)
    add \reg, \base, \reg
.endm

//
// fn __onyx_loader_entry(kernel_base: *const u8, layout: *const KernelLayout, kips: *const ())
//
.section .r0.text, "ax", %progbits
.global __onyx_loader_entry
.type __onyx_loader_entry, %function
__onyx_loader_entry:
    lla t0, __onyx_loader_start
    LOAD_LABEL_ADDR t1, t0, __onyx_loader_bss_start
    LOAD_LABEL_ADDR t2, t0, __onyx_loader_bss_end

    // Clear every double word in .bss section.
0:
    bgeu t1, t2, 1f
    sd zero, 0(t1)
    addi t1, t1, 8
    j 0b

1:
    // Set the stack pointer to the end of the initialized .bss section.
    LOAD_LABEL_ADDR sp, t0, __onyx_loader_stack_top

    // Back up our arguments and the link register on the stack.
    addi sp, sp, -32
    sd a0, 0(sp)
    sd a1, 8(sp)
    sd a2, 16(sp)
    sd ra, 24(sp)

    // Enter Rust
    call main

    li a0, 'C'
    li a1, 0x10000000
    sw a0, 0(a1)

    2: j 2b


.balign 8
__onyx_loader_stack_top:
    .quad __stack_top__ - __onyx_loader_start
__onyx_loader_bss_start:
    .quad __bss_start__ - __onyx_loader_start
__onyx_loader_bss_end:
    .quad __bss_end__   - __onyx_loader_start
__onyx_loader_dynamic_start:
    .quad _DYNAMIC      - __onyx_loader_start
