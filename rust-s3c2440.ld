ENTRY(_start)
SECTIONS {
  . = 0x30001000;
  .text : {
    KEEP(*(.text.entry))
    *(.text) *(.text.*)
    . = ALIGN(16);
  }
  . = ALIGN(16);
  .rodata : {
    *(.rodata) *(.rodata.*)
  }
  . = ALIGN(16);
  .data : {
    *(.data) *(.data.*)
    *(.eh_frame) *(.gcc_except_table)
  }
  . = ALIGN(16);
  .bss : {
    rust_bss_start = .;
    *(.sbss) *(.scommon) *(.dynbss) *(.bss) *(.bss.stack.*) *(COMMON)
    . = ALIGN(16);
    rust_bss_end = .;
  }
  . = ALIGN(16);

  /DISCARD/ : {
    *(.ARM.exidx*)
  }
}
