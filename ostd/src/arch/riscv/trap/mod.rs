// SPDX-License-Identifier: MPL-2.0

//! Handles trap.

mod trap;

use align_ext::AlignExt;
use log::debug;
use riscv::register::scause::{Exception, Interrupt};
pub use trap::{GeneralRegs, TrapFrame, UserContext};

use crate::{
    arch::timer::timer_callback,
    cpu_local_cell,
    mm::{
        kspace::{KERNEL_PAGE_TABLE, LINEAR_MAPPING_BASE_VADDR, LINEAR_MAPPING_VADDR_RANGE},
        CachePolicy, PageFlags, PageProperty, PrivilegedPageFlags, PAGE_SIZE,
    },
    trap::disable_local,
};

cpu_local_cell! {
    static IS_KERNEL_INTERRUPTED: bool = false;
}

/// Initialize interrupt handling on RISC-V.
pub unsafe fn init(on_bsp: bool) {
    self::trap::init();
}

/// Returns true if this function is called within the context of an IRQ handler
/// and the IRQ occurs while the CPU is executing in the kernel mode.
/// Otherwise, it returns false.
pub fn is_kernel_interrupted() -> bool {
    IS_KERNEL_INTERRUPTED.load()
}

/// Handle traps (only from kernel).
#[no_mangle]
extern "C" fn trap_handler(f: &mut TrapFrame) {
    use riscv::register::scause::Trap;

    match riscv::register::scause::read().cause() {
        Trap::Interrupt(interrupt) => {
            let _guard = disable_local();
            IS_KERNEL_INTERRUPTED.store(true);
            match interrupt {
                Interrupt::SupervisorSoft => todo!(),
                Interrupt::SupervisorTimer => timer_callback(),
                Interrupt::SupervisorExternal => todo!(),
                Interrupt::Unknown => todo!(),
            }
            IS_KERNEL_INTERRUPTED.store(false);
        }
        Trap::Exception(e) => {
            let stval = riscv::register::stval::read();

            // match e {
            //     Exception::StorePageFault
            //     | Exception::LoadPageFault
            //     | Exception::InstructionPageFault => {
            //         handle_kernel_page_fault(e, f, stval as u64);
            //     }
            //     _ => {}
            // }
            panic!(
                "Cannot handle kernel cpu exception: {e:?}. stval: {stval:#x}, trapframe: {f:#x?}.",
            );
        }
    }
}

/// FIXME: this is a hack because we don't allocate kernel space for IO memory. We are currently
/// using the linear mapping for IO memory. This is not a good practice.
fn handle_kernel_page_fault(exception: Exception, f: &TrapFrame, page_fault_vaddr: u64) {
    debug!(
        "kernel page fault: address {:?}, exception {:?}",
        page_fault_vaddr as *const (), exception
    );

    assert!(
        LINEAR_MAPPING_VADDR_RANGE.contains(&(page_fault_vaddr as usize)),
        "kernel page fault: the address is outside the range of the linear mapping",
    );

    // Do the mapping
    let page_table = KERNEL_PAGE_TABLE
        .get()
        .expect("kernel page fault: the kernel page table is not initialized");
    let vaddr = (page_fault_vaddr as usize).align_down(PAGE_SIZE);
    let paddr = vaddr - LINEAR_MAPPING_BASE_VADDR;

    let priv_flags = PrivilegedPageFlags::GLOBAL;

    // SAFETY:
    // 1. We have checked that the page fault address falls within the address range of the direct
    //    mapping of physical memory.
    // 2. We map the address to the correct physical page with the correct flags, where the
    //    correctness follows the semantics of the direct mapping of physical memory.
    unsafe {
        page_table
            .map(
                &(vaddr..vaddr + PAGE_SIZE),
                &(paddr..paddr + PAGE_SIZE),
                PageProperty {
                    flags: PageFlags::RW,
                    cache: CachePolicy::Uncacheable,
                    priv_flags,
                },
            )
            .unwrap();
    }
}
