.section .ExceptionsJump, "ax", %progbits
  .global ExceptionsJump
ExceptionsJump:
  ldr pc, _reset 
  ldr pc, _undefined_instruction 
  ldr pc, _software_interrupt 
  ldr pc, _prefetch_abort 
  ldr pc, _data_abort 
  nop
  ldr pc, _irq 
  ldr pc, _fiq 

  _reset: .word _start
  _undefined_instruction: .word UndefinedInstruction
  _software_interrupt: .word SoftwareInterrupt
  _prefetch_abort: .word PrefetchAbort
  _data_abort: .word DataAbort
  _irq: .word HardwareInterrupt
  _fiq: .word FastInterrupt
