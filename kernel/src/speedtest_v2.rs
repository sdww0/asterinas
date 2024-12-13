use core::{hint::spin_loop, time::Duration};

use ostd::{
    arch::{qemu::exit_qemu, read_tsc},
    early_println,
    io_mem::IoMem,
    mm::{paddr_to_vaddr, FrameAllocOptions, HasPaddr, PageFlags, VmIo, VmIoOnce},
    task::{kernel_stack::KernelStack, TaskOptions},
};

use crate::{prelude::*, read_write::__memcpy_fallible, time::SystemTime};

const BASE_UNIT: usize = 16;
const TEST_TIMES: usize = 100;

pub fn test() {
    crate::time::init();
    do_small_rw_test();
    // do_multiple_large_rw_test();
    // do_kernel_stack_test();
    loop {}
}

fn do_multiple_large_rw_test() {
    let test_base_units = vec![16, 64, 256, 1024, 2048, 4096, 16384, 65536];

    let mut results = Vec::with_capacity(16);

    for base_unit in test_base_units.iter() {
        let res = do_single_large_rw_test(*base_unit);
        results.push(res);
    }

    for (index, res) in results.iter().enumerate() {
        early_println!(
            "Base unit: {:>6}. Result: [Read] {:>10.3} , [Write] {:>10.3}",
            (test_base_units.get(index).unwrap()),
            res.0,
            res.1
        );
    }
}

fn do_kernel_stack_test() {
    early_println!("Doing kernel stack test");
    const WARM_UP: usize = 2;

    let mut durations: [f64; TEST_TIMES + WARM_UP] = [0.0; TEST_TIMES + WARM_UP];

    for i in 0..(TEST_TIMES + WARM_UP) {
        let test = SystemTime::now();
        while SystemTime::now().duration_since(&test).unwrap().as_millis() < 1 {
            spin_loop();
        }
        durations[i] = unsafe { kernel_stack_test() };
        early_println!("Next iter: {}", i);
    }
    early_println!("Duration: {:#?}", durations);

    let mut duration = 0.0;
    for i in WARM_UP..(TEST_TIMES + WARM_UP) {
        duration += durations[i];
    }

    duration /= TEST_TIMES as f64;

    early_println!("Average TSC time: {:?}", duration);
}

fn do_single_large_rw_test(base_unit: usize) -> (f64, f64) {
    early_println!("Doing single large read write test");

    let mut durations: [(f64, f64); TEST_TIMES + 1] = [(0.0, 0.0); (TEST_TIMES + 1)];

    for i in 0..(TEST_TIMES + 1) {
        durations[i] = unsafe { segment_test(base_unit) };
    }
    early_println!("Duration: {:#?}", durations);

    let mut read_duration = 0.0;
    let mut write_duration = 0.0;
    for i in 1..(TEST_TIMES + 1) {
        read_duration += durations[i].0;
        write_duration += durations[i].1;
    }

    read_duration /= TEST_TIMES as f64;
    write_duration /= TEST_TIMES as f64;

    (read_duration, write_duration)
}

