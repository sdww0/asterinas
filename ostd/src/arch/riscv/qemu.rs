// SPDX-License-Identifier: MPL-2.0

//! Providing the ability to exit QEMU and return a value as debug result.

/// The exit code of QEMU.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QemuExitCode {
    /// The code that indicates a successful exit.
    Success,
    /// The code that indicates a failed exit.
    Failed,
}

/// Exit QEMU with the given exit code.
///
/// This function assumes that the kernel is run in QEMU with the following
/// QEMU command line arguments that specifies the ISA debug exit device:
/// `-device isa-debug-exit,iobase=0xf4,iosize=0x04`.
pub fn exit_qemu(exit_code: QemuExitCode) -> ! {
    log::debug!("exit qemu with exit code {exit_code:?}");
    match exit_code {
        QemuExitCode::Success => sbi_rt::system_reset(sbi_rt::Shutdown, sbi_rt::NoReason),
        QemuExitCode::Failed => sbi_rt::system_reset(sbi_rt::Shutdown, sbi_rt::SystemFailure),
    };
    unreachable!("qemu does not exit");
}
