OUTPUT_ARCH(riscv64imac)
ENTRY(__onyx_start)

PHDRS {
    text    PT_LOAD FLAGS(5); /* PF_R | PF_X */
    rodata  PT_LOAD FLAGS(4); /* PF_R        */
    data    PT_LOAD FLAGS(6); /* PF_R | PF_W */
    dynamic PT_DYNAMIC;
}

MEMORY {
    /* Definition for qemu-system-riscv64 virt machines. */
    ram (rwx): ORIGIN = 0x80000000, LENGTH = 128M
}

/* The page size used by the kernel. */
PAGE_SIZE = 4K;

SECTIONS {
    PROVIDE(__start__ = 0);

    . = __start__;

    .r0 : {
        KEEP(*(.r0 .r0.*))
        . = ALIGN(8);
    } :text

    .text : {
        *(.text .text.*)
        . = ALIGN(8);
    } :text

    .plt : {
        *(.plt .plt.*)
        . = ALIGN(8);
    } :text

    . = ALIGN(PAGE_SIZE);
    __text_end__ = .;

    __rodata_start__ = .;

    .rodata : {
        *(.rodata .rodata.*)
        . = ALIGN(8);
    } :rodata

    .data.rel.ro : {
        *(.data.rel.ro .data.rel.ro.*)
        . = ALIGN(8);
    } :rodata

    .hash     : { *(.hash)             } :rodata
    .gnu.hash : { *(.gnu.hash)         } :rodata
    .dynsym   : { *(.dynsym .dynsym.*) } :rodata
    .dynstr   : { *(.dynstr .dynstr.*) } :rodata
    .rela.dyn : { *(.rela.*)           } :rodata

    .dynamic : {
        HIDDEN(__dynamic_start__ = .);
        *(.dynamic)
    } :rodata :dynamic

    __got_start__ = .;

    .got : {
        *(.got)
        *(.igot)
    } :rodata

    .got.plt : {
        *(.got.plt)
        *(.igot.plt)
    } :rodata

    __got_end__ = .;

    . = ALIGN(PAGE_SIZE);
    __rodata_end__ = .;

    __data_start__ = .;

    .data ALIGN(8) : {
        *(.data .data.*)
        SORT(CONSTRUCTORS)
        . = ALIGN(8);
    } :data

    . = ALIGN(PAGE_SIZE);
    __data_end__ = .;

    __bss_start__ = .;

    .rela.dyn : { *(.rela.*) } :data

    .bss ADDR(.rela.dyn) (NOLOAD) : {
        *(.bss .bss.*)
        *(COMMON)
        *(.dynbss)
    } :data

    . = ALIGN(PAGE_SIZE);
    __bss_end__ = .;

    __end__ = ABSOLUTE(.);

    /DISCARD/ : {
        *(.group)
        *(.comment)
        *(.note)
        *(.interp)
    }
}
