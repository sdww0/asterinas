use core::{hint::spin_loop, time::Duration};

use ostd::{
    arch::qemu::exit_qemu,
    early_println,
    io_mem::IoMem,
    mm::{paddr_to_vaddr, FrameAllocOptions, HasPaddr, PageFlags, VmIo, VmIoOnce},
    task::{kernel_stack::KernelStack, TaskOptions},
};

use crate::{prelude::*, read_write::__memcpy_fallible, time::SystemTime};

const BASE_UNIT: usize = 1;
const TEST_TIMES: usize = 100;

static mut VALUE: [u8; BASE_UNIT] = [1; BASE_UNIT];

pub fn test() {
    crate::time::init();
    do_io_mem_test();

    for i in 0..BASE_UNIT {
        unsafe {
            VALUE[i] = (i % 256) as u8;
        }
    }

    let mut durations: [Duration; TEST_TIMES] = [Duration::new(0, 0); TEST_TIMES];

    for i in 0..TEST_TIMES {
        durations[i] = unsafe { segment_test2() };
    }
    early_println!("Duration: {:#?}", durations);

    let mut duration = Duration::new(0, 0);
    for i in 0..TEST_TIMES {
        duration += durations[i];
    }

    duration /= (TEST_TIMES as u32);

    early_println!("Average time: {:?}", duration);

    loop {}
    exit_qemu(ostd::arch::qemu::QemuExitCode::Success);
}

fn do_io_mem_test() {
    let mut durations: [Duration; TEST_TIMES * 4] = [Duration::new(0, 0); TEST_TIMES * 4];

    early_println!("Doing io memory test");

    for i in 0..TEST_TIMES {
        // let test = SystemTime::now();
        // while SystemTime::now().duration_since(&test).unwrap().as_secs() < 1{
        //     spin_loop();
        // }
        let (u8_duration, u16_duration, u32_duration, u64_duration) = unsafe { raw_memcpy_test() };
        durations[i] = u8_duration;
        durations[i + TEST_TIMES] = u16_duration;
        durations[i + 2 * TEST_TIMES] = u32_duration;
        durations[i + 3 * TEST_TIMES] = u64_duration;
        // let test = SystemTime::now();
        // while SystemTime::now().duration_since(&test).unwrap().as_secs() < 1{
        //     spin_loop();
        // }
    }
    early_println!("Duration: {:#?}", durations);

    let (mut u8_duration, mut u16_duration, mut u32_duration, mut u64_duration) = (
        Duration::new(0, 0),
        Duration::new(0, 0),
        Duration::new(0, 0),
        Duration::new(0, 0),
    );

    for i in 0..TEST_TIMES {
        u8_duration += durations[i];
        u16_duration += durations[i + TEST_TIMES];
        u32_duration += durations[i + 2 * TEST_TIMES];
        u64_duration += durations[i + 3 * TEST_TIMES];
    }

    u8_duration /= TEST_TIMES as u32;
    u16_duration /= TEST_TIMES as u32;
    u32_duration /= TEST_TIMES as u32;
    u64_duration /= TEST_TIMES as u32;

    early_println!(" u8 Average time: {:?}", u8_duration);
    early_println!(" u16 Average time: {:?}", u16_duration);
    early_println!(" u32 Average time: {:?}", u32_duration);
    early_println!(" u64 Average time: {:?}", u64_duration);

    loop {}
}

unsafe fn segment_test1() -> Duration {
    let segment = FrameAllocOptions::new(16).alloc_contiguous().unwrap();

    const ITERATION: usize = 100000;

    let start = SystemTime::now();
    for _i in 0..ITERATION {
        for i in 0..(16 * PAGE_SIZE / BASE_UNIT) {
            segment.write_bytes(1024, &VALUE).unwrap();
            segment.read_bytes(1024, &mut VALUE).unwrap();
        }
    }
    SystemTime::now().duration_since(&start).unwrap()
}

unsafe fn segment_test2() -> Duration {
    let segment = FrameAllocOptions::new(30).alloc_contiguous().unwrap();

    const ITERATION: usize = 100000;

    const RW_ITER: usize = 100;

    let start = SystemTime::now();
    for _i in 0..ITERATION {
        for i in 0..RW_ITER {
            segment.write_bytes(i * BASE_UNIT, &VALUE).unwrap();
            segment.read_bytes(i * BASE_UNIT, &mut VALUE).unwrap();
        }
    }
    SystemTime::now().duration_since(&start).unwrap()
}

