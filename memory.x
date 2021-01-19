
EXTERN(ExceptionsJump)
ENTRY(_start)

MEMORY {
   BOOT : ORIGIN = 0x0, LENGTH = 1M           /* Not actual memory, ROM, FLASH or SRAM are mapped to it */
   ROM : ORIGIN = 0x00100000, LENGTH = 1M
   SRAM : ORIGIN = 0x00200000, LENGTH = 1M
   FLASH : ORIGIN = 0x10000000, LENGTH = 16M
   SDRAM : ORIGIN = 0X20000000, LENGTH = 64M
}

SECTIONS
{
  .vector_table :
  {
    *(.ExceptionsJump);
  } > SRAM

  .text : {
    *(.text*);
    _end_text = .;
  } > SDRAM

  .data : {
    *(.data*);
    _end_data = .;
  } > SDRAM

  .rodata : {
    *(.rodata*);
    _end_rodata = .;
  } > SDRAM

  .bss : {
    *(.bss*);
    _end_bss = .;
  } > SDRAM


  ASSERT(
     (_end_bss < 0X21000000),
     "kernel program data overflows custom code entry")

  
  /DISCARD/ :
  {
    /* Unused exception related info that only wastes space */
    *(.ARM.exidx);
    *(.ARM.exidx.*);
    *(.ARM.extab.*);
    *(.ARM.attributes*);
    *(.debug*);
  }
}
