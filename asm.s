.section .ExceptionsJump, "ax", %progbits
  .global ExceptionsJump
ExceptionsJump:
  b _start
  b UndefinedInstructionHandler
  b SoftwareInterruptHandler 
  b PrefetchAbortHandler
  b DataAbortHandler
  nop
  ldr  pc,[pc,# -0xF20]
  ldr  pc,[pc,# -0xF20]
