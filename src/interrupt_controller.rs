use crate::helpers::{write_register, read_register};
use crate::processor;
use log::debug;

/// Advanced Interrupt Controller base address
const AIC : u32 = 0xFFFFF000;

/// AIC Source Vector register 1 aka. system interrupt offset
const AIC_SVR1 : u32 = 0x84;

/// AIC Interrupt Mask register offset
const AIC_IMR : u32 = 0x110;

/// AIC Interrupt Enable Command register offset
const AIC_IECR : u32 = 0x120;

/// End of Interrupt Command Register
const AIC_EOICR : u32 = 0x130;

/// Sets system interrupt vector and enables the interrupt.
pub fn init_system_interrupt() {

    write_register(AIC, AIC_SVR1, handler as *mut () as u32);

    let mut status : u32 = read_register(AIC, AIC_IMR);

    status = status | 0x2;

    write_register(AIC, AIC_IECR, status);

}

fn logging() {
  debug!("Interrupt Handler for interrupt line 1");
  debug!("processor mode {:?}", processor::get_processor_mode());
  write_register(AIC, AIC_EOICR, 1);
}

#[naked]
pub extern fn handler (){
    unsafe { 
        asm!("
            PUSH {{r0-r12}}
            PUSH {{lr}}
        "); 
    }
    logging();
    unsafe { 
        asm!("
            POP {{lr}}
            POP {{r0-r12}}
            SUBS pc, lr, #4"); 
    }
}