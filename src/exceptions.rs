use crate::{memory, system_timer};
use crate::{processor, threads};
use alloc::boxed::Box;
use core::{alloc::Layout, convert::TryFrom};
use log::{error, trace};
use processor::ProcessorMode;
use rost_api::syscalls::Syscalls;

#[rost_macros::exception]
unsafe fn Reset() {
    error!("reset handler");
    panic!();
}

#[rost_macros::exception]
unsafe fn UndefinedInstruction() {
    trace!("undefined instruction handler");
    debug_assert!(processor::get_processor_mode() == ProcessorMode::System);

    let mut lr: usize;
    asm!("mov {}, r14", out(reg) lr);

    error!("undefined instruction at {:#X}", lr - 4);
    panic!();
}

#[rost_macros::exception]
unsafe extern "C" fn SoftwareInterrupt(arg0: u32, arg1: u32, arg2: u32, service_id: u32) -> usize {
    // let mut lr: usize;
    // asm!("mov {}, r12", out(reg) lr);
    // println!("software interrupt at {:#X}", lr-4);

    trace!(
        "software interrupt handler {} {} {} {}",
        arg0,
        arg1,
        arg2,
        service_id
    );
    debug_assert!(processor::get_processor_mode() == ProcessorMode::System);

    match Syscalls::try_from(service_id) {
        Ok(Syscalls::YieldThread) => {
            trace!("syscall: YieldThread");
            super::threads::schedule(None);
            return 0;
        }
        Ok(Syscalls::CreateThread) => {
            trace!("syscall: CreateThread");
            let raw_entry: *mut dyn FnMut() = core::mem::transmute((arg0, arg1));
            let entry = Box::<dyn FnMut() + 'static>::from_raw(raw_entry);
            let id = super::threads::create_thread_internal(entry);
            return id;
        }
        Ok(Syscalls::ExitThread) => {
            trace!("syscall: ExitThread");
            super::threads::exit_internal();
            return 0;
        }
        Ok(Syscalls::ReceiveDBGU) => {
            trace!("syscall: ReceiveDBGU");
            if let Some(ch) = crate::dbgu::DBGU_BUFFER.pop() {
                return ch as usize;
            }
            return 0xFFFF;
        }
        Ok(Syscalls::SendDBGU) => {
            trace!("syscall: SendDBGU");
            super::dbgu::write_char(arg0 as u8 as char);
            return 0;
        }
        Ok(Syscalls::Allocate) => {
            trace!("syscall: Allocate");
            let layout = Layout::from_size_align(arg0 as usize, arg1 as usize).expect("Bad layout");

            return alloc::alloc::alloc(layout) as usize;
        }
        Ok(Syscalls::Deallocate) => {
            trace!("syscall: Deallocate");
            let layout = Layout::from_size_align(arg1 as usize, arg2 as usize).expect("Bad layout");

            alloc::alloc::dealloc(arg0 as *mut u8, layout);
            return 0;
        }
        Ok(Syscalls::GetCurrentRealTime) => {
            trace!("syscall: GetCurrentRealTime");
            return system_timer::get_current_real_time() as usize;
        }
        Ok(Syscalls::Sleep) => {
            trace!("syscall: Sleep");
            let current_time = system_timer::get_current_real_time() as usize;
            let current_tcb = threads::get_current_thread();

            current_tcb.wakeup_timestamp = current_time + arg0 as usize;
            current_tcb.state = threads::ThreadState::Waiting;

            threads::schedule(None);

            return system_timer::get_current_real_time() as usize - current_time as usize;
        }
        _ => {
            error!("unknown syscall id {}", service_id);
            panic!();
        }
    }
}

#[rost_macros::exception]
unsafe fn PrefetchAbort() {
    error!("prefetch abort handler");
    debug_assert!(processor::get_processor_mode() == ProcessorMode::System);

    let mut lr: usize;
    asm!("mov {}, r14", out(reg) lr);

    error!("prefetch abort at {:#X}", lr - 4);
    panic!();
}

#[rost_macros::exception]
unsafe fn DataAbort() {
    error!("data abort handler");
    debug_assert!(processor::get_processor_mode() == ProcessorMode::System);

    let mut lr: usize;
    asm!("mov {}, pc", out(reg) lr);
    error!(
        "data abort at {:#X} for address {:#X}",
        lr - 8,
        memory::mc_get_abort_address() // doesn't work in the emulator
    );
    panic!();
}
