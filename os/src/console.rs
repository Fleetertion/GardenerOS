use core::arch::asm;
use core::fmt::{self, Write};

/// 发起系统调用，返回值存放在 x10
fn syscall(id: usize, args: [usize; 3]) -> isize {
    let mut ret: isize;
    unsafe {
        asm!(
            "ecall",
            in("x17") id,
            in("x10") args[0],
            in("x11") args[1],
            in("x12") args[2],
            lateout("x10") ret,
        );
    }
    ret
}

/// 退出进程（永不返回）
pub fn sys_exit(code: i32) -> ! {
    const SYSCALL_EXIT: usize = 93;
    syscall(SYSCALL_EXIT, [code as usize, 0, 0]);
    loop {}
}

/// 写字符串到文件描述符，返回写入字节数
pub fn sys_write(fd: usize, buf: &[u8]) -> isize {
    const SYSCALL_WRITE: usize = 64;
    syscall(SYSCALL_WRITE, [fd, buf.as_ptr() as usize, buf.len()])
}

/// 实现 Write trait，用于核心库的格式化输出
struct Stdout;

impl Write for Stdout {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        sys_write(1, s.as_bytes());
        Ok(())
    }
}

/// 底层打印函数，接受 format_args! 生成的参数
pub fn print(args: fmt::Arguments) {
    Stdout.write_fmt(args).unwrap();
}

/// 导出 print! 宏，支持任意格式化字符串
#[macro_export]
macro_rules! print {
    ($($tokens:tt)*) => {
        $crate::console::print(format_args!($($tokens)*));
    };
}

/// 导出 println! 宏，自动追加换行符，支持多种调用形式
#[macro_export]
macro_rules! println {
    () => {
        $crate::console::print(format_args!("\n"));
    };
    ($fmt:expr) => {
        $crate::console::print(format_args!(concat!($fmt, "\n")));
    };
    ($fmt:expr, $($arg:tt)+) => {
        $crate::console::print(format_args!(concat!($fmt, "\n"), $($arg)+));
    };
}

