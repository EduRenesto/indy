SECTIONS {
    . = 0x00400000;
    .text : { *(.text) }
    . = 0x10010000;
    .data : { *(.data) }
}

ENTRY(__start)
