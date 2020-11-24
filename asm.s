.section .ExceptionsJump, "ax", %progbits
  .global ExceptionsJump
ExceptionsJump:
  ldr pc, _reset 
  ldr pc, _undefined_instruction 
  ldr pc, _software_interrupt 
  ldr pc, _prefetch_abort 
  ldr pc, _data_abort 
  ldr pc, _unused 
  ldr pc, _irq 
  ldr pc, _fiq 

  _reset: .word _start
  _undefined_instruction: .word SoftwareInterrupt
  _software_interrupt: .word __xcpt_dummy
  _prefetch_abort: .word SoftwareInterrupt
  _data_abort: .word SoftwareInterrupt
  _unused: .word SoftwareInterrupt
  _irq: .word SoftwareInterrupt
  _fiq: .word SoftwareInterrupt

.global __xcpt_dummy
__xcpt_dummy:
b       __xcpt_dummy