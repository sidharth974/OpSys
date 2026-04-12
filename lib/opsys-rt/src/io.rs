use core::fmt;
use super::syscall;

/// A writer that sends output to the kernel via sys_debug_print.
pub struct SyscallWriter;

impl fmt::Write for SyscallWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        syscall::debug_print(s);
        Ok(())
    }
}

/// Print via syscall. Used by userspace programs.
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::io::_print(format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    SyscallWriter.write_fmt(args).unwrap();
}
