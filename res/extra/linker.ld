SECTIONS {
    . = 0x00400000;
    .text : ALIGN(4) { *(.text); }
    . = 0x10010000;
    .data : ALIGN(4) { *(.data); }
}

ENTRY(__start)
