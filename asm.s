.section .ExceptionsJump, "ax", %progbits
  .global ExceptionsJump
ExceptionsJump:
  b _start
  b UndefinedInstructionHandler
  b SoftwareInterruptHandler 
  b PrefetchAbortHandler
  b DataAbortHandler
  nop
  b HardwareInterruptHandler
  b FastInterruptHandler
