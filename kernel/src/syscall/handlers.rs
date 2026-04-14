use crate::serial_println;
use opsys_api::syscall::*;
use opsys_api::error::Error;

/// Main syscall dispatcher.
#[unsafe(no_mangle)]
pub extern "C" fn syscall_dispatch(
    nr: u64,
    arg0: u64,
    arg1: u64,
    arg2: u64,
    arg3: u64,
    _arg4: u64,
) -> u64 {
    let result = match nr as usize {
        SYS_DEBUG_PRINT => sys_debug_print(arg0, arg1),
        SYS_EXIT => sys_exit(arg0),
        SYS_YIELD => sys_yield(),
        SYS_OPEN => sys_open(arg0, arg1),
        SYS_READ => sys_read(arg0, arg1, arg2),
        SYS_WRITE => sys_write(arg0, arg1, arg2),
        SYS_CLOSE => sys_close(arg0),
        SYS_STAT => sys_stat(arg0, arg1, arg2),
        SYS_READDIR => sys_readdir(arg0, arg1, arg2),
        SYS_MKDIR => sys_mkdir(arg0, arg1),
        SYS_UNLINK => sys_unlink(arg0, arg1),
        SYS_GETPID => sys_getpid(),
        SYS_CLOCK_GETTIME => sys_clock_gettime(),
        SYS_UNAME => sys_uname(arg0, arg1),
        SYS_SYSINFO => sys_sysinfo(arg0, arg1),
        _ => Err(Error::InvalidSyscall),
    };

    match result {
        Ok(val) => val as u64,
        Err(e) => e as i64 as u64,
    }
}

fn sys_debug_print(buf_ptr: u64, len: u64) -> Result<usize, Error> {
    let len = len as usize;
    if len > 4096 { return Err(Error::InvalidArg); }
    let buf = unsafe { core::slice::from_raw_parts(buf_ptr as *const u8, len) };
    if let Ok(s) = core::str::from_utf8(buf) {
        serial_println!("{}", s);
    }
    Ok(0)
}

fn sys_exit(_code: u64) -> Result<usize, Error> {
    use crate::proc::scheduler::SCHEDULER;
    use crate::proc::thread::ThreadState;
    let mut sched = SCHEDULER.lock();
    let tid = sched.current_tid();
    if let Some(t) = sched.thread_mut(tid) {
        t.state = ThreadState::Dead;
    }
    drop(sched);
    unsafe { crate::proc::scheduler::do_schedule(); }
    Ok(0)
}

fn sys_yield() -> Result<usize, Error> {
    unsafe { crate::proc::scheduler::do_schedule(); }
    Ok(0)
}

/// Open a file by path. Returns a file descriptor (inode ID for now).
fn sys_open(path_ptr: u64, path_len: u64) -> Result<usize, Error> {
    let len = path_len as usize;
    if len > 1024 { return Err(Error::InvalidArg); }
    let buf = unsafe { core::slice::from_raw_parts(path_ptr as *const u8, len) };
    let path = core::str::from_utf8(buf).map_err(|_| Error::InvalidArg)?;

    let vfs = crate::fs::vfs::VFS.lock();
    vfs.resolve_path(path).ok_or(Error::NotFound)
}

/// Read from a file descriptor. Returns bytes read.
fn sys_read(fd: u64, buf_ptr: u64, buf_len: u64) -> Result<usize, Error> {
    let vfs = crate::fs::vfs::VFS.lock();
    let data = vfs.read_file(fd as usize).ok_or(Error::InvalidArg)?;
    let to_copy = (buf_len as usize).min(data.len());
    let dest = unsafe { core::slice::from_raw_parts_mut(buf_ptr as *mut u8, to_copy) };
    dest.copy_from_slice(&data[..to_copy]);
    Ok(to_copy)
}

/// Write to a file descriptor.
fn sys_write(fd: u64, buf_ptr: u64, buf_len: u64) -> Result<usize, Error> {
    let len = buf_len as usize;
    let buf = unsafe { core::slice::from_raw_parts(buf_ptr as *const u8, len) };
    let mut vfs = crate::fs::vfs::VFS.lock();
    if vfs.write_file(fd as usize, buf) {
        Ok(len)
    } else {
        Err(Error::InvalidArg)
    }
}

fn sys_close(_fd: u64) -> Result<usize, Error> {
    Ok(0) // No-op for now
}

