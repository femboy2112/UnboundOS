ENTRY(_mb2_start)

SECTIONS
{
  . = 2M;

  .boot : ALIGN(8)
  {
    KEEP(*(.multiboot2_header))
    *(.text.boot)
  }

  .text : ALIGN(4K)
  {
    *(.text .text.*)
  }

  .rodata : ALIGN(4K)
  {
    *(.rodata .rodata.*)
  }

  .data : ALIGN(4K)
  {
    *(.data .data.*)
  }

  .bss : ALIGN(4K)
  {
    *(COMMON)
    *(.bss .bss.*)
  }
}