fn do_small_rw_test() {
    let mut tsc_values: [(f64, f64); TEST_TIMES * 4] = [(0.0, 0.0); TEST_TIMES * 4];

    early_println!("Doing small read write test");

    for i in 0..TEST_TIMES {
        let test = SystemTime::now();
        while SystemTime::now().duration_since(&test).unwrap().as_secs() < 1 {
            spin_loop();
        }
        let (u8_duration, u16_duration, u32_duration, u64_duration) =
            unsafe { io_mem_with_device_test() };
        tsc_values[i] = u8_duration;
        tsc_values[i + TEST_TIMES] = u16_duration;
        tsc_values[i + 2 * TEST_TIMES] = u32_duration;
        tsc_values[i + 3 * TEST_TIMES] = u64_duration;
    }
    early_println!("Duration: {:#?}", tsc_values);

    let (mut u8_duration, mut u16_duration, mut u32_duration, mut u64_duration) =
        (0.0, 0.0, 0.0, 0.0);

    for i in 1..TEST_TIMES {
        u8_duration += tsc_values[i].0;
        u16_duration += tsc_values[i + TEST_TIMES].0;
        u32_duration += tsc_values[i + 2 * TEST_TIMES].0;
        u64_duration += tsc_values[i + 3 * TEST_TIMES].0;
    }

    u8_duration /= TEST_TIMES as f64;
    u16_duration /= TEST_TIMES as f64;
    u32_duration /= TEST_TIMES as f64;
    u64_duration /= TEST_TIMES as f64;

    early_println!(" u8 read average TSC time: {:?}", u8_duration);
    early_println!(" u16 read average TSC time: {:?}", u16_duration);
    early_println!(" u32 read average TSC time: {:?}", u32_duration);
    early_println!(" u64 read average TSC time: {:?}", u64_duration);

    let (mut u8_duration, mut u16_duration, mut u32_duration, mut u64_duration) =
        (0.0, 0.0, 0.0, 0.0);

    for i in 1..TEST_TIMES {
        u8_duration += tsc_values[i].1;
        u16_duration += tsc_values[i + TEST_TIMES].1;
        u32_duration += tsc_values[i + 2 * TEST_TIMES].1;
        u64_duration += tsc_values[i + 3 * TEST_TIMES].1;
    }

    u8_duration /= TEST_TIMES as f64;
    u16_duration /= TEST_TIMES as f64;
    u32_duration /= TEST_TIMES as f64;
    u64_duration /= TEST_TIMES as f64;

    early_println!(" u8 write average TSC time: {:?}", u8_duration);
    early_println!(" u16 write average TSC time: {:?}", u16_duration);
    early_println!(" u32 write average TSC time: {:?}", u32_duration);
    early_println!(" u64 write average TSC time: {:?}", u64_duration);
}

unsafe fn segment_test(base_unit: usize) -> (f64, f64) {
    let segment = FrameAllocOptions::new(400 * 4).alloc_contiguous().unwrap();

    const MAX: usize = 65536;
    static mut VALUE: [u8; MAX] = [1; MAX];

    const ITERATION: usize = 10000;

    const RW_ITER: usize = 100;

    let mut slice = &mut VALUE[0..base_unit];

    let start = read_tsc();
    for _i in 0..ITERATION {
        for i in 0..RW_ITER {
            segment.read_bytes(i * base_unit, &mut slice).unwrap();
        }
    }
    let val1 = read_tsc() - start;

    let start = read_tsc();
    for _i in 0..ITERATION {
        for i in 0..RW_ITER {
            segment.write_bytes(i * base_unit, &slice).unwrap();
        }
    }
    let val2 = read_tsc() - start;

    drop(segment);
    (
        val1 as f64 / (ITERATION * RW_ITER) as f64,
        val2 as f64 / (ITERATION * RW_ITER) as f64,
    )
}

fn kernel_stack_test() -> f64 {
    const ITERATION: usize = 100000;

    let mut stacks = Vec::with_capacity(ITERATION);

    let start = read_tsc();
    for _i in 0..ITERATION {
        stacks.push(KernelStack::new_with_guard_page());
    }
    let res = read_tsc() - start;
    drop(stacks);
    res as f64 / ITERATION as f64
}

