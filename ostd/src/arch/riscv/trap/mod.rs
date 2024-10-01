// SPDX-License-Identifier: MPL-2.0

//! Handles trap.

mod trap;

use riscv::register::scause::{Interrupt, Trap};
pub use trap::{GeneralRegs, TrapFrame, UserContext};

use super::{device::plic::claim_interrupt, timer::timer_callback};
use crate::{cpu_local_cell, trap::call_irq_callback_functions};

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
extern "C" fn trap_handler(trap_frame: &mut TrapFrame) {
    match riscv::register::scause::read().cause() {
        Trap::Interrupt(interrupt) => {
            IS_KERNEL_INTERRUPTED.store(true);
            match interrupt {
                Interrupt::SupervisorSoft => todo!(),
                Interrupt::SupervisorTimer => timer_callback(),
                Interrupt::SupervisorExternal => {
                    while let irq = claim_interrupt()
                        && irq != 0
                    {
                        call_irq_callback_functions(trap_frame, irq as usize);
                    }
                }
                Interrupt::Unknown => todo!(),
            }
            IS_KERNEL_INTERRUPTED.store(false);
        }
        Trap::Exception(e) => {
            let stval = riscv::register::stval::read();
            panic!(
                "Cannot handle kernel cpu exception: {e:?}. stval: {stval:#x}, trapframe: {trap_frame:#x?}.",
            );
        }
    }
}
