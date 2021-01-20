use crate::{memory, system_timer};
use crate::{processor, threads};
use alloc::boxed::Box;
use core::{alloc::Layout, convert::TryFrom};
use log::{error, trace};
use processor::ProcessorMode;
use rost_api::syscalls::Syscalls;
use threads::ThreadState;

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
            0
        }
        Ok(Syscalls::CreateThread) => {
            trace!("syscall: CreateThread");
            let raw_entry: *mut dyn FnMut() = core::mem::transmute((arg0, arg1));
            let entry = Box::<dyn FnMut() + 'static>::from_raw(raw_entry);
            super::threads::create_thread_internal(entry)
        }
        Ok(Syscalls::ExitThread) => {
            trace!("syscall: ExitThread");
            super::threads::exit_internal();
            0
        }
        Ok(Syscalls::ReceiveDBGU) => {
            trace!("syscall: ReceiveDBGU");
            if let Some(ch) = crate::dbgu::DBGU_BUFFER.pop() {
                return ch as usize;
            }
            0xFFFF
        }
        Ok(Syscalls::SendDBGU) => {
            trace!("syscall: SendDBGU");
            super::dbgu::write_char(arg0 as u8 as char);
            0
        }
        Ok(Syscalls::Allocate) => {
            trace!("syscall: Allocate");
            let layout = Layout::from_size_align(arg0 as usize, arg1 as usize).expect("Bad layout");

            alloc::alloc::alloc(layout) as usize
        }
        Ok(Syscalls::Deallocate) => {
            trace!("syscall: Deallocate");
            let layout = Layout::from_size_align(arg1 as usize, arg2 as usize).expect("Bad layout");

            alloc::alloc::dealloc(arg0 as *mut u8, layout);
            0
        }
        Ok(Syscalls::GetCurrentRealTime) => {
            trace!("syscall: GetCurrentRealTime");
            system_timer::get_current_real_time() as usize
        }
        Ok(Syscalls::Sleep) => {
            trace!("syscall: Sleep");
            let current_time = system_timer::get_current_real_time() as usize;
            let current_tcb = threads::get_current_thread();

            current_tcb.wakeup_timestamp = Some(current_time + arg0 as usize);
            current_tcb.state = threads::ThreadState::Waiting;

            threads::schedule(None);

            system_timer::get_current_real_time() as usize - current_time as usize
        }
        Ok(Syscalls::JoinThread) => {
            trace!("syscall: JoinThread");
            let join_thread = {
                let thread = threads::get_thread_by_id(arg0 as usize);
                if threads::get_thread_by_id(arg0 as usize).is_none() {
                    return 0;
                }
                thread.unwrap()
            };

            if join_thread.state == ThreadState::Stopped {
                return 0;
            }

            let current_tcb = threads::get_current_thread();

            if join_thread.parent_thread_id != current_tcb.id {
                panic!("JoinThread: you cannot join a thread which is not your parent thread");
            }

            current_tcb.joined_thread_ids.insert(arg0 as usize);
            if arg1 > 0 {
                let current_time = system_timer::get_current_real_time() as usize;
                current_tcb.wakeup_timestamp = Some(current_time + arg1 as usize);
            }
            current_tcb.state = threads::ThreadState::Waiting;

            threads::schedule(None);
            0
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
