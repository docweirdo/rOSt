.section .ExceptionsJump, "ax"
  .global ExceptionsJump
ExceptionsJump:
  ldr pc, ResetTrampoline
  ldr pc, UndefTrampoline
  ldr pc, SWITrampoline


ResetTrampoline:
  b ResetHandlerq

UndefTrampoline:
  b UndefinedInstruction

SWITrampoline:
  b SoftwareInterrupt
