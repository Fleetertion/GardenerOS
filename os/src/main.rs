#![no_std]
#![no_main]

#[macro_use]
mod console;
use crate::console::sys_exit;
use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
extern "C" fn _start() -> ! {
    println!("Hello, world!");
    sys_exit(0);
}
