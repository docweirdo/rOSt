use super::threads;
use core::convert::TryFrom;
use num_enum::TryFromPrimitive;

#[repr(u8)]
#[derive(TryFromPrimitive, Debug, Copy, Clone, Eq, PartialEq)]
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

    ProcessorMode::try_from((cpsr & 0x1F) as u8).unwrap()
}

/// Switches the processor to the specified mode.  
/// Requires the caller to be in a priviliged mode.  
/// Clobbers r0.
macro_rules! _switch_processor_mode {
    ($mode:expr) => {
        #[allow(unused_unsafe)]
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

pub fn interrupts_enabled() -> bool {
    let mut cpsr: u32;

    unsafe {
        asm!("MRS {0}, CPSR", out(reg) cpsr);
    }

    (cpsr & 0x80) == 0
}

/// Either sets or unsets the interrupt mask bit in the processor status word.
/// Requires the caller to be in priviliged mode.
/// Clobbers r0.
macro_rules! _set_interrupts_enabled {
    (false) => {
        #[allow(unused_unsafe)]
        unsafe {
            asm!(
                "
                MRS r0, CPSR
                ORR r0, r0, #0x80
                MSR    CPSR_c, r0
            "
            )
        }
    };
    (true) => {
        #[allow(unused_unsafe)]
        unsafe {
            asm!(
                "
                MRS r0, CPSR
                BIC r0, r0, #0x80
                MSR    CPSR_c, r0
            "
            )
        }
    };
}

pub(crate) use _set_interrupts_enabled as set_interrupts_enabled;

// macro_rules! _exception_routine {
//     ($subcall:ident, $lr_size:expr, $enable_msk_intr:expr, $mark_intr_end:expr) => {
//         #[allow(unused_unsafe)]
//         unsafe {
//             asm!("
//                 push {{r0-r12, r14}}
//                 mrs r14, SPSR
//                 mrs r12, CPSR
//                 push {{r14}}
//             ", );
//             processor::switch_processor_mode!(processor::ProcessorMode::System);
//             if $enable_msk_intr {processor::set_interrupts_enabled!(true);}
//             asm!("push {{r12, r14}}");
//             $subcall();
//             asm!("
//                 pop {{r12, r14}}
//                 msr CPSR, r12
//             ");
//             //if $enable_msk_intr { processor::set_interrupts_enabled!(false);}
//             //processor::switch_processor_mode!(processor::ProcessorMode::IRQ);
//             if $mark_intr_end {crate::interrupt_controller::mark_end_of_interrupt!();}
//             asm!("
//                 pop {{r14}}
//                 msr SPSR, r14
//                 pop {{r0-r12, r14}}
//                 subs pc, lr, #{}
//             ", const $lr_size, options(noreturn));
//         }
//     };
// }

// pub(crate) use _exception_routine as exception_routine;

/// Macro to switch into system mode.    
///
/// This macro allows for the interrupt routine to be executed in  
/// System Mode instead of the exception mode that was entered by the  
/// exception. The benefit is that it allows the interrupt routine to   
/// enable nested interrupts while using function calls, without the risk  
/// of corrupting the link register by a second exception to the same exception mode.
macro_rules! _exception_routine {
    (subroutine=$subcall:ident, lr_size=$lr_size:expr, nested_interrupt=true, mark_end_of_interrupt=true) => {
            asm!(
                // save two work registers (r0, r1), get sp_user and push spsr and context to userstack
                "push {{r0, r1}}",  // r1, r0   |
                "mrs r0, spsr",
                "sub sp, sp, #4",   // r1, r0, x    |
                "stm sp, {{sp}}^",  // r1, r0, sp_user  |
                "nop",
                "pop {{r1}}",       // r1, r0   |
                "sub lr, lr, #{lr_size}",
                "stmfd r1!, {{r0, r2-r12,r14}}",  // r1, r0   |   spsr, r2-12, r14_irq

                // save original r0 and cpsr on userstack
                "pop {{r0}}",               // r1   |   spsr, r2-12, r14_irq
                "stmfd r1!, {{r0}}",          // r1   |  spsr, r2-12, r14_irq, r0
                "mrs r0, cpsr",
                "stmfd r1!, {{r0}}",          // r1   |  spsr, r2-12, r14_irq, r0, cpsr
                "pop {{r1}}",               //      |  spsr, r2-12, r14_irq, r0, cpsr

                // switch to system mode
                "MRS r0, cpsr",
                "BIC r0, r0, #0x1F",
                "ORR r0, r0, #0x1F",
                "MSR cpsr_c, r0",

                // increment sp to actual stacksize and save rest of thread context plus user lr
                "sub sp, sp, #(15*4)",
                "push {{r1, r14}}",         // spsr, r2-12, r14_irq, r0, cpsr, r14_user, r0

                // enable interrupts
                "MRS r0, CPSR",
                "BIC r0, r0, #0x80",
                "MSR    CPSR_c, r0",

                // jump to subcall
                "bl {subcall}",

                // disable interrupts
                "MRS r0, CPSR",
                "ORR r0, r0, #0x80",
                "MSR    CPSR_c, r0",

                // restore user r1 and lr, switch back to former exception mode
                "pop {{r1,r14}}",       // spsr, r2-12, r14_irq, r0, cpsr
                "pop {{r0}}",           // spsr, r2-12, r14_irq, r0
                "msr CPSR, r0", // switch to former exception mode

                // get sp_user, get user/thread context back
                "push {{r1}}",      // r1   |   spsr, r2-r14, r0
                "sub sp, sp, #4",   // r1, x   |   spsr, r2-r14, r0
                "stm sp, {{sp}}^", // r1, sp_user | spsr, r2-r14, r0
                "nop",
                "pop {{r1}}",   // r1   | spsr, r2-r14, r0
                "ldmfd r1!, {{r0}}",  // r1   | spsr, r2-r14
                "push {{r0}}",  // r1, r0   | spsr, r2-r14
                "ldmfd r1!, {{r0, r2-r12,r14}}",  // r1, r0

                // write updated sp_user back to orig. register
                "push {{r1}}",  // r1, r0, sp_user
                "ldm sp, {{sp}}^",  // r1, r0, x
                "nop",
                "add sp, sp, #4",    // r1, r0

                // write back orig. exception spsr and restore work registers
                "msr SPSR, r0",
                "pop {{r0, r1}}", // r1, r0

                // return to user mode
                "subs pc, lr, #0"
            , subcall = sym $subcall, lr_size = const $lr_size,
             options(noreturn));
    };
    (subroutine=$subcall:ident, lr_size=$lr_size:expr, nested_interrupt=false, mark_end_of_interrupt=false) => {
            asm!(
                // save two work registers (r0, r1), get sp_user and push spsr and context to userstack
                "push {{r0, r1}}",  // r1, r0   |
                "mrs r0, spsr",
                "sub sp, sp, #4",   // r1, r0, x    |
                "stm sp, {{sp}}^",  // r1, r0, sp_user  |
                "nop",
                "pop {{r1}}",       // r1, r0   |
                "sub lr, lr, #{lr_size}",
                "stmfd r1!, {{r0, r2-r12,r14}}",  // r1, r0   |   spsr, r2-12, r14_irq

                // save original r0 and cpsr on userstack
                "pop {{r0}}",               // r1   |   spsr, r2-12, r14_irq
                "stmfd r1!, {{r0}}",          // r1   |  spsr, r2-12, r14_irq, r0
                "mrs r0, cpsr",
                "stmfd r1!, {{r0}}",          // r1   |  spsr, r2-12, r14_irq, r0, cpsr
                "pop {{r1}}",               //      |  spsr, r2-12, r14_irq, r0, cpsr

                // save lr for swi
                "mov r12, lr",

                // switch to system mode
                "MRS r0, cpsr",
                "BIC r0, r0, #0x1F",
                "ORR r0, r0, #0x1F",
                "MSR cpsr_c, r0",

                // increment sp to actual stacksize and save rest of thread context plus user lr
                "sub sp, sp, #(15*4)",
                "push {{r1, r14}}",         // spsr, r2-12, r14_irq, r0, cpsr, r1, r14_user

                // jump to subcall
                "bl {subcall}",

                // restore user r1 and lr, switch back to former exception mode
                "pop {{r1,r14}}",       // spsr, r2-12, r14_irq, r0, cpsr, r1, lr_user, r1
                "pop {{r0}}",           // spsr, r2-12, r14_irq, r0
                "msr CPSR, r0", // switch to former exception mode

                // get sp_user, get user/thread context back
                "push {{r1}}",      // r1   |   spsr, r2-r14, r0
                "sub sp, sp, #4",   // r1, x   |   spsr, r2-r14, r0
                "stm sp, {{sp}}^", // r1, sp_user | spsr, r2-r14, r0
                "nop",
                "pop {{r1}}",   // r1   | spsr, r2-r14, r0
                "ldmfd r1!, {{r0}}",  // r1   | spsr, r2-r14
                "push {{r0}}",  // r1, r0   | spsr, r2-r14
                "ldmfd r1!, {{r0, r2-r12,r14}}",  // r1, r0

                // write updated sp_user back to orig. register
                "push {{r1}}",  // r1, r0, sp_user
                "ldm sp, {{sp}}^",  // r1, r0, x
                "nop",
                "add sp, sp, #4",    // r1, r0

                // write back orig. exception spsr and restore work registers
                "msr SPSR, r0",
                "pop {{r0, r1}}", // r1, r0

                // return to user mode
                "subs pc, lr, #0"
            , subcall = sym $subcall,
            lr_size = const $lr_size, options(noreturn));
    };
}

pub(crate) use _exception_routine as exception_routine;
