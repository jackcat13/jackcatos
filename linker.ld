ENTRY(_start)

SECTIONS
{
    /* Kernel is loaded at physical 0x8000 */
    . = 0x8000;

    /* --- Text --- */
    .text ALIGN(4K) :
    {
        *(.text .text.*)
    }

    /* --- Read-only data --- */
    .rodata ALIGN(4K) :
    {
        *(.rodata .rodata.*)
    }

    /* --- Data --- */
    .data ALIGN(4K) :
    {
        *(.data .data.*)
    }

    /* --- BSS --- */
    .bss ALIGN(4K) :
    {
        _bss_start = .;
        *(.bss .bss.*)
        *(COMMON)
        _bss_end = .;
    }

    _bss_size = _bss_end - _bss_start;
}