fn io_mem_with_device_test() -> ((f64, f64), (f64, f64), (f64, f64), (f64, f64)) {
    let mut io_mem = None;
    for (_, dev) in aster_block::all_devices() {
        io_mem = dev.get_io_mem();
    }

    let io_mem = io_mem.unwrap();

    // We simulate the operation to the virtio device.
    // u8: device_status = 15 (offset = 20)
    // u16: queue_select = 0 (offset = 22)
    // u32: device_feature_select = 0 (offset = 0)
    // u64: queue_desc = (Allocate a frame).paddr (offset = 32)

    const ITERATION: usize = 100000;

    let mut u8_read_tsc = 0;
    let mut u16_read_tsc = 0;
    let mut u32_read_tsc = 0;
    let mut u64_read_tsc = 0;

    let mut u8_write_tsc = 0;
    let mut u16_write_tsc = 0;
    let mut u32_write_tsc = 0;
    let mut u64_write_tsc = 0;

    let start = read_tsc();
    for _i in 0..ITERATION {
        io_mem.read_once::<u8>(20).unwrap();
    }
    u8_read_tsc = read_tsc() - start;

    let start = read_tsc();
    for _i in 0..ITERATION {
        io_mem.write_once(20, &15u8).unwrap();
    }
    u8_write_tsc = read_tsc() - start;

    let start = read_tsc();
    for _i in 0..ITERATION {
        io_mem.read_once::<u16>(22).unwrap();
    }
    u16_read_tsc = read_tsc() - start;

    let start = read_tsc();
    for _i in 0..ITERATION {
        io_mem.write_once(22, &0u16).unwrap();
    }
    u16_write_tsc = read_tsc() - start;

    let start = read_tsc();
    for _i in 0..ITERATION {
        io_mem.read_once::<u32>(0).unwrap();
    }
    u32_read_tsc = read_tsc() - start;

    let start = read_tsc();
    for _i in 0..ITERATION {
        io_mem.write_once(0, &0u32).unwrap();
    }
    u32_write_tsc = read_tsc() - start;

    let frame = FrameAllocOptions::new(1).alloc_single().unwrap();
    let paddr = frame.paddr() as u64;

    let start = read_tsc();
    for _i in 0..ITERATION {
        io_mem.read_once::<u64>(32).unwrap();
    }
    u64_read_tsc = read_tsc() - start;

    let start = read_tsc();
    for _i in 0..ITERATION {
        io_mem.write_once(32, &paddr).unwrap();
    }
    u64_write_tsc = read_tsc() - start;

    (
        (
            u8_read_tsc as f64 / (ITERATION) as f64,
            u8_write_tsc as f64 / (ITERATION) as f64,
        ),
        (
            u16_read_tsc as f64 / (ITERATION) as f64,
            u16_write_tsc as f64 / (ITERATION) as f64,
        ),
        (
            u32_read_tsc as f64 / (ITERATION) as f64,
            u32_write_tsc as f64 / (ITERATION) as f64,
        ),
        (
            u64_read_tsc as f64 / (ITERATION) as f64,
            u64_write_tsc as f64 / (ITERATION) as f64,
        ),
    )
}

fn io_mem_with_memory_test() -> (f64, f64, f64, f64) {
    const ITERATION: usize = 100000;
    const RW_ITER: usize = 100;

    let frame = FrameAllocOptions::new(1).alloc_single().unwrap();

    let io_mem = unsafe {
        IoMem::new(
            frame.start_paddr()..frame.end_paddr(),
            PageFlags::RW,
            ostd::mm::CachePolicy::Writeback,
        )
    };

    let u8_tsc;
    let u16_tsc;
    let u32_tsc;
    let u64_tsc;

    let start = read_tsc();
    for _i in 0..ITERATION {
        for i in 0..RW_ITER {
            io_mem.read_once::<u8>(i * 1).unwrap();
            io_mem.write_once(i * 1, &1u8).unwrap();
        }
    }
    u8_tsc = read_tsc() - start;

    let start = read_tsc();
    for _i in 0..ITERATION {
        for i in 0..RW_ITER {
            io_mem.read_once::<u16>(i * 2).unwrap();
            io_mem.write_once(i * 2, &1u16).unwrap();
        }
    }
    u16_tsc = read_tsc() - start;

    let start = read_tsc();
    for _i in 0..ITERATION {
        for i in 0..RW_ITER {
            io_mem.read_once::<u32>(i * 4).unwrap();
            io_mem.write_once(i * 4, &1u32).unwrap();
        }
    }
    u32_tsc = read_tsc() - start;

    let start = read_tsc();
    for _i in 0..ITERATION {
        for i in 0..RW_ITER {
            io_mem.read_once::<u64>(i * 8).unwrap();
            io_mem.write_once(i * 8, &1u64).unwrap();
        }
    }
    u64_tsc = read_tsc() - start;

    (
        u8_tsc as f64 / (ITERATION * RW_ITER) as f64,
        u16_tsc as f64 / (ITERATION * RW_ITER) as f64,
        u32_tsc as f64 / (ITERATION * RW_ITER) as f64,
        u64_tsc as f64 / (ITERATION * RW_ITER) as f64,
    )
}
