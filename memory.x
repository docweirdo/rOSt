ENTRY(_start)

EXTERN(ExceptionsJump)

SECTIONS
{

   . = 0x00000000;
  .vector_table :
  {
      *(.ExceptionsJump);
      *(.ExceptionsTrampolines);
  }
   . = 0x20000000;
   .text : {
      *(.text);
  } 
}