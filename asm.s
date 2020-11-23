.section .ExceptionsJump, "ax"
  .global ExceptionsJump
ExceptionsJump:
  ldr pc, ResetTrampoline
  ldr pc, UndefTrampoline
  ldr pc, SWITrampoline


ResetTrampoline:
  b ResetHandler

UndefTrampoline:
  b UndefinedInstruction

SWITrampoline:
  b SoftwareInterrupt
