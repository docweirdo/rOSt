ENTRY(main)

MEMORY {
   MAIN : ORIGIN = 0x21000000, LENGTH = 8M
}

SECTIONS
{
  .usercode : {
    *(.text.main);
    *(.text.*);
  } > MAIN

  /DISCARD/ :
  {
    /* Unused exception related info that only wastes space */
    *(.ARM.exidx);
    *(.ARM.exidx.*);
    *(.ARM.extab.*);
  }
}
