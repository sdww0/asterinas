use core::hint::spin_loop;

use ostd::{
    arch::{
        qemu::{exit_qemu, QemuExitCode},
        read_tsc,
    }, cpu::{CpuId, CpuSet}, early_println, task::{Task, TaskOptions}
};

use crate::{sched::priority::Priority, thread::kernel_thread::ThreadOptions, time::SystemTime};

pub fn test() {

    let mut affinity = CpuSet::new_empty();
    affinity.add(CpuId::bsp());
    ThreadOptions::new(start_thread)
        .priority(Priority::idle())
        .cpu_affinity(affinity)
        .spawn();
}

pub fn start_thread() {
    TaskOptions::new(thread1).spawn().unwrap();

    let mut count_time = 0;
    const TOTAL_TIME: usize = 10000000;
    let mut total_tick: u64 = 0;

    while count_time < TOTAL_TIME {
        let before_tick = read_tsc();
        Task::yield_now();
        let after_tick = read_tsc();
        total_tick += (after_tick - before_tick);
        count_time += 1;
    }

    early_println!("Average tick: {:?}", total_tick as f64 / TOTAL_TIME as f64);

    let test = read_tsc();
    while (read_tsc() - test) < 2000000000 {
        spin_loop();
    }

    exit_qemu(QemuExitCode::Success);
}

pub fn thread1() {
    loop {
        Task::yield_now();
    }
}
