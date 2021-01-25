use crate::helpers::{read_register, read_register_bit, write_register};

struct ST;
#[allow(dead_code)]
impl ST {
    /// System Timer base address
    const BASE_ADDRESS: u32 = 0xFFFFFD00;

    /// ST period interval mode register offset
    const PIMR: u32 = 0x4;

    /// ST Real-time Mode Register
    const RTMR: u32 = 0x000C;

    /// ST Status Register offset
    const SR: u32 = 0x10;

    /// ST interrupt enable register offset
    const IER: u32 = 0x14;

    /// ST intterupt mask register offset
    const IMR: u32 = 0x1C;

    //// ST Current Real-time Register offset
    const CRTR: u32 = 0x24;
}

const ST_CLOCK_HZ: usize = 32768;

static mut ST_PERIOD_INTERRUPT_INTERVAL: core::time::Duration =
    core::time::Duration::from_millis(10);
static mut ST_REALTIME_UNIT_INTERVAL: core::time::Duration = core::time::Duration::from_millis(10);

#[allow(dead_code)]
pub fn get_period_timer_interval() -> core::time::Duration {
    unsafe { ST_PERIOD_INTERRUPT_INTERVAL }
}

pub fn get_real_time_unit_interval() -> core::time::Duration {
    unsafe { ST_REALTIME_UNIT_INTERVAL }
}

/// Enables periodic timer interrupt and sets the periodic counter interval to given value.
/// Maximum duration between ticks is 2 seconds and minimum is 1 millisecond
pub fn init_system_timer_interrupt(time_between_ticks: core::time::Duration) {
    // enable system interrupt
    write_register(ST::BASE_ADDRESS, ST::IER, 0x1);

    if time_between_ticks > core::time::Duration::from_secs(2)
        || time_between_ticks < core::time::Duration::from_millis(1)
    {
        panic!("invalid argument: time_between_ticks");
    }
    // set interval
    // Values are counted in downwards a 16-bit register, therefore minimal period is with 1,
    // maximum period is with value 0 because of overflow.
    let counter_value = (ST_CLOCK_HZ * time_between_ticks.as_millis() as usize) / 1000;
    unsafe {
        ST_PERIOD_INTERRUPT_INTERVAL = time_between_ticks;
    }
    write_register(ST::BASE_ADDRESS, ST::PIMR, counter_value as u32);
}

/// Sets the duration for a real time unit
pub fn set_real_time_timer_interval(interval_duration: core::time::Duration) {
    // The real timer is built around a 20-bit counter fed by Slow Clock divided by a programmable value.
    // At reset, this value is set to 0x8000, corresponding to feeding the real-time counter
    // with a 1 Hz signal when the Slow Clock is 32.768 Hz
    let interval_value = (ST_CLOCK_HZ * interval_duration.as_millis() as usize) / 1000;
    unsafe {
        ST_REALTIME_UNIT_INTERVAL = interval_duration;
    }
    write_register(ST::BASE_ADDRESS, ST::RTMR, interval_value as u32);
}

pub fn get_current_real_time() -> u32 {
    read_register(ST::BASE_ADDRESS, ST::CRTR)
}

pub fn get_current_real_time_as_duration() -> core::time::Duration {
    core::time::Duration::from_millis(
        get_current_real_time() as u64 * get_real_time_unit_interval().as_millis() as u64,
    )
}

pub fn has_system_timer_elapsed() -> bool {
    read_register_bit(ST::BASE_ADDRESS, ST::SR, 0) != 0
}

pub fn get_periodic_interrupts_enabled() -> bool {
    read_register_bit(ST::BASE_ADDRESS, ST::IMR, 0) != 0
}