/// Stat a file. Writes size to the pointer.
fn sys_stat(path_ptr: u64, path_len: u64, out_size: u64) -> Result<usize, Error> {
    let buf = unsafe { core::slice::from_raw_parts(path_ptr as *const u8, path_len as usize) };
    let path = core::str::from_utf8(buf).map_err(|_| Error::InvalidArg)?;
    let vfs = crate::fs::vfs::VFS.lock();
    let id = vfs.resolve_path(path).ok_or(Error::NotFound)?;
    let inode = vfs.stat(id).ok_or(Error::NotFound)?;
    if out_size != 0 {
        unsafe { *(out_size as *mut u64) = inode.size as u64; }
    }
    Ok(inode.itype as usize)
}

fn sys_readdir(fd: u64, buf_ptr: u64, buf_len: u64) -> Result<usize, Error> {
    let vfs = crate::fs::vfs::VFS.lock();
    let entries = vfs.list_dir(fd as usize).ok_or(Error::InvalidArg)?;
    // Write entry names separated by newlines
    let mut output = alloc::string::String::new();
    for (name, _, _) in &entries {
        output.push_str(name);
        output.push('\n');
    }
    let to_copy = (buf_len as usize).min(output.len());
    let dest = unsafe { core::slice::from_raw_parts_mut(buf_ptr as *mut u8, to_copy) };
    dest.copy_from_slice(&output.as_bytes()[..to_copy]);
    Ok(to_copy)
}

fn sys_mkdir(path_ptr: u64, path_len: u64) -> Result<usize, Error> {
    let buf = unsafe { core::slice::from_raw_parts(path_ptr as *const u8, path_len as usize) };
    let path = core::str::from_utf8(buf).map_err(|_| Error::InvalidArg)?;
    let mut vfs = crate::fs::vfs::VFS.lock();
    // Split parent/name
    let (parent_path, dir_name) = match path.rfind('/') {
        Some(pos) if pos > 0 => (&path[..pos], &path[pos+1..]),
        _ => ("/", path.trim_start_matches('/')),
    };
    let parent = vfs.resolve_path(parent_path).ok_or(Error::NotFound)?;
    let id = vfs.mkdir(parent, dir_name);
    Ok(id)
}

fn sys_unlink(path_ptr: u64, path_len: u64) -> Result<usize, Error> {
    let buf = unsafe { core::slice::from_raw_parts(path_ptr as *const u8, path_len as usize) };
    let path = core::str::from_utf8(buf).map_err(|_| Error::InvalidArg)?;
    let mut vfs = crate::fs::vfs::VFS.lock();
    if vfs.remove(path) { Ok(0) } else { Err(Error::NotFound) }
}

fn sys_getpid() -> Result<usize, Error> {
    let sched = crate::proc::scheduler::SCHEDULER.lock();
    Ok(sched.current_tid())
}

fn sys_clock_gettime() -> Result<usize, Error> {
    let ticks = crate::arch::x86_64::idt::TICKS.load(core::sync::atomic::Ordering::Relaxed);
    Ok(ticks as usize)
}

fn sys_uname(buf_ptr: u64, buf_len: u64) -> Result<usize, Error> {
    let info = b"OpSys 0.1.0 x86_64 Microkernel";
    let to_copy = (buf_len as usize).min(info.len());
    let dest = unsafe { core::slice::from_raw_parts_mut(buf_ptr as *mut u8, to_copy) };
    dest.copy_from_slice(&info[..to_copy]);
    Ok(to_copy)
}

fn sys_sysinfo(buf_ptr: u64, buf_len: u64) -> Result<usize, Error> {
    let free = crate::mm::pmm::free_count();
    let devs = crate::drivers::pci::PCI_DEVICES.lock().len();
    let info = alloc::format!(
        "mem_free={}\npci_devices={}\nuptime_ticks={}\n",
        free * 4096, devs,
        crate::arch::x86_64::idt::TICKS.load(core::sync::atomic::Ordering::Relaxed)
    );
    let to_copy = (buf_len as usize).min(info.len());
    let dest = unsafe { core::slice::from_raw_parts_mut(buf_ptr as *mut u8, to_copy) };
    dest.copy_from_slice(&info.as_bytes()[..to_copy]);
    Ok(to_copy)
}
