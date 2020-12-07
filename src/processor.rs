use num_traits::{FromPrimitive, ToPrimitive};

#[repr(u8)]
#[derive(FromPrimitive, ToPrimitive, Debug, Copy, Clone, Eq, PartialEq)]
pub enum ProcessorMode {
    User = 0x10,
    FIQ = 0x11,
    IRQ = 0x12,
    Supervisor = 0x13,
    Abort = 0x17,
    Undefined = 0x1b,
    System = 0x1F,
}

/// Returns the current processor mode.
/// Requires the caller to be in a priviliged mode.
pub fn get_processor_mode() -> ProcessorMode {
    let mut cpsr: u32;

    unsafe {
        asm!("MRS {0}, CPSR", out(reg) cpsr);
    }

    ProcessorMode::from_u8((cpsr & 0x1F) as u8).unwrap()
}

/// Switches the processor to the specified mode.
/// Requires the caller to be in a priviliged mode.  
/// Clobbers r0.
macro_rules! _switch_processor_mode {
    ($mode:expr) => {
        unsafe {
            asm!(
                "
            MRS r0, cpsr
            BIC r0, r0, #0x1F
            ORR r0, r0, {}
            MSR cpsr_c, r0
            ", const $mode as i32
            );
        }
   };
}

pub(crate) use _switch_processor_mode as switch_processor_mode;

/// Either sets or unsets the interrupt mask bit in the processor status word.
/// Requires the caller to be in priviliged mode.  
/// Clobbers r0.
macro_rules! _set_interrupts_enabled {
    (false) => {
        unsafe{
            asm!("
                MRS r0, CPSR
                ORR r0, r0, #0x80
                MSR    CPSR_c, r0
            ")
        }
    };
    (true) => {
        unsafe{
            asm!("
                MRS r0, CPSR
                BIC r0, r0, #0x80
                MSR    CPSR_c, r0
            ")
        }
    }
}

pub(crate) use _set_interrupts_enabled as set_interrupts_enabled;

/// Macro to switch into system mode.    
///
/// This macro allows for the interrupt routine to be executed in  
/// System Mode instead of the exception mode that was entered by the  
/// exception. The benefit is that it allows the interrupt routine to   
/// enable nested interrupts while using function calls, without the risk  
/// of corrupting the link register by a second exception to the same exception mode.
macro_rules! _exception_routine {
    ($subcall:expr, $lr_size:expr) => {
        #[allow(unused_unsafe)]
        unsafe {
            asm!("
                stmfd sp!, {{r0-r12, r14}}
                mrs r14, SPSR
                mrs r12, CPSR
                stmfd sp!, {{r14}}
            ", );
            processor::switch_processor_mode!(processor::ProcessorMode::System);
            processor::set_interrupts_enabled!(true);
            asm!("stmfd sp!, {{r12, lr}}");
            $subcall
            asm!("
                ldmfd sp!, {{r12, lr}}
                msr CPSR, r12
            ");
            //interrupt_controller::set_interrupts_enabled!(false);
            //switch_processor_mode!(processor::ProcessorMode::IRQ);
            $crate::interrupt_controller::mark_end_of_interrupt!();
            asm!("
                ldmfd sp!, {{r14}}
                msr SPSR, r14 
                ldmfd sp!, {{r0-r12, r14}}
                subs pc, lr, #{}
            ", const $lr_size);
        }
    };
}

pub(crate) use _exception_routine as exception_routine;

