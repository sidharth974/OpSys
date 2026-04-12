use crate::serial_println;
use opsys_api::syscall::*;
use opsys_api::error::Error;

/// Main syscall dispatcher. Called from the assembly entry point.
/// Returns the result in RAX.
#[unsafe(no_mangle)]
pub extern "C" fn syscall_dispatch(
    nr: u64,
    arg0: u64,
    arg1: u64,
    _arg2: u64,
    _arg3: u64,
    _arg4: u64,
) -> u64 {
    let result = match nr as usize {
        SYS_DEBUG_PRINT => sys_debug_print(arg0, arg1),
        SYS_EXIT => sys_exit(arg0),
        SYS_YIELD => sys_yield(),
        _ => Err(Error::InvalidSyscall),
    };

    match result {
        Ok(val) => val as u64,
        Err(e) => e as i64 as u64,
    }
}

/// Debug print: write a string from userspace to the serial console.
/// arg0 = pointer to string buffer, arg1 = length
fn sys_debug_print(buf_ptr: u64, len: u64) -> Result<usize, Error> {
    // Safety: In a real kernel, we'd validate the user pointer is in user address space.
    // For now, we trust it (kernel threads share the address space).
    let len = len as usize;
    if len > 4096 {
        return Err(Error::InvalidArg);
    }
    let buf = unsafe { core::slice::from_raw_parts(buf_ptr as *const u8, len) };
    if let Ok(s) = core::str::from_utf8(buf) {
        serial_println!("{}", s);
    }
    Ok(0)
}

/// Exit the current thread.
fn sys_exit(_code: u64) -> Result<usize, Error> {
    use crate::proc::scheduler::SCHEDULER;
    use crate::proc::thread::ThreadState;

    let mut sched = SCHEDULER.lock();
    let tid = sched.current_tid();
    if let Some(t) = sched.thread_mut(tid) {
        t.state = ThreadState::Dead;
    }
    drop(sched);

    // Yield — scheduler won't re-enqueue a dead thread
    unsafe { crate::proc::scheduler::do_schedule(); }

    Ok(0)
}

/// Voluntary yield.
fn sys_yield() -> Result<usize, Error> {
    unsafe { crate::proc::scheduler::do_schedule(); }
    Ok(0)
}
