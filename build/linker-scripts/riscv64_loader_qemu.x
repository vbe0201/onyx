OUTPUT_ARCH(riscv64imac)
ENTRY(__onyx_loader_start)

PHDRS {
    loader  PT_LOAD FLAGS(7);
    dynamic PT_DYNAMIC;
}

SECTIONS {
    PROVIDE(__start__ = 0);

    . = __start__;

    .r0 : {
        KEEP(*(.r0 .r0.*))
        . = ALIGN(8);
    } :loader

    .text : {
        *(.text .text.*)
        . = ALIGN(8);
    } :loader

    .plt : {
        *(.plt .plt.*)
        . = ALIGN(8);
    } :loader

    __rodata_start__ = .;

    .rodata : {
        *(.rodata .rodata.*)
    } :loader

    .hash     : { *(.hash)             } :loader
    .gnu.hash : { *(.gnu.hash)         } :loader
    .dynsym   : { *(.dynsym .dynsym.*) } :loader
    .dynstr   : { *(.dynstr .dynstr.*) } :loader
    .rela.dyn : { *(.rela.*)           } :loader

    .dynamic : {
        HIDDEN(__dynamic_start__ = .);
        *(.dynamic)
    } :loader :dynamic

    __rodata_end__ = .;

    __data_start__ = .;

    .data ALIGN(8) : {
        *(.data .data.*)
        SORT(CONSTRUCTORS)
    } :loader

    __got_start__ = .;

    .got : {
        *(.got)
        *(.igot)
    } :loader

    .got.plt : {
        *(.got.plt)
        *(.igot.plt)
    } :loader

    __got_end__ = .;

    __data_end__ = .;

    .bss ALIGN(16) : {
        HIDDEN(__bss_start__ = .);
        *(.bss .bss.*)
        *(COMMON)
        *(.dynbss)

        /* Reserve 4KiB of stack memory. */
        . = ALIGN(16);
        __stack_bottom__ = .;
        . += 0x1000;
        __stack_top__ = .;
        HIDDEN(__bss_end__ = .);
    } :loader

    PROVIDE(__end__ = ABSOLUTE(.));

    /DISCARD/ : {
        *(.group)
        *(.comment)
        *(.note)
        *(.interp)
    }
}
