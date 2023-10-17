/* output an x86-64 ELF file */
OUTPUT_ARCH(i386:x86-64)
OUTPUT_FORMAT(elf64-x86-64)

/* make `kinit` symbol the entry point */
ENTRY(kinit)

/* define program headers to get desired MMU permissions */
PHDRS
{
    text    PT_LOAD    FLAGS((1 << 0) | (1 << 2)) ; /* Execute + Read */
    rodata  PT_LOAD    FLAGS((1 << 2)) ;            /* Read only */
    data    PT_LOAD    FLAGS((1 << 1) | (1 << 2)) ; /* Write + Read */
    dynamic PT_DYNAMIC FLAGS((1 << 1) | (1 << 2)) ; /* Write + Read */
}

SECTIONS
{
    /* We want to be placed in the topmost 2GiB of the address space, for optimizations */
    /* and because that is what the Limine spec mandates. */
    /* Any address in this region will do, but often 0xffffffff80000000 is chosen as */
    /* that is the beginning of the region. */
    . = 0xffffffff80000000;

    .text : {
        *(.text .text.*)
    } :text

    /* Move to the next memory page for .rodata */
    . += CONSTANT(MAXPAGESIZE);

    /* The built-in `x86_64-unknown-none` target generates relocatable executables */
    /* by default, so we need to include the relocation information (.dynstr, .dynsym, */
    /* and .rela) for the bootloader to properly load the kernel at runtime. */
    .dynsym : {
        *(.dynsym)
    } :rodata

    .dynstr : {
        *(.dynstr)
    } :rodata

    .rela : {
        *(.rela*)
    } :rodata

    .rodata : {
        *(.rodata .rodata.*)
    } :rodata

    /* Move to the next memory page for .data */
    . += CONSTANT(MAXPAGESIZE);

    /* The dynamic table is used to find the relocation info (declared above), so it */
    /* must be included both in the :data and :dynamic segments. */
    .dynamic : {
        *(.dynamic)
    } :data :dynamic

    .got : {
        *(.got)
    } :data

    .data : {
        *(.data.rel.ro .data.rel.ro.*)
        *(.data .data.*)
    } :data

    /* NOTE: .bss needs to be the last thing mapped to :data, otherwise lots of */
    /* unnecessary zeros will be written to the binary. */
    /* If you need, for example, .init_array and .fini_array, those should be placed */
    /* above this. */
    .bss : {
        *(COMMON)
        *(.dynbss)
        *(.bss .bss.*)
    } :data

    /* Discard .note.* and .eh_frame since they may cause issues on some hosts. */
    /DISCARD/ : {
        *(.eh_frame)
        *(.note .note.*)
    }
}