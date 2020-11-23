
EXTERN(ExceptionsJump)
EXTERN(StartJump)

ENTRY(_start)

MEMORY {
   BOOT : ORIGIN = 0x0, LENGTH = 1M
   SRAM : ORIGIN = 0x00200000, LENGTH = 1M
   SDRAM : ORIGIN = 0X20000000, LENGTH = 16M
}

SECTIONS
{

  .vector_table :
  {
      *(.ExceptionsJump);
      *(.ExceptionsTrampolines);
  } > BOOT

   .text : {
      *(.text);
  } > SDRAM 


  /DISCARD/ :
  {
    /* Unused exception related info that only wastes space */
    *(.ARM.exidx);
    *(.ARM.exidx.*);
    *(.ARM.extab.*);
  }
}
