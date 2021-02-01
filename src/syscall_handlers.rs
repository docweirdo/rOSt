use crate::{system_timer, threads};
use alloc::boxed::Box;
use core::{alloc::Layout, convert::TryFrom};
use log::trace;
use rost_api::syscalls;
use rost_api::syscalls::Syscalls;
use threads::ThreadState;

fn yield_thread() -> usize {
    trace!("syscall: YieldThread");
    super::threads::schedule(None);
    0
}

fn create_thread(fat_pointer_part1: usize, fat_pointer_part2: usize) -> usize {
    trace!("syscall: CreateThread");
    unsafe {
        let raw_entry: *mut dyn FnMut() =
            core::mem::transmute((fat_pointer_part1, fat_pointer_part2));
        let entry = Box::<dyn FnMut() + 'static>::from_raw(raw_entry);
        super::threads::create_thread_internal(entry)
    }
}

fn exit_thread() -> usize {
    trace!("syscall: ExitThread");
    super::threads::exit_internal();
    0
}

fn receive_dbgu(blocking: bool) -> usize {
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

    if blocking {
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

fn send_dbgu(character: char) -> usize {
    trace!("syscall: SendDBGU");
    super::dbgu::write_char(character);
    0
}

fn allocate(size: usize, align: usize) -> usize {
    trace!("syscall: Allocate");
    let layout = Layout::from_size_align(size, align).expect("Bad layout");
    unsafe { alloc::alloc::alloc(layout) as usize }
}

fn deallocate(ptr: *mut u8, size: usize, align: usize) -> usize {
    trace!("syscall: Deallocate");
    let layout = Layout::from_size_align(size, align).expect("Bad layout");

    unsafe {
        alloc::alloc::dealloc(ptr, layout);
    }
    0
}

fn get_current_real_time() -> usize {
    trace!("syscall: GetCurrentRealTime");
    system_timer::get_current_real_time() as usize
}

fn subscribe_thread_service(service_id: usize) -> usize {
    trace!("syscall: Subscribe");
    let current_tcb = threads::get_current_thread();
    let service =
        rost_api::syscalls::ThreadServices::try_from(service_id as u32).expect("invalid service given");

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

fn unsubscribe_thread_service(service_id: usize) -> usize {
    trace!("syscall: Unsubscribe");
    let current_tcb = threads::get_current_thread();
    let service = rost_api::syscalls::ThreadServices::try_from(service_id as u32).expect("invalid service given");

    if current_tcb.subscribed_services.remove(&service) == None {
        panic!(
            "syscall Unsubscribe: Service {:?} was not subscribed",
            service
        );
    }
    0
}

fn sleep(time_ms: usize) -> usize {
    trace!("syscall: Sleep");
    let current_time = system_timer::get_current_real_time() as usize;
    let current_tcb = threads::get_current_thread();

    let time_in_realtime_units: usize = time_ms / system_timer::get_real_time_unit_interval().as_millis() as usize;

    current_tcb.state = threads::ThreadState::Waiting(threads::WaitingReason::Sleep(
        current_time + time_in_realtime_units,
    ));

    threads::schedule(None);

    system_timer::get_real_time_unit_interval().as_millis() as usize
        * (system_timer::get_current_real_time() as usize - current_time as usize)
}

fn join_thread(thread_id: usize, timeout_ms: usize) -> usize {
    trace!("syscall: JoinThread");
    let join_thread = {
        let thread = threads::get_thread_by_id(thread_id);
        if threads::get_thread_by_id(thread_id).is_none() {
            return 0;
        }
        thread.unwrap()
    };

    if join_thread.state == ThreadState::Stopped {
        return 0;
    }

    let current_tcb = threads::get_current_thread();

    if join_thread.parent_thread_id != current_tcb.id {
        panic!(
            "JoinThread: you cannot join a thread which is not your parent thread: p: {} != {}",
            join_thread.parent_thread_id, current_tcb.id
        );
    }

    if let threads::ThreadState::Waiting(threads::WaitingReason::Join(joined_thread_ids, _)) =
        &mut current_tcb.state
    {
        joined_thread_ids.insert(thread_id);
    } else {
        let timeout = {
            if timeout_ms > 0 {
                let current_time = system_timer::get_current_real_time() as usize;
                Some(current_time + timeout_ms)
            } else {
                None
            }
        };
        let mut joined_thread_ids = alloc::collections::btree_set::BTreeSet::new();
        joined_thread_ids.insert(thread_id);
        current_tcb.state =
            threads::ThreadState::Waiting(threads::WaitingReason::Join(joined_thread_ids, timeout));
    }

    threads::schedule(None);
    0
}

pub fn syscall_handler(arg0: usize, arg1: usize, arg2: usize, service_id: usize) -> usize {
    match Syscalls::try_from(service_id as u32) {
        Ok(Syscalls::YieldThread) => yield_thread(),
        Ok(Syscalls::CreateThread) => create_thread(arg0, arg1),
        Ok(Syscalls::ExitThread) => exit_thread(),
        Ok(Syscalls::ReceiveDBGU) => receive_dbgu(arg0 != 0),
        Ok(Syscalls::SendDBGU) => send_dbgu(arg0 as u8 as char),
        Ok(Syscalls::Allocate) => allocate(arg0, arg1),
        Ok(Syscalls::Deallocate) => deallocate(arg0 as *mut u8, arg1, arg2),
        Ok(Syscalls::GetCurrentRealTime) => get_current_real_time(),
        Ok(Syscalls::Subscribe) => subscribe_thread_service(arg0),
        Ok(Syscalls::Unsubscribe) => unsubscribe_thread_service(arg0),
        Ok(Syscalls::Sleep) => sleep(arg0),
        Ok(Syscalls::JoinThread) => join_thread(arg0, arg1),
        _ => {
            log::error!("unknown syscall id {}", service_id);
            panic!()
        }
    }
}
