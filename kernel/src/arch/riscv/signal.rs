// SPDX-License-Identifier: MPL-2.0

use ostd::cpu::{CpuExceptionInfo, UserContext};
use riscv::register::scause::Exception;

use crate::process::signal::{
    constants::{ILL_ILLOPC, SIGILL},
    sig_num::SigNum,
    signals::fault::FaultSignal,
    SignalContext,
};

impl SignalContext for UserContext {
    fn set_arguments(&mut self, sig_num: SigNum, siginfo_addr: usize, ucontext_addr: usize) {
        self.set_a0(sig_num.as_u8() as usize);
        self.set_a1(siginfo_addr);
        self.set_a2(ucontext_addr);
    }
}

impl From<&CpuExceptionInfo> for FaultSignal {
    fn from(trap_info: &CpuExceptionInfo) -> Self {
        log::info!("trap_info: {:?}", trap_info);
        let (num, code, addr) = match trap_info.code {
            Exception::InstructionMisaligned => todo!(),
            Exception::InstructionFault => todo!(),
            Exception::IllegalInstruction => (SIGILL, ILL_ILLOPC, None),
            Exception::Breakpoint => todo!(),
            Exception::LoadMisaligned => todo!(),
            Exception::LoadFault => todo!(),
            Exception::StoreMisaligned => todo!(),
            Exception::StoreFault => todo!(),
            Exception::UserEnvCall => todo!(),
            Exception::SupervisorEnvCall => todo!(),
            Exception::InstructionPageFault => todo!(),
            Exception::LoadPageFault => todo!(),
            Exception::StorePageFault => todo!(),
            Exception::Unknown => todo!(),
        };
        FaultSignal::new(num, code, addr)
    }
}