fn task_test() -> Duration {
    fn dummy() {}

    const TASKS: usize = 100000;

    let mut tasks = Vec::with_capacity(TASKS);

    let start = SystemTime::now();
    for _i in 0..TASKS {
        tasks.push(TaskOptions::new(dummy).data(0).build().unwrap());
    }
    let duration = SystemTime::now().duration_since(&start).unwrap();
    drop(tasks);
    duration
}

fn kernel_stack_test() -> Duration {
    fn dummy() {}

    const STACKS: usize = 100000;

    let mut stacks = Vec::with_capacity(STACKS);

    let start = SystemTime::now();
    for _i in 0..STACKS {
        stacks.push(KernelStack::new_without_guard_page());
    }
    let duration = SystemTime::now().duration_since(&start).unwrap();
    drop(stacks);
    duration
}

fn io_mem_with_device_test() -> (Duration, Duration, Duration, Duration) {
    let mut io_mem = None;
    for (string, dev) in aster_block::all_devices() {
        io_mem = dev.get_io_mem();
    }

    let io_mem = io_mem.unwrap();

    // We simulate the operation to the virtio device.
    // u8: device_status = 15 (offset = 20)
    // u16: queue_select = 0 (offset = 22)
    // u32: device_feature_select = 0 (offset = 0)
    // u64: queue_desc = (Allocate a frame).paddr (offset = 32)

    const ITERATION: usize = 100000 * 1;

    let mut u8_duration = Duration::new(0, 0);
    let mut u16_duration = Duration::new(0, 0);
    let mut u32_duration = Duration::new(0, 0);
    let mut u64_duration = Duration::new(0, 0);

    let start = SystemTime::now();
    for _i in 0..ITERATION {
        io_mem.read_once::<u8>(20).unwrap();
        io_mem.write_once(20, &15u8).unwrap();
    }
    u8_duration = SystemTime::now().duration_since(&start).unwrap();

    let start = SystemTime::now();
    for _i in 0..ITERATION {
        io_mem.read_once::<u16>(22).unwrap();
        io_mem.write_once(22, &0u16).unwrap();
    }
    u16_duration = SystemTime::now().duration_since(&start).unwrap();

    let start = SystemTime::now();
    for _i in 0..ITERATION {
        io_mem.read_once::<u32>(0).unwrap();
        io_mem.write_once(0, &0u32).unwrap();
    }
    u32_duration = SystemTime::now().duration_since(&start).unwrap();

    let frame = FrameAllocOptions::new(1).alloc_single().unwrap();
    let paddr = frame.paddr() as u64;

    let start = SystemTime::now();
    for _i in 0..ITERATION {
        io_mem.read_once::<u64>(32).unwrap();
        io_mem.write_once(32, &paddr).unwrap();
    }
    u64_duration = SystemTime::now().duration_since(&start).unwrap();

    (u8_duration, u16_duration, u32_duration, u64_duration)
}

fn io_mem_with_memory_once_test() -> (Duration, Duration, Duration, Duration) {
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

    let u8_duration;
    let u16_duration;
    let u32_duration;
    let u64_duration;

    let start = SystemTime::now();
    for _i in 0..ITERATION {
        for i in 0..RW_ITER {
            io_mem.read_once::<u8>(i * 1).unwrap();
            io_mem.write_once(i * 1, &1u8).unwrap();
        }
    }
    u8_duration = SystemTime::now().duration_since(&start).unwrap();

    let start = SystemTime::now();
    for _i in 0..ITERATION {
        for i in 0..RW_ITER {
            io_mem.read_once::<u16>(i * 2).unwrap();
            io_mem.write_once(i * 2, &1u16).unwrap();
        }
    }
    u16_duration = SystemTime::now().duration_since(&start).unwrap();

    let start = SystemTime::now();
    for _i in 0..ITERATION {
        for i in 0..RW_ITER {
            io_mem.read_once::<u32>(i * 4).unwrap();
            io_mem.write_once(i * 4, &1u32).unwrap();
        }
    }
    u32_duration = SystemTime::now().duration_since(&start).unwrap();

    let start = SystemTime::now();
    for _i in 0..ITERATION {
        for i in 0..RW_ITER {
            io_mem.read_once::<u64>(i * 8).unwrap();
            io_mem.write_once(i * 8, &1u64).unwrap();
        }
    }
    u64_duration = SystemTime::now().duration_since(&start).unwrap();

    (u8_duration, u16_duration, u32_duration, u64_duration)
}

