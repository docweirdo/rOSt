.section .ExceptionsJump, "ax", %progbits
  .global ExceptionsJump
ExceptionsJump:
  b Reset
  b UndefinedInstruction
  b SoftwareInterrupt
  b PrefetchAbort
  b DataAbort
  nop
  ldr  pc,[pc,# -0xF20]
  ldr  pc,[pc,# -0xF20]
