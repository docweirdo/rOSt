use super::threads;
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
                // save userstack pointer, spsr, adjusted lr, r0 as work register and cpsr to later return on userstack
                "push {{r1}}",
                "ldm r1, {{sp}}^", // TODO: nop or not?
                "nop",
                "sub lr, lr, #{lr_size}",
                "stm r1, {{r1, spsr, lr, r0, cpsr}}",
                "pop {{r1}}",

                // switch to system mode
                "MRS r0, cpsr",
                "BIC r0, r0, #0x1F",
                "ORR r0, r0, #0x1F",
                "MSR cpsr_c, r0",

                // increment sp to actual stacksize and save rest of thread context
                "sub sp, sp, 20",
                "push {{r1-r14}}",

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
                
                // partially restore context of thread and return to former exception mode
                "pop {{r1-r14}}",
                "pop {{r0}}",
                "msr CPSR, r0", // switch to former exception mode

                // mark end of interrupt
                "ldr r0, =0xFFFFF000",
                "str r0, [r0, #0x130]",

                // pop lr and spsr from userstack! to correctly return to user mode
                "push {{r1}}",
                "ldm r1, {{sp}}^", // TODO: nop or not?
                "nop",
                "ldm r1!, {{spsr, lr, r0}}",
                "ldm sp!, {{sp}}^",
                "pop {{r1}}",

                // return to user mode
                "subs pc, lr, #0"
            , subcall = sym $subcall, lr_size = const $lr_size, options(noreturn));
    };
    (subroutine=$subcall:ident, lr_size=$lr_size:expr, nested_interrupt=false, mark_end_of_interrupt=true) => {
            asm!(
                // save all registers
                "push {{r0-r12, r14}}",
                "mrs r14, SPSR",
                "mrs r12, CPSR",
                "push {{r14}}",

                // switch to system mode
                "MRS r0, cpsr",
                "BIC r0, r0, #0x1F",
                "ORR r0, r0, #0x1F",
                "MSR cpsr_c, r0",

                // jump to subcall
                "push {{r12, r14}}",
                "bl {}",
                "pop {{r12, r14}}",
                "msr CPSR, r12", // restore exception mode and disable interrupts

                // mark end of interrupt
                "ldr r0, =0xFFFFF000",
                "str r0, [r0, #0x130]",

                // restore registers
                "pop {{r14}}",
                "msr SPSR, r14 ",
                "pop {{r0-r12, r14}}",
                "subs pc, lr, #{}"
            , sym $subcall, const $lr_size, options(noreturn));
    };
    (subroutine=$subcall:ident, lr_size=$lr_size:expr, nested_interrupt=true, mark_end_of_interrupt=false) => {
            asm!(
                // save all registers
                "push {{r0-r12, r14}}",
                "mrs r14, SPSR",
                "mrs r12, CPSR",
                "push {{r14}}",

                // switch to system mode
                "MRS r0, cpsr",
                "BIC r0, r0, #0x1F",
                "ORR r0, r0, #0x1F",
                "MSR cpsr_c, r0",

                // enable interrupts
                "MRS r0, CPSR",
                "BIC r0, r0, #0x80",
                "MSR    CPSR_c, r0",

                // jump to subcall
                "push {{r12, r14}}",
                "bl {}",
                "pop {{r12, r14}}",
                "msr CPSR, r12", // restore exception mode and disable interrupts

                // restore registers
                "pop {{r14}}",
                "msr SPSR, r14 ",
                "pop {{r0-r12, r14}}",
                "subs pc, lr, #{}"
            , sym $subcall, const $lr_size, options(noreturn));
    };
    (subroutine=$subcall:ident, lr_size=$lr_size:expr, nested_interrupt=false, mark_end_of_interrupt=false) => {
            asm!(
                // save all registers
                "push {{r0-r12, r14}}",
                "mrs r14, SPSR",
                "mrs r12, CPSR",
                "push {{r14}}",

                // switch to system mode
                "MRS r0, cpsr",
                "BIC r0, r0, #0x1F",
                "ORR r0, r0, #0x1F",
                "MSR cpsr_c, r0",

                // jump to subcall
                "push {{r12, r14}}",
                "bl {subcall}",
                "pop {{r12, r14}}",
                "msr CPSR, r12", // restore exception mode and disable interrupts

                // call thread scheduler
                "bl {schedule_threads}",

                // restore registers
                "pop {{r14}}",
                "msr SPSR, r14 ",
                "pop {{r0-r12, r14}}",
                "subs pc, lr, #{lr_size}"
            , subcall = sym $subcall, schedule_threads = sym $crate::threads::schedule_threads, lr_size = const $lr_size, options(noreturn));
    };
}

pub(crate) use _exception_routine as exception_routine;
