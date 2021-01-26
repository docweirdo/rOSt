use crate::{memory, system_timer};
use crate::{processor, threads};
use alloc::boxed::Box;
use core::{alloc::Layout, convert::TryFrom};
use log::{error, trace};
use processor::ProcessorMode;
use rost_api::syscalls;
use rost_api::syscalls::Syscalls;
use threads::ThreadState;

#[rost_macros::exception]
unsafe extern "C" fn Reset() {
    panic!("reset handler");
}

#[rost_macros::exception]
unsafe extern "C" fn UndefinedInstruction(lr: usize) {
    trace!("undefined instruction handler");
    assert!(processor::get_processor_mode() == ProcessorMode::System);

    panic!("undefined instruction at {:#X}", lr - 4);
}

#[rost_macros::exception]
unsafe extern "C" fn SoftwareInterrupt(arg0: u32, arg1: u32, arg2: u32, service_id: u32) -> usize {
    trace!(
        "software interrupt handler {} {} {} {}",
        arg0,
        arg1,
        arg2,
        service_id
    );
    assert!(processor::get_processor_mode() == ProcessorMode::System);

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
            let current_tcb = threads::get_current_thread();
            match current_tcb
                .subscribed_services
                .get_mut(&syscalls::ThreadServices::DBGU)
            {
                Some(messages) => {
                    if let Some(threads::ThreadMessage::DBGU(character)) = messages.pop_front() {
                        return character as usize;
                    }
                }
                _ => {
                    panic!(
                        "syscall DBGU: Service {:?} not subscribed",
                        syscalls::ThreadServices::DBGU
                    );
                }
            }

            if arg0 != 0 {
                current_tcb.state = threads::ThreadState::Waiting(threads::WaitingReason::DBGU);
                threads::schedule(None);

                let current_tcb = threads::get_current_thread();
                if let Some(messages) = current_tcb
                    .subscribed_services
                    .get_mut(&syscalls::ThreadServices::DBGU)
                {
                    if let Some(threads::ThreadMessage::DBGU(character)) = messages.pop_front() {
                        return character as usize;
                    }
                }
                panic!("should always be a character saved from the interrupt");
            } else {
                0xFFFF
            }
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
        Ok(Syscalls::Subscribe) => {
            trace!("syscall: Subscribe");
            let current_tcb = threads::get_current_thread();
            let service = syscalls::ThreadServices::try_from(arg0).expect("invalid service given");

            if current_tcb
                .subscribed_services
                .insert(service, alloc::collections::VecDeque::new())
                != None
            {
                panic!(
                    "syscall Subscribe: Service {:?} already subscribed",
                    service
                );
            }
            0
        }
        Ok(Syscalls::Unsubscribe) => {
            trace!("syscall: Unsubscribe");
            let current_tcb = threads::get_current_thread();
            let service = syscalls::ThreadServices::try_from(arg0).expect("invalid service given");

            if current_tcb.subscribed_services.remove(&service) == None {
                panic!(
                    "syscall Unsubscribe: Service {:?} was not subscribed",
                    service
                );
            }
            0
        }
        Ok(Syscalls::Sleep) => {
            trace!("syscall: Sleep");
            let current_time = system_timer::get_current_real_time() as usize;
            let current_tcb = threads::get_current_thread();

            let time_in_realtime_units: usize =
                arg0 as usize / system_timer::get_real_time_unit_interval().as_millis() as usize;

            current_tcb.state = threads::ThreadState::Waiting(threads::WaitingReason::Sleep(
                current_time + time_in_realtime_units,
            ));

            threads::schedule(None);

            system_timer::get_real_time_unit_interval().as_millis() as usize
                * (system_timer::get_current_real_time() as usize - current_time as usize)
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
                panic!("JoinThread: you cannot join a thread which is not your parent thread: p: {} != {}",
                        join_thread.parent_thread_id, current_tcb.id);
            }

            if let threads::ThreadState::Waiting(threads::WaitingReason::Join(
                joined_thread_ids,
                _,
            )) = &mut current_tcb.state
            {
                joined_thread_ids.insert(arg0 as usize);
            } else {
                let timeout = {
                    if arg1 > 0 {
                        let current_time = system_timer::get_current_real_time() as usize;
                        Some(current_time + arg1 as usize)
                    } else {
                        None
                    }
                };
                let mut joined_thread_ids = alloc::collections::btree_set::BTreeSet::new();
                joined_thread_ids.insert(arg0 as usize);
                current_tcb.state = threads::ThreadState::Waiting(threads::WaitingReason::Join(
                    joined_thread_ids,
                    timeout,
                ));
            }

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
unsafe extern "C" fn PrefetchAbort(lr: usize) {
    error!("prefetch abort handler");
    assert!(processor::get_processor_mode() == ProcessorMode::System);

    panic!("prefetch abort at {:#X}", lr - 4);
}

#[rost_macros::exception]
unsafe extern "C" fn DataAbort(lr: usize) {
    error!("data abort handler");
    assert!(processor::get_processor_mode() == ProcessorMode::System);

    panic!(
        "data abort at {:#X} for address {:#X}",
        lr - 4,
        memory::mc_get_abort_address() // doesn't work in the emulator
    );
}
