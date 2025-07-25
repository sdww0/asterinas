use alloc::fmt::format;
use core::arch::asm;

use ostd::{
    mm::{FrameAllocOptions, Vaddr, VmIo, VmSpace, MAX_USERSPACE_VADDR, PAGE_SIZE},
    task::disable_preempt,
};
use owo_colors::OwoColorize;

pub fn main_test() {
    let guard = ostd::trap::irq::disable_local();
    ostd::early_println!("[kernel] Security test started.");
    // access_illegal_address();
    ostd::early_println!("--------------------------------------");
    ostd::early_println!("[Security] Testing out of bound io memory access.");
    out_of_bounds_io_memory_access();
    ostd::early_println!("--------------------------------------");
    ostd::early_println!("[Security] Testing out of bound memory access.");
    out_of_bound_memory_access();
    ostd::early_println!("--------------------------------------");
    ostd::early_println!("[Security] Testing illegal page table modification.");
    illegal_page_table_modification();
    ostd::early_println!("--------------------------------------");
    ostd::early_println!("[Security] Testing illegal device DMA access.");
    drop(guard);
    trigger_wrong_dma_buf();
    let guard = ostd::trap::irq::disable_local();
    ostd::early_println!("--------------------------------------");
    ostd::early_println!("[Security] Testing stack overflow.");
    let current_task = ostd::task::Task::current().unwrap();
    ostd::early_println!(
        "[Security] Stack range of current task: {}..{}",
        format(format_args!("{:#x}", current_task.stack_start()))
            .as_str()
            .green(),
        format(format_args!("{:#x}", current_task.stack_end()))
            .as_str()
            .green()
    );
    stack_overflow_test(0, current_task.stack_start());
    drop(guard);

    panic!()
}

fn trigger_wrong_dma_buf() {
    let devices = aster_console::all_devices();
    devices.iter().for_each(|(name, device)| {
        ostd::early_println!("[Security] Testing trigger_wrong_dma_buf, device: {}", name);
        let result = device.trigger_wrong_dma_buf();
    });
}

fn out_of_bounds_io_memory_access() {
    let devices = aster_console::all_devices();
    devices.iter().for_each(|(name, device)| {
        ostd::early_println!(
            "[Security] Testing out_of_bounds_io_memory_access, device: {}",
            name
        );
        let result = device.out_of_bounds_io_memory_access();
        ostd::early_println!(
            "[Security] Device: {}, out_of_bounds_io_memory_access result: {:?}",
            name,
            format(format_args!("{:?}", result)).as_str().red()
        );
    });
}

fn out_of_bound_memory_access() {
    let frame = FrameAllocOptions::new().alloc_frame().unwrap();
    ostd::early_println!(
        "[Security] Try to write [0,0,0,0] at 2 * 4096 = {} offset, Frame size: {}",
        format(format_args!("{}", 2 * PAGE_SIZE)).as_str().yellow(),
        format(format_args!("{:?}", frame.size())).as_str().green()
    );
    let result = frame.write_bytes(2 * PAGE_SIZE, &[0, 0, 0, 0]);

    ostd::early_println!(
        "[Security] Out of bound memory access result: {:?}",
        format(format_args!("{:?}", result)).as_str().red()
    );
}

fn illegal_page_table_modification() {
    let vm_space = VmSpace::new();

    let preempt_guard = disable_preempt();
    ostd::early_println!(
        "[Security] Modifying page table for Upper max_userspace_vaddr, range: {}",
        format(format_args!(
            "{:#x}..{:#x}",
            MAX_USERSPACE_VADDR,
            MAX_USERSPACE_VADDR + 1024 * PAGE_SIZE
        ))
        .as_str()
        .yellow(),
    );

    if let Err(err) = vm_space.cursor_mut(
        &preempt_guard,
        &(MAX_USERSPACE_VADDR..MAX_USERSPACE_VADDR + 1024 * PAGE_SIZE),
    ) {
        ostd::early_println!(
            "[Security] Modification failed, error: {:?}",
            format(format_args!("{:?}", err)).as_str().red()
        );
        return;
    }

    ostd::early_println!(
            "[Security] Cursor get succeeded, page fault modification for Upper max_userspace_vaddr succeeded"
        );
}

fn access_illegal_address() {
    unsafe {
        let value = *(0xffffdffffefe1000 as *const u32);
        panic!(
            "[Security] Accessed illegal address: {:#x}, value: {}",
            0xffffdffffefe1000u64, value
        );
    }
}

fn get_stack_pointer() -> usize {
    let sp: usize;
    unsafe {
        asm!("mov {}, rsp", out(reg) sp);
    }
    sp
}

/// In stack overflow test, we recursively call the function until a stack overflow occurs.
/// If the kernel stack is created with a guard page, the iteration will stop at the guard page.
/// If the kernel stack is created without a guard page, the iteration will stop until it reach a page without mapping.
///
/// To determine whether the stack overflow test is successful, one can:
/// 1. Create a kernel stack with a guard page and run the test.
/// 2. The output will show the iteration times A and the stack pointer.
/// 3. Create a kernel stack without a guard page and run the test.
/// 4. The output will show the iteration times B and the stack pointer.
/// 5. If A is less than B, the test is successful, indicating that the guard page effectively prevented the stack overflow
/// from corrupting other's kernel stack.
///
fn stack_overflow_test(time: usize, current_task_stack_start: Vaddr) {
    ostd::early_println!(
        "[Security] Stack overflow test, current time: {}, stack pointer: {:#x?}",
        time,
        get_stack_pointer()
    );
    if get_stack_pointer() < current_task_stack_start {
        ostd::early_println!(
            "[Security] Stack overflow test stopped at time: {}, Current stack pointer is out of range! Current stack pointer: {:#x?}",
            time,
            format(format_args!("{:#x?}", get_stack_pointer())).as_str().red()
        );
        return;
    }

    let mut buffer = [0u8; 4096];
    buffer[0] = 1;
    buffer[1022] = 222;
    stack_overflow_test(time + 1, current_task_stack_start);

    buffer[0] = 2;
    ostd::early_println!("buffer[0]: {}, buffer[1022]: {}", buffer[0], buffer[1022]);
}
