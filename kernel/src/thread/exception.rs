// SPDX-License-Identifier: MPL-2.0

#[cfg(any(target_arch = "x86_64", target_arch = "riscv64"))]
use ostd::arch::cpu::context::CpuException;
#[cfg(target_arch = "loongarch64")]
use ostd::arch::cpu::context::CpuExceptionInfo as CpuException;
use ostd::{arch::cpu::context::UserContext, task::Task, user::UserContextApi};

use crate::{
    prelude::*,
    process::signal::signals::fault::FaultSignal,
    vm::vmar::{PageFaultInfo, Vmar},
};

pub(super) fn handle_exception(ctx: &Context, user_ctx: &UserContext, exception: CpuException) {
    debug!("handle exception: {:#x?}", exception);

    if let Ok(page_fault_info) = PageFaultInfo::try_from(&exception) {
        let user_space = ctx.user_space();
        let vmar = user_space.vmar();
        if handle_page_fault_from_vmar(vmar, &page_fault_info).is_ok() {
            return;
        }
    }

    log_unhandled_user_exception_context(user_ctx);

    // We cannot handle most exceptions. Send a fault signal to the current thread before returning
    // to user space.
    generate_fault_signal(exception, ctx, user_ctx);
}

#[cfg(target_arch = "riscv64")]
fn log_unhandled_user_exception_context(user_ctx: &UserContext) {
    let regs = user_ctx.general_regs();
    warn!(
        "unhandled user exception context: sepc={:#x}, ra={:#x}, sp={:#x}, gp={:#x}, tp={:#x}, \
         t0={:#x}, t1={:#x}, t2={:#x}, a0={:#x}, a1={:#x}, a2={:#x}, a3={:#x}, \
         a4={:#x}, a5={:#x}, a6={:#x}, a7={:#x}",
        user_ctx.instruction_pointer(),
        regs.ra,
        regs.sp,
        regs.gp,
        regs.tp,
        regs.t0,
        regs.t1,
        regs.t2,
        regs.a0,
        regs.a1,
        regs.a2,
        regs.a3,
        regs.a4,
        regs.a5,
        regs.a6,
        regs.a7
    );
}

#[cfg(not(target_arch = "riscv64"))]
fn log_unhandled_user_exception_context(user_ctx: &UserContext) {
    warn!(
        "unhandled user exception context: ip={:#x}, sp={:#x}",
        user_ctx.instruction_pointer(),
        user_ctx.stack_pointer()
    );
}

/// Handles the page fault occurs in the VMAR.
fn handle_page_fault_from_vmar(vmar: &Vmar, page_fault_info: &PageFaultInfo) -> Result<(), ()> {
    if let Err(e) = vmar.handle_page_fault(page_fault_info) {
        warn!(
            "page fault handler failed: info: {:#x?}, err: {:?}",
            page_fault_info, e
        );
        return Err(());
    }
    Ok(())
}

/// A trait that converts CPU exceptions into fault signals.
///
/// This trait should be implemented by architecture-specific code for [`CpuException`].
pub trait ToFaultSignal {
    /// Converts a CPU exception into a fault signal.
    ///
    /// Returns `None` if the exception must be handled earlier and cannot be delivered as a signal.
    ///
    /// POSIX [requires] `SIGILL` and `SIGFPE` to report the address of the faulting instruction,
    /// and `SIGSEGV` and `SIGBUS` to report the address of the faulting memory reference. Linux
    /// behavior, however, is highly architecture-specific and does not always follow POSIX: for
    /// some exceptions, it reports neither the expected code nor the expected address. Linux may
    /// also attach an instruction address to `SIGTRAP`, but this behavior is not consistent across
    /// architectures.
    ///
    /// [requires]: https://pubs.opengroup.org/onlinepubs/009695399/basedefs/signal.h.html
    fn to_fault_signal(&self, user_ctx: &UserContext) -> Option<FaultSignal>;
}

/// Generates a fault signal for the current thread.
fn generate_fault_signal(exception: CpuException, ctx: &Context, user_ctx: &UserContext) {
    let Some(signal) = exception.to_fault_signal(user_ctx) else {
        panic!("`{:?}` cannot be handled via signals", exception);
    };
    ctx.posix_thread.enqueue_signal(Box::new(signal));
}

pub(super) fn page_fault_handler(info: &CpuException) -> Result<(), ()> {
    let task = Task::current().unwrap();
    let thread_local = task.as_thread_local().unwrap();

    if thread_local.is_page_fault_disabled() {
        // Do nothing if the page fault handler is disabled. This will typically cause the fallible
        // memory operation to report `EFAULT` errors immediately.
        return Err(());
    }

    let user_space = CurrentUserSpace::new(thread_local);
    handle_page_fault_from_vmar(user_space.vmar(), &info.try_into().unwrap())
}
