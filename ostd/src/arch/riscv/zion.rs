// SPDX-License-Identifier: MPL-2.0

//! Zion-specific SBI calls for RISC-V CVM guests.

const TVM_SBI_EXT_ID: usize = 0x8424_b45;
const SBI_SM_REGISTER_SHARED_MEM_WITH_REE: usize = 1030;
const SBI_SM_FREE_SHARED_MEM_WITH_REE: usize = 1031;

/// Registers a shared-memory range with the Zion host.
pub fn register_shared_mem(addr: usize, nr_pages: usize) -> usize {
    sbi_call_2(
        TVM_SBI_EXT_ID,
        SBI_SM_REGISTER_SHARED_MEM_WITH_REE,
        addr,
        nr_pages,
    )
    .error
}

/// Releases a shared-memory range from the Zion host.
pub fn free_shared_mem(addr: usize, nr_pages: usize) -> usize {
    sbi_call_2(
        TVM_SBI_EXT_ID,
        SBI_SM_FREE_SHARED_MEM_WITH_REE,
        addr,
        nr_pages,
    )
    .error
}

fn sbi_call_2(eid: usize, fid: usize, arg0: usize, arg1: usize) -> sbi_rt::SbiRet {
    let (error, value);

    // SAFETY: The inline assembly follows the SBI calling convention: a7 is
    // the extension ID, a6 is the function ID, and a0/a1 carry arguments and
    // return the SBI error/value pair after `ecall`.
    unsafe {
        core::arch::asm!(
            "ecall",
            in("a7") eid,
            in("a6") fid,
            inlateout("a0") arg0 => error,
            inlateout("a1") arg1 => value,
        );
    }

    sbi_rt::SbiRet { error, value }
}
