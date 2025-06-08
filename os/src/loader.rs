// src/loader.rs
use core::arch::asm;

use crate::trap::{TrapContext, trap_handler};
use crate::task::TaskContext;
use crate::config::*;
use crate::mm::KERNEL_SPACE;    // bring the kernel page‐table into scope

#[repr(align(4096))]
#[derive(Copy, Clone)]
struct KernelStack {
    data: [u8; KERNEL_STACK_SIZE],
}

#[repr(align(4096))]
#[derive(Copy, Clone)]
struct UserStack {
    data: [u8; USER_STACK_SIZE],
}

static KERNEL_STACK: [KernelStack; MAX_APP_NUM] = [
    KernelStack { data: [0; KERNEL_STACK_SIZE] };
    MAX_APP_NUM
];

static USER_STACK: [UserStack; MAX_APP_NUM] = [
    UserStack { data: [0; USER_STACK_SIZE] };
    MAX_APP_NUM
];

impl KernelStack {
    fn get_sp(&self) -> usize {
        self.data.as_ptr() as usize + KERNEL_STACK_SIZE
    }
    pub fn push_context(
        &self,
        trap_cx: TrapContext,
        task_cx: TaskContext
    ) -> &'static mut TaskContext {
        unsafe {
            // lay out TrapContext then TaskContext on this stack
            let trap_ptr = (self.get_sp() - core::mem::size_of::<TrapContext>()) as *mut TrapContext;
            *trap_ptr = trap_cx;
            let task_ptr = (trap_ptr as usize - core::mem::size_of::<TaskContext>()) as *mut TaskContext;
            *task_ptr = task_cx;
            task_ptr.as_mut().unwrap()
        }
    }
}

impl UserStack {
    fn get_sp(&self) -> usize {
        self.data.as_ptr() as usize + USER_STACK_SIZE
    }
}

fn get_base_i(app_id: usize) -> usize {
    APP_BASE_ADDRESS + app_id * APP_SIZE_LIMIT
}

pub fn load_apps() {
    extern "C" { fn _num_app(); }
    let num_app = get_num_app();
    let ptr = _num_app as usize as *const usize;
    let app_start = unsafe { core::slice::from_raw_parts(ptr.add(1), num_app + 1) };
    unsafe { asm!("fence.i"); }
    for i in 0..num_app {
        let base = get_base_i(i);
        (base..base + APP_SIZE_LIMIT).for_each(|addr| unsafe { (addr as *mut u8).write_volatile(0) });
        let src = unsafe { core::slice::from_raw_parts(app_start[i] as *const u8, app_start[i+1]-app_start[i]) };
        let dst = unsafe { core::slice::from_raw_parts_mut(base as *mut u8, src.len()) };
        dst.copy_from_slice(src);
    }
}

pub fn init_app_cx(app_id: usize) -> &'static mut TaskContext {
    // compute top of this app's kernel stack:
    let kernel_stack_top = KERNEL_STACK[app_id].get_sp();

    // build initial trap context:
    let trap_cx = TrapContext::app_init_context(
        get_base_i(app_id),
        USER_STACK[app_id].get_sp(),
        KERNEL_SPACE.exclusive_access().token(),
        kernel_stack_top,
        trap_handler as usize,
    );

    // build the TaskContext that will jump back from a trap:
    let task_cx = TaskContext::goto_trap_return(kernel_stack_top);

    // push both onto the kernel stack and return the pointer to TaskContext:
    KERNEL_STACK[app_id].push_context(trap_cx, task_cx)
}

pub fn get_num_app() -> usize {
    extern "C" { fn _num_app(); }
    unsafe {
        let ptr = _num_app as usize as *const usize;
        ptr.read_volatile()
    }
}

pub fn get_app_data(app_id: usize) -> &'static [u8] {
    extern "C" { fn _num_app(); }
    let num_app = get_num_app();
    let ptr = _num_app as usize as *const usize;
    let app_start = unsafe { core::slice::from_raw_parts(ptr.add(1), num_app + 1) };
    assert!(app_id < num_app);
    unsafe {
        core::slice::from_raw_parts(
            app_start[app_id] as *const u8,
            app_start[app_id + 1] - app_start[app_id]
        )
    }
}