#[no_mangle]
unsafe fn raw_volatile_ptr_test() -> (Duration, Duration, Duration, Duration) {
    const ITERATION: usize = 100000 * 1;
    const RW_ITER: usize = 100;

    let frame = FrameAllocOptions::new(1).alloc_single().unwrap();

    let raw_u8_ptr = paddr_to_vaddr(frame.start_paddr()) as *mut u8;

    let u8_duration;
    let u16_duration;
    let u32_duration;
    let u64_duration;

    let start = SystemTime::now();
    for _i in 0..ITERATION {
        for i in 0..RW_ITER {
            core::ptr::read_volatile(raw_u8_ptr.add(i * 1).cast::<u8>());
            core::ptr::write_volatile(raw_u8_ptr.add(i * 1).cast::<u8>(), 1);
        }
    }
    u8_duration = SystemTime::now().duration_since(&start).unwrap();

    let start = SystemTime::now();
    for _i in 0..ITERATION {
        for i in 0..RW_ITER {
            core::ptr::read_volatile(raw_u8_ptr.add(i * 2).cast::<u16>());
            core::ptr::write_volatile(raw_u8_ptr.add(i * 2).cast::<u16>(), 1);
        }
    }
    u16_duration = SystemTime::now().duration_since(&start).unwrap();

    let start = SystemTime::now();
    for _i in 0..ITERATION {
        for i in 0..RW_ITER {
            core::ptr::read_volatile(raw_u8_ptr.add(i * 4).cast::<u32>());
            core::ptr::write_volatile(raw_u8_ptr.add(i * 4).cast::<u32>(), 1);
        }
    }
    u32_duration = SystemTime::now().duration_since(&start).unwrap();

    let start = SystemTime::now();
    for _i in 0..ITERATION {
        for i in 0..RW_ITER {
            core::ptr::read_volatile(raw_u8_ptr.add(i * 8).cast::<u64>());
            core::ptr::write_volatile(raw_u8_ptr.add(i * 8).cast::<u64>(), 1);
        }
    }
    u64_duration = SystemTime::now().duration_since(&start).unwrap();

    (u8_duration, u16_duration, u32_duration, u64_duration)
}

#[no_mangle]
unsafe fn raw_memcpy_test() -> (Duration, Duration, Duration, Duration) {
    const ITERATION: usize = 100000 * 1;
    const RW_ITER: usize = 100;

    let frame = FrameAllocOptions::new(1).alloc_single().unwrap();

    let buf_frame = FrameAllocOptions::new(1).alloc_single().unwrap();
    buf_frame.write_bytes(0, &[1, 1, 1, 1, 1, 1, 1, 1]).unwrap();

    let first_u8_ptr_mut = paddr_to_vaddr(frame.start_paddr()) as *mut u8;
    let second_u8_ptr_mut = paddr_to_vaddr(buf_frame.start_paddr()) as *mut u8;

    let u8_duration;
    let u16_duration;
    let u32_duration;
    let u64_duration;

    let start = SystemTime::now();
    for _i in 0..ITERATION {
        for i in 0..RW_ITER {
            __memcpy_fallible(first_u8_ptr_mut.add(i * 1), second_u8_ptr_mut, 1);
            __memcpy_fallible(second_u8_ptr_mut, first_u8_ptr_mut.add(i * 1), 1);
        }
    }
    u8_duration = SystemTime::now().duration_since(&start).unwrap();

    let start = SystemTime::now();
    for _i in 0..ITERATION {
        for i in 0..RW_ITER {
            __memcpy_fallible(first_u8_ptr_mut.add(i * 2), second_u8_ptr_mut, 2);
            __memcpy_fallible(second_u8_ptr_mut, first_u8_ptr_mut.add(i * 2), 2);
        }
    }
    u16_duration = SystemTime::now().duration_since(&start).unwrap();

    let start = SystemTime::now();
    for _i in 0..ITERATION {
        for i in 0..RW_ITER {
            __memcpy_fallible(first_u8_ptr_mut.add(i * 4), second_u8_ptr_mut, 4);
            __memcpy_fallible(second_u8_ptr_mut, first_u8_ptr_mut.add(i * 4), 4);
        }
    }
    u32_duration = SystemTime::now().duration_since(&start).unwrap();

    let start = SystemTime::now();
    for _i in 0..ITERATION {
        for i in 0..RW_ITER {
            __memcpy_fallible(first_u8_ptr_mut.add(i * 8), second_u8_ptr_mut, 8);
            __memcpy_fallible(second_u8_ptr_mut, first_u8_ptr_mut.add(i * 8), 8);
        }
    }
    u64_duration = SystemTime::now().duration_since(&start).unwrap();

    (u8_duration, u16_duration, u32_duration, u64_duration)
}
