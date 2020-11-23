.section .ExceptionsJump, "ax"
  .global ExceptionsJump
ExceptionsJump:
  ldr pc, _SoftwareInterrupt
  ldr pc, _SoftwareInterrupt
  ldr pc, _SoftwareInterrupt
  ldr pc, _SoftwareInterrupt
  ldr pc, _SoftwareInterrupt
  ldr pc, _SoftwareInterrupt
  ldr pc, _SoftwareInterrupt

  _SoftwareInterrupt: .word SoftwareInterrupt